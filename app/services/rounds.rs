use crate::entity::round_participation::ParticipationState;
use crate::entity::{round, round_participation, student, task, task_score};
use crate::error::{AppError, AppResult};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait, IntoActiveModel, ModelTrait,
};
use sea_orm::{ColumnTrait, QueryOrder};
use sea_orm::{QueryFilter, QuerySelect};
use std::collections::{HashMap, HashSet};

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
        .ok_or(AppError::Internal(anyhow::anyhow!(
            "Logic Error: No first round for edition"
        )))
}

pub async fn update_participation(
    participation: round_participation::Model,
    state: Option<ParticipationState>,
    task_points: Vec<(i32, f32)>,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let participation_id = participation.id;
    let mut participation = participation.into_active_model();
    if let Some(state) = state {
        participation.state = ActiveValue::Set(state);
    }
    let mut scores: HashMap<i32, task_score::ActiveModel> = participation
        .find_related(task_score::Entity)
        .all(db)
        .await?
        .into_iter()
        .map(|x| (x.task_id, x.into_active_model()))
        .collect();

    for (task_id, score) in task_points {
        scores
            .entry(task_id)
            .or_insert_with(|| task_score::ActiveModel::create_from_ids(task_id, participation_id))
            .points = ActiveValue::Set(score);
    }

    for (_, score) in scores {
        score.save(db).await?;
    }

    participation.save(db).await?;

    Ok(())
}

pub async fn move_students(
    target_round: round::Model,
    students: Vec<i32>,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
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

    let to_insert: Vec<round_participation::ActiveModel> = students
        .into_iter()
        .filter(|student_id| !existing.contains(student_id))
        .map(|student_id| round_participation::ActiveModel {
            id: ActiveValue::NotSet,
            round_id: ActiveValue::Set(target_round.id),
            student_id: ActiveValue::Set(student_id),
            state: ActiveValue::Set(ParticipationState::Invited),
            ..Default::default()
        })
        .collect();

    if !to_insert.is_empty() {
        round_participation::Entity::insert_many(to_insert)
            .exec(db)
            .await?;
    }

    Ok(())
}

pub async fn statistics_for_rounds(
    round_ids: &[i32],
    db: &impl ConnectionTrait,
) -> AppResult<HashMap<i32, Statistics>> {
    let stats: Vec<(i32, Option<u32>, u32)> = task::Entity::find()
        .filter(task::COLUMN.id.is_in(round_ids.to_vec()))
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
        .column(round_participation::Column::RoundId) // todo: why can we not use typed column here?
        .column_as(round_participation::COLUMN.id.count(), "count")
        .group_by(round_participation::COLUMN.round_id)
        .into_tuple()
        .all(db)
        .await?;

    let mut participant_map: HashMap<i32, i64> = participant_counts.into_iter().collect();

    Ok(stats
        .into_iter()
        .map(|(round_id, total_points, task_count)| {
            let participant_count = participant_map.remove(&round_id).unwrap_or(0) as u32;
            (
                round_id,
                Statistics {
                    total_points: total_points.unwrap_or(0),
                    task_count,
                    participant_count,
                },
            )
        })
        .collect())
}

pub async fn statistics_for_round(
    round: &round::Model,
    db: &impl ConnectionTrait,
) -> AppResult<Statistics> {
    let stats = statistics_for_rounds(&[round.id], db).await?;
    Ok(stats
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Could not calculate statistics".into()))?
        .1)
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
