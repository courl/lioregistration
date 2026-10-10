use crate::entity::{registration, school};
use crate::error::{AppError, AppResult};
use sea_orm::TransactionTrait;
use sea_orm::entity::prelude::*;

pub async fn list_schools(db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    Ok(school::Entity::find().all(db).await?)
}

pub async fn find_schools(query: &str, db: &impl ConnectionTrait) -> AppResult<Vec<school::Model>> {
    if query.len() < 2 {
        return list_schools(db).await;
    }
    let mut schools = list_schools(db)
        .await?
        .into_iter()
        .filter_map(|school| {
            let similarity = strsim::jaro_winkler(query, school.name.as_str());
            if similarity > 0.5 {
                Some((similarity, school))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // Most similar first.
    schools.sort_by(|(sim1, _), (sim2, _)| sim2.total_cmp(sim1));

    Ok(schools.into_iter().map(|(_, school)| school).collect())
}

/// Merge `other` into `into`: re-point every registration from `other` onto `into`, then
/// delete `other`. Both steps run in one transaction so a half-merged state is impossible.
pub async fn merge(
    into: &school::Model,
    other: &school::Model,
    db: &DatabaseConnection,
) -> AppResult<()> {
    let into_id = into.id;
    let other_id = other.id;
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            registration::Entity::update_many()
                .filter(registration::Column::SchoolId.eq(other_id))
                .col_expr(registration::Column::SchoolId, Expr::value(into_id))
                .exec(txn)
                .await?;

            school::Entity::delete_by_id(other_id).exec(txn).await?;
            Ok(())
        })
    })
    .await
    .map_err(AppError::from)
}

/*pub async fn stats(
    school: &school::Model,
    edition: &edition::Model,
    db: &impl ConnectionTrait,
) -> AppResult<Statistics> {
    let registrations = registration::Entity::find()
        .filter(registration::Column::EditionId.eq(edition.id))
        .filter(registration::Column::SchoolId.eq(school.id))
        .all(db)
        .await?;
    stats_for_registrations(edition, &registrations, db).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::registration::ReviewStatus;
    use crate::entity::round::RoundType;
    use crate::entity::round_participation::ParticipationState;
    use crate::services::scoring::set_task_points;
    use crate::utils::test_utils::tests::{
        TestApp, create_edition, create_participation, create_registration, create_round,
        create_school, create_student, create_task, date, setup_test,
    };
    use sea_orm::{ActiveValue::Set, IntoActiveModel};

    #[tokio::test]
    async fn merge_reparents_registrations_and_deletes_source() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let keep = create_school(db, "Lycée A").await;
        let drop = create_school(db, "Lycée B").await;

        let reg = create_registration(db, &edition, &drop, None).await;

        merge(&keep, &drop, db).await.unwrap();

        // Source school gone, target kept.
        assert!(
            school::Entity::find_by_id(drop.id)
                .one(db)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            school::Entity::find_by_id(keep.id)
                .one(db)
                .await
                .unwrap()
                .is_some()
        );

        // Registration re-pointed to the kept school.
        let reg = registration::Entity::find_by_id(reg.id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reg.school_id, keep.id);
    }

    /// A complete edition-2026 scenario for school A:
    /// - 3 registrations (s1, s2 linked to students; 1 unlinked + Validated)
    /// - Round 1 (day 5, Task A/100) and Round 2 (day 10, Task B/50)
    /// - s1 + s2 compete in Round 1; s1 is invited to Round 2; s2 refuses Round 2
    /// - scores: A: s1=80, s2=60 ; B: s1=40
    async fn edition_2026(app: &TestApp) -> (school::Model, edition::Model) {
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let school = create_school(db, "Lycée A").await;
        let s1 = create_student(db, "Ada", "T", date(10)).await;
        let s2 = create_student(db, "Grace", "T", date(11)).await;

        create_registration(db, &edition, &school, Some(s1.id)).await;
        create_registration(db, &edition, &school, Some(s2.id)).await;
        let unlinked = create_registration(db, &edition, &school, None)
            .await
            .into_active_model();
        let mut unlinked = unlinked;
        unlinked.review_status = Set(ReviewStatus::Validated);
        unlinked.update(db).await.unwrap();

        let round1 = create_round(
            db,
            edition.id,
            "Round 1",
            RoundType::Normal,
            date(5),
            date(6),
        )
        .await;
        let round2 = create_round(
            db,
            edition.id,
            "Round 2",
            RoundType::Normal,
            date(10),
            date(11),
        )
        .await;
        let task_a = create_task(db, round1.id, "Task A", 1, 100).await;
        let task_b = create_task(db, round2.id, "Task B", 1, 50).await;

        create_participation(db, &round1, &s1, ParticipationState::Accepted).await;
        create_participation(db, &round1, &s2, ParticipationState::Accepted).await;
        create_participation(db, &round2, &s1, ParticipationState::Invited).await;
        create_participation(db, &round2, &s2, ParticipationState::Refused).await;

        set_task_points(&round1, &task_a, &s1, 80, db)
            .await
            .unwrap();
        set_task_points(&round1, &task_a, &s2, 60, db)
            .await
            .unwrap();
        set_task_points(&round2, &task_b, &s1, 40, db)
            .await
            .unwrap();

        (school, edition)
    }

    #[tokio::test]
    async fn edition_stats_count_signups_validated_and_competitors() {
        let app = setup_test().await;
        let (school, edition) = edition_2026(&app).await;

        let stats = stats(&school, &edition, &app.state.db).await.unwrap();

        assert_eq!(stats.edition_id, edition.id);
        assert_eq!(stats.registrations, 3);
        assert_eq!(stats.validated, 1);
    }

    #[tokio::test]
    async fn edition_stats_round_distribution_and_points() {
        let app = setup_test().await;
        let (school, edition) = edition_2026(&app).await;

        let stats = stats(&school, &edition, &app.state.db).await.unwrap();

        assert_eq!(stats.rounds.len(), 2);

        let r1 = &stats.rounds[0]; // chronological order
        assert_eq!(r1.round_id, 1);
        assert_eq!(r1.participants, 2);
        assert_eq!(r1.achievable_points, 100);
        assert_eq!(r1.minimal_points, 60);
        assert_eq!(r1.maximal_points, 80);

        let r2 = &stats.rounds[1];
        assert_eq!(r2.round_id, 2);
        assert_eq!(r2.participants, 1);
        assert_eq!(r2.achievable_points, 50);
        assert_eq!(r2.minimal_points, 40);
        assert_eq!(r2.maximal_points, 40);
    }

    #[tokio::test]
    async fn edition_stats_competitors_zero_when_not_linked_to_students() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let school = create_school(db, "Lycée A").await;
        create_registration(db, &edition, &school, None).await;
        create_round(
            db,
            edition.id,
            "Round 1",
            RoundType::Normal,
            date(5),
            date(6),
        )
        .await;

        let stats = stats(&school, &edition, db).await.unwrap();

        assert_eq!(stats.registrations, 1);
        assert_eq!(stats.validated, 0);
        assert_eq!(stats.rounds.len(), 1);
        assert_eq!(stats.rounds[0].participants, 0);
    }
}
*/
