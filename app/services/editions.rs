use crate::entity::registration::ReviewStatus;
use crate::entity::{edition, registration, round};
use crate::error::{AppError, AppResult};
use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct RegistrationsForDay {
    pub day: ChronoDate,
    pub new: u32,
    pub reviewed: u32,
}

#[derive(Serialize)]
pub struct EditionStatistics {
    pub rounds: u32,
    pub students: u32,
    pub registrations: Vec<RegistrationsForDay>,
}

pub async fn get_edition(id: i32, db: &impl ConnectionTrait) -> AppResult<edition::Model> {
    edition::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or(AppError::NotFound("Edition does not exist".into()))
}

/// Loads an edition for a mutation, refusing it unless it is the active one.
pub async fn get_editable_edition(id: i32, db: &impl ConnectionTrait) -> AppResult<edition::Model> {
    let edition = get_edition(id, db).await?;
    edition.ensure_editable()?;
    Ok(edition)
}

pub async fn active_edition(db: &impl ConnectionTrait) -> AppResult<edition::Model> {
    edition::Entity::find()
        .filter(edition::Column::Active.eq(true))
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("No active edition".to_string()))
}

pub async fn set_active(edition: &edition::Model, db: &DatabaseConnection) -> AppResult<()> {
    let id = edition.id;
    db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            edition::Entity::update_many()
                .col_expr(edition::Column::Active, Expr::value(false))
                .exec(txn)
                .await?;

            edition::Entity::update_many()
                .col_expr(edition::Column::Active, Expr::value(true))
                .filter(edition::Column::Id.eq(id))
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await
    .map_err(AppError::from)
}

pub async fn statistics(
    edition: &edition::Model,
    db: &impl ConnectionTrait,
) -> AppResult<EditionStatistics> {
    let rounds = edition.find_related(round::Entity).count(db).await? as u32;
    let registrations = edition.find_related(registration::Entity).all(db).await?;
    let students = registrations
        .iter()
        .filter(|x| x.student_id.is_some() && x.review_status == ReviewStatus::Validated)
        .count() as u32;

    let mut registrations_per_day = HashMap::<ChronoDate, _>::new();
    for registration in registrations {
        registrations_per_day
            .entry(registration.registered_at.date_naive())
            .or_insert_with(|| RegistrationsForDay {
                day: registration.registered_at.date_naive(),
                new: 0,
                reviewed: 0,
            })
            .new += 1;
        if let Some(date) = registration.reviewed_at {
            registrations_per_day
                .entry(date.date_naive())
                .or_insert_with(|| RegistrationsForDay {
                    day: date.date_naive(),
                    new: 0,
                    reviewed: 0,
                })
                .reviewed += 1;
        }
    }

    let mut registrations: Vec<RegistrationsForDay> = registrations_per_day.into_values().collect();
    registrations.sort_by_key(|day| day.day);
    Ok(EditionStatistics {
        rounds,
        students,
        registrations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::registration::ReviewStatus;
    use crate::utils::test_utils::tests::{
        create_edition, create_registration, create_round, create_school, create_student, date,
        setup_test,
    };
    use sea_orm::{ActiveValue::Set, IntoActiveModel};

    #[tokio::test]
    async fn totals_and_funnel_aggregate_across_schools() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let school_a = create_school(db, "Lycée A").await;
        let school_b = create_school(db, "Lycée B").await;
        let s1 = create_student(db, "Ada", "T", date(10)).await;
        let s2 = create_student(db, "Grace", "T", date(11)).await;
        let s3 = create_student(db, "Alan", "T", date(12)).await; // school B

        create_registration(db, &edition, &school_a, Some(s1.id)).await;
        create_registration(db, &edition, &school_a, Some(s2.id)).await;
        create_registration(db, &edition, &school_b, Some(s3.id)).await;
        // One unlinked sign-up, marked Validated.
        let mut unlinked = create_registration(db, &edition, &school_a, None)
            .await
            .into_active_model();
        unlinked.review_status = Set(ReviewStatus::Validated);
        unlinked.update(db).await.unwrap();

        let mut refused = create_registration(db, &edition, &school_a, None)
            .await
            .into_active_model();
        refused.review_status = Set(ReviewStatus::Refused);
        refused.update(db).await.unwrap();

        let mut pending = create_registration(db, &edition, &school_a, None)
            .await
            .into_active_model();
        pending.review_status = Set(ReviewStatus::Pending);
        pending.update(db).await.unwrap();

        create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        create_round(db, edition.id, "Round 2", date(10), date(11)).await;

        let stats = statistics(&edition, db).await.unwrap();

        assert_eq!(stats.registrations.len(), 1);
        assert_eq!(stats.registrations[0].new, 6);
        assert_eq!(stats.rounds, 2);
        assert_eq!(stats.students, 3);
    }

    #[tokio::test]
    async fn empty_edition_reports_zeros() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;

        let stats = statistics(&edition, db).await.unwrap();
        assert_eq!(stats.registrations.len(), 0);
        assert_eq!(stats.students, 0);
        assert_eq!(stats.rounds, 0);
    }
}
