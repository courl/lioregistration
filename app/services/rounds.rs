use crate::entity::registration::ReviewStatus;
use crate::entity::round_participation::ParticipationState;
use crate::entity::{
    credentials, registration, round, round_participation, student, task, task_score,
};
use crate::error::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, ModelTrait, PaginatorTrait, TransactionTrait,
};
use sea_orm::{ColumnTrait, QueryOrder};
use sea_orm::{QueryFilter, QuerySelect};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Default)]
pub struct Statistics {
    pub total_points: u32,
    pub task_count: u32,
    pub participant_count: u32,
}

pub struct Participant {
    pub student: student::Model,
    pub participation: round_participation::Model,
    pub scores: Vec<task_score::Model>,
}

pub struct Details {
    pub tasks: Vec<task::Model>,
    pub participants: Vec<Participant>,
}

pub async fn get_first_round(
    edition_id: i32,
    db: &impl ConnectionTrait,
) -> AppResult<round::Model> {
    round::Entity::find_by_edition(edition_id)
        .order_by_asc(round::COLUMN.starts_at)
        .one(db)
        .await?
        .ok_or_else(|| {
            AppError::Conflict("This edition has no round yet; create a round first".into())
        })
}

/// Sets a participation's state and/or task scores atomically. See [`update_participation_in`].
pub async fn update_participation(
    participation: round_participation::Model,
    state: Option<ParticipationState>,
    task_points: Vec<(i32, f32)>,
    db: &DatabaseConnection,
) -> AppResult<()> {
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(
            async move { update_participation_in(participation, state, task_points, txn).await },
        )
    })
    .await
    .map_err(AppError::from)
}

/// Composable body of [`update_participation`]: every task must belong to the participation's
/// round and every score must lie in `0..=max_points`; only the given scores are written.
pub async fn update_participation_in(
    participation: round_participation::Model,
    state: Option<ParticipationState>,
    task_points: Vec<(i32, f32)>,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let tasks: HashMap<i32, task::Model> = task::Entity::find()
        .filter(task::COLUMN.round_id.eq(participation.round_id))
        .all(db)
        .await?
        .into_iter()
        .map(|task| (task.id, task))
        .collect();

    // Last value wins for a task listed twice.
    let task_points: HashMap<i32, f32> = task_points.into_iter().collect();
    for (task_id, points) in &task_points {
        let task = tasks.get(task_id).ok_or_else(|| {
            AppError::BadRequest(format!("Task {task_id} does not belong to this round"))
        })?;
        validate_points(*points, task)?;
    }

    let mut existing: HashMap<i32, task_score::Model> = participation
        .find_related(task_score::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|score| (score.task_id, score))
        .collect();

    for (task_id, points) in task_points {
        if let Some(score) = existing.remove(&task_id) {
            if score.points != points {
                let mut score = score.into_active_model();
                score.points = ActiveValue::Set(points);
                score.update(db).await?;
            }
        } else {
            task_score::ActiveModel::create_from_ids(task_id, participation.id, points)
                .insert(db)
                .await?;
        }
    }

    if let Some(state) = state
        && state != participation.state
    {
        let mut participation = participation.into_active_model();
        participation.state = ActiveValue::Set(state);
        participation.update(db).await?;
    }

    Ok(())
}

fn validate_points(points: f32, task: &task::Model) -> AppResult<()> {
    #[allow(clippy::cast_precision_loss)]
    let max = task.max_points as f32;
    if !points.is_finite() || points < 0.0 || points > max {
        return Err(AppError::BadRequest(format!(
            "Score {points} for task '{}' is outside 0..={}",
            task.name, task.max_points
        )));
    }
    Ok(())
}

/// Imports scores keyed by contest username (credentials) in one transaction. Rows that can't
/// be matched to a participant of `round` are skipped and their usernames returned, so the
/// caller can report them instead of losing them silently. Any invalid score aborts the import.
pub async fn import_scores(
    round: &round::Model,
    rows: Vec<(String, Vec<(i32, f32)>)>,
    db: &DatabaseConnection,
) -> AppResult<Vec<String>> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let round_id = round.id;
    let edition_id = round.edition_id;
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            let usernames: Vec<String> = rows.iter().map(|(name, _)| name.clone()).collect();
            let credentials = credentials::Entity::find()
                .filter(credentials::COLUMN.edition_id.eq(edition_id))
                .filter(credentials::COLUMN.username.is_in(usernames))
                .all(txn)
                .await?;

            let registration_ids: Vec<i32> = credentials
                .iter()
                .filter_map(|c| c.registration_id)
                .collect();
            let student_by_registration: HashMap<i32, i32> = registration::Entity::find()
                .filter(registration::COLUMN.id.is_in(registration_ids))
                .all(txn)
                .await?
                .into_iter()
                .filter_map(|reg| Some((reg.id, reg.student_id?)))
                .collect();

            let student_by_username: HashMap<String, i32> = credentials
                .into_iter()
                .filter_map(|c| {
                    let student_id = *student_by_registration.get(&c.registration_id?)?;
                    Some((c.username, student_id))
                })
                .collect();

            let mut participations: HashMap<i32, round_participation::Model> =
                round_participation::Entity::find()
                    .filter(round_participation::COLUMN.round_id.eq(round_id))
                    .filter(
                        round_participation::COLUMN
                            .student_id
                            .is_in(student_by_username.values().copied()),
                    )
                    .all(txn)
                    .await?
                    .into_iter()
                    .map(|p| (p.student_id, p))
                    .collect();

            let mut unmatched = Vec::new();
            for (username, task_points) in rows {
                let participation = student_by_username
                    .get(&username)
                    .and_then(|student_id| participations.remove(student_id));
                match participation {
                    Some(participation) => {
                        update_participation_in(participation, None, task_points, txn).await?;
                    }
                    None => unmatched.push(username),
                }
            }
            Ok(unmatched)
        })
    })
    .await
    .map_err(AppError::from)
}

/// Invites students to `target_round`. Students already in the round are skipped; every other
/// student must hold a validated registration in the round's edition.
pub async fn move_students(
    target_round: round::Model,
    students: Vec<i32>,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let students: BTreeSet<i32> = students.into_iter().collect();
    if students.is_empty() {
        return Ok(());
    }

    let existing: HashSet<i32> = target_round
        .find_related(round_participation::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.student_id)
        .collect();
    let new_students: Vec<i32> = students
        .into_iter()
        .filter(|student_id| !existing.contains(student_id))
        .collect();
    if new_students.is_empty() {
        return Ok(());
    }

    let eligible: HashSet<i32> = registration::Entity::find()
        .filter(registration::COLUMN.edition_id.eq(target_round.edition_id))
        .filter(
            registration::COLUMN
                .review_status
                .eq(ReviewStatus::Validated),
        )
        .filter(registration::COLUMN.student_id.is_in(new_students.clone()))
        .all(db)
        .await?
        .into_iter()
        .filter_map(|reg| reg.student_id)
        .collect();
    let ineligible: Vec<i32> = new_students
        .iter()
        .copied()
        .filter(|id| !eligible.contains(id))
        .collect();
    if !ineligible.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Students without a validated registration in this edition: {ineligible:?}"
        )));
    }

    round_participation::Entity::insert_many(new_students.into_iter().map(|student_id| {
        round_participation::ActiveModel {
            id: ActiveValue::NotSet,
            round_id: ActiveValue::Set(target_round.id),
            student_id: ActiveValue::Set(student_id),
            state: ActiveValue::Set(ParticipationState::Invited),
            award: ActiveValue::Set(None),
        }
    }))
    .exec(db)
    .await?;

    Ok(())
}

/// Deletes a round, refusing while it still has tasks or participants (their scores would go
/// with them).
pub async fn delete_round(round: round::Model, db: &impl ConnectionTrait) -> AppResult<()> {
    let participants = round
        .find_related(round_participation::Entity)
        .count(db)
        .await?;
    let tasks = round.find_related(task::Entity).count(db).await?;
    if participants > 0 || tasks > 0 {
        return Err(AppError::Conflict(format!(
            "Round still has {tasks} task(s) and {participants} participant(s); remove them first"
        )));
    }
    round.delete(db).await?;
    Ok(())
}

/// Deletes a task, refusing while scores are recorded for it.
pub async fn delete_task(task: task::Model, db: &impl ConnectionTrait) -> AppResult<()> {
    let scores = task.find_related(task_score::Entity).count(db).await?;
    if scores > 0 {
        return Err(AppError::Conflict(format!(
            "Task has {scores} recorded score(s); it cannot be deleted"
        )));
    }
    task.delete(db).await?;
    Ok(())
}

/// Rejects lowering a task's `max_points` below a score that is already recorded.
pub async fn ensure_max_points_fit(
    task: &task::Model,
    max_points: u32,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let highest = task
        .find_related(task_score::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|score| score.points)
        .fold(0.0_f32, f32::max);
    #[allow(clippy::cast_precision_loss)]
    if highest > max_points as f32 {
        return Err(AppError::Conflict(format!(
            "A recorded score ({highest}) exceeds the new maximum of {max_points}"
        )));
    }
    Ok(())
}

pub async fn statistics_for_rounds(
    round_ids: &[i32],
    db: &impl ConnectionTrait,
) -> AppResult<HashMap<i32, Statistics>> {
    let mut result: HashMap<i32, Statistics> = round_ids
        .iter()
        .map(|id| (*id, Statistics::default()))
        .collect();

    let task_stats: Vec<(i32, Option<i64>, i64)> = task::Entity::find()
        .filter(task::COLUMN.round_id.is_in(round_ids.to_vec()))
        .select_only()
        .column(task::Column::RoundId)
        .column_as(task::COLUMN.max_points.sum(), "total_points")
        .column_as(task::COLUMN.id.count(), "task_count")
        .group_by(task::COLUMN.round_id)
        .into_tuple()
        .all(db)
        .await?;

    let participant_counts: Vec<(i32, i64)> = round_participation::Entity::find()
        .filter(
            round_participation::COLUMN
                .round_id
                .is_in(round_ids.to_vec()),
        )
        .filter(
            round_participation::COLUMN
                .state
                .eq(ParticipationState::Accepted),
        )
        .select_only()
        .column(round_participation::Column::RoundId)
        .column_as(round_participation::COLUMN.id.count(), "count")
        .group_by(round_participation::COLUMN.round_id)
        .into_tuple()
        .all(db)
        .await?;

    for (round_id, total_points, task_count) in task_stats {
        if let Some(stats) = result.get_mut(&round_id) {
            stats.total_points = u32::try_from(total_points.unwrap_or(0)).unwrap_or(u32::MAX);
            stats.task_count = u32::try_from(task_count).unwrap_or(u32::MAX);
        }
    }
    for (round_id, count) in participant_counts {
        if let Some(stats) = result.get_mut(&round_id) {
            stats.participant_count = u32::try_from(count).unwrap_or(u32::MAX);
        }
    }

    Ok(result)
}

pub async fn statistics_for_round(
    round: &round::Model,
    db: &impl ConnectionTrait,
) -> AppResult<Statistics> {
    Ok(statistics_for_rounds(&[round.id], db)
        .await?
        .remove(&round.id)
        .unwrap_or_default())
}

pub async fn details(round: &round::Model, db: &impl ConnectionTrait) -> AppResult<Details> {
    // fixme: should we not just directly look up registrations with student that are not refused?
    let id = round.id;
    let (tasks, participation_students, scores) = tokio::try_join!(
        task::Entity::find()
            .filter(task::COLUMN.round_id.eq(id))
            .order_by_asc(task::COLUMN.ordinal)
            .all(db),
        round_participation::Entity::find()
            .filter(round_participation::COLUMN.round_id.eq(id))
            .find_also_related(student::Entity)
            .all(db),
        task_score::Entity::find()
            .find_also_related(round_participation::Entity)
            .filter(round_participation::COLUMN.round_id.eq(id))
            .all(db)
    )?;

    let mut scores_by_participation: HashMap<i32, Vec<task_score::Model>> = HashMap::new();
    for (score, _) in scores {
        scores_by_participation
            .entry(score.round_participation_id)
            .or_default()
            .push(score);
    }

    let participants = participation_students
        .into_iter()
        .map(|(participation, student)| {
            let student = student.ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Database corrupt: participation {} has no associated student",
                    participation.id
                ))
            })?;
            let scores = scores_by_participation
                .remove(&participation.id)
                .unwrap_or_default();

            Ok(Participant {
                student,
                participation,
                scores,
            })
        })
        .collect::<AppResult<Vec<_>>>()?;

    Ok(Details {
        tasks,
        participants,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::tests::{
        create_credential, create_edition, create_participation, create_registration, create_round,
        create_school, create_score, create_student, create_task, date, setup_test,
    };
    use sea_orm::DatabaseConnection;

    struct Fixture {
        round: round::Model,
        task1: task::Model,
        task2: task::Model,
        student: student::Model,
        participation: round_participation::Model,
    }

    /// Edition with one round (2 tasks, max 10 + 20) and one validated, accepted student.
    async fn fixture(db: &DatabaseConnection) -> Fixture {
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let school = create_school(db, "Lycée A").await;
        let student = create_student(db, "Ada", "Lovelace", date(2)).await;
        create_registration(db, &edition, &school, Some(student.id)).await;
        let round = create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        let task1 = create_task(db, round.id, "A", 1, 10).await;
        let task2 = create_task(db, round.id, "B", 2, 20).await;
        let participation =
            create_participation(db, &round, &student, ParticipationState::Accepted).await;
        Fixture {
            round,
            task1,
            task2,
            student,
            participation,
        }
    }

    async fn points(
        db: &DatabaseConnection,
        participation: &round_participation::Model,
    ) -> Vec<(i32, f32)> {
        let mut scores: Vec<(i32, f32)> = participation
            .find_related(task_score::Entity)
            .all(db)
            .await
            .unwrap()
            .into_iter()
            .map(|s| (s.task_id, s.points))
            .collect();
        scores.sort_by_key(|s| s.0);
        scores
    }

    #[tokio::test]
    async fn statistics_cover_rounds_with_and_without_tasks() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;
        let empty = create_round(db, f.round.edition_id, "Round 2", date(10), date(11)).await;

        let stats = statistics_for_rounds(&[f.round.id, empty.id], db)
            .await
            .unwrap();
        let full = &stats[&f.round.id];
        assert_eq!(
            (full.total_points, full.task_count, full.participant_count),
            (30, 2, 1)
        );
        let none = &stats[&empty.id];
        assert_eq!(
            (none.total_points, none.task_count, none.participant_count),
            (0, 0, 0)
        );

        // Regression: a round without tasks used to yield 404 here.
        assert_eq!(
            statistics_for_round(&empty, db).await.unwrap().task_count,
            0
        );
    }

    #[tokio::test]
    async fn update_participation_writes_and_updates_scores() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;

        update_participation(f.participation.clone(), None, vec![(f.task1.id, 7.5)], db)
            .await
            .unwrap();
        update_participation(
            f.participation.clone(),
            Some(ParticipationState::NoShow),
            vec![(f.task1.id, 10.0), (f.task2.id, 0.0)],
            db,
        )
        .await
        .unwrap();

        assert_eq!(
            points(db, &f.participation).await,
            vec![(f.task1.id, 10.0), (f.task2.id, 0.0)]
        );
        let participation = round_participation::Entity::find_by_id(f.participation.id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(participation.state, ParticipationState::NoShow);
    }

    #[tokio::test]
    async fn update_participation_rejects_invalid_scores_atomically() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;
        let other_round = create_round(db, f.round.edition_id, "Round 2", date(10), date(11)).await;
        let foreign = create_task(db, other_round.id, "X", 1, 10).await;

        for bad in [
            vec![(foreign.id, 1.0)],
            vec![(f.task1.id, 10.5)],
            vec![(f.task1.id, -1.0)],
            vec![(f.task1.id, f32::NAN)],
            vec![(f.task2.id, 5.0), (f.task1.id, f32::INFINITY)],
        ] {
            let err = update_participation(f.participation.clone(), None, bad, db)
                .await
                .unwrap_err();
            assert!(matches!(err, AppError::BadRequest(_)), "{err:?}");
        }
        assert!(
            points(db, &f.participation).await.is_empty(),
            "nothing may be written"
        );
    }

    #[tokio::test]
    async fn move_students_dedupes_and_requires_validated_registration() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;
        let round2 = create_round(db, f.round.edition_id, "Round 2", date(10), date(11)).await;
        let outsider = create_student(db, "Grace", "Hopper", date(3)).await;

        // Duplicates in one request used to hit the unique key and 500.
        move_students(round2.clone(), vec![f.student.id, f.student.id], db)
            .await
            .unwrap();
        // Already present: no-op.
        move_students(round2.clone(), vec![f.student.id], db)
            .await
            .unwrap();
        assert_eq!(
            round2
                .find_related(round_participation::Entity)
                .count(db)
                .await
                .unwrap(),
            1
        );

        let err = move_students(round2.clone(), vec![outsider.id], db)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)), "{err:?}");
    }

    #[tokio::test]
    async fn import_scores_reports_unmatched_usernames() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;
        let edition = crate::services::editions::get_edition(f.round.edition_id, db)
            .await
            .unwrap();
        let registration = registration::Entity::find()
            .filter(registration::COLUMN.student_id.eq(f.student.id))
            .one(db)
            .await
            .unwrap()
            .unwrap();
        let mut credential = create_credential(db, "ada01", "pw", &edition)
            .await
            .into_active_model();
        credential.registration_id = ActiveValue::Set(Some(registration.id));
        credential.update(db).await.unwrap();
        create_credential(db, "free01", "pw", &edition).await;

        let unmatched = import_scores(
            &f.round,
            vec![
                ("ada01".into(), vec![(f.task1.id, 4.0)]),
                ("free01".into(), vec![(f.task1.id, 1.0)]),
                ("nobody".into(), vec![(f.task1.id, 1.0)]),
            ],
            db,
        )
        .await
        .unwrap();

        assert_eq!(unmatched, vec!["free01".to_string(), "nobody".to_string()]);
        assert_eq!(points(db, &f.participation).await, vec![(f.task1.id, 4.0)]);
    }

    #[tokio::test]
    async fn deletes_are_blocked_by_dependent_data() {
        let app = setup_test().await;
        let db = &app.state.db;
        let f = fixture(db).await;
        create_score(db, &f.task1, &f.participation, 5.0).await;

        assert!(matches!(
            delete_task(f.task1.clone(), db).await,
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            delete_round(f.round.clone(), db).await,
            Err(AppError::Conflict(_))
        ));
        assert!(matches!(
            ensure_max_points_fit(&f.task1, 4, db).await,
            Err(AppError::Conflict(_))
        ));
        ensure_max_points_fit(&f.task1, 5, db).await.unwrap();

        // An unscored task and an empty round can go.
        delete_task(f.task2.clone(), db).await.unwrap();
        let empty = create_round(db, f.round.edition_id, "Round 2", date(10), date(11)).await;
        delete_round(empty, db).await.unwrap();
    }

    #[tokio::test]
    async fn first_round_missing_is_a_conflict() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        assert!(matches!(
            get_first_round(edition.id, db).await,
            Err(AppError::Conflict(_))
        ));
    }
}
