#![allow(dead_code)]

use crate::entity::registration::{NewRegistration, ReviewStatus};
use crate::entity::{
    admin_user, credentials, edition, registration, round, round_participation, school, student,
};
use crate::error::{AppError, AppResult};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TransactionTrait};

pub enum LinkBehaviour {
    CreateNew,
    Link(student::Model),
    Unlink,
}

pub async fn register_student(
    edition: &edition::Model,
    new: NewRegistration,
    db: &DatabaseConnection,
) -> AppResult<registration::Model> {
    let today = chrono::Local::now().naive_local().date();
    if !edition.is_registration_open(today) {
        return Err(AppError::Gone("registration closed".to_string()));
    }
    let edition_id = edition.id;
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            let school = match school::Entity::find_by_name(&new.school_name)
                .one(txn)
                .await?
            {
                Some(school) => school,
                None => {
                    school::ActiveModel::create(&new.school_name)
                        .insert(txn)
                        .await?
                }
            };

            let registration = new
                .into_active_model(edition_id, &school)
                .insert(txn)
                .await?;

            if let Some(credential) = credentials::Entity::find_free(edition_id).one(txn).await? {
                let mut credential = credential.into_active_model();
                credential.set_registration(&registration);
                let credential = credential.update(txn).await?;

                super::email_generation::send_registration_email(
                    &registration,
                    &school,
                    &credential,
                    txn,
                )
                .await?;
            } else {
                super::email_generation::send_error_email(
                    format!(
                        "**Out of credentials**: Unable to send credentials to {} {} (id=`{}`)",
                        registration.given_name, registration.family_name, registration.id
                    ),
                    txn,
                )
                .await?;
            }

            Ok(registration)
        })
    })
    .await
    .map_err(AppError::from)
}

pub async fn link_registration(
    edition: &edition::Model,
    registration: registration::Model,
    reviewer: &admin_user::Model,
    behaviour: LinkBehaviour,
    db: &DatabaseConnection,
) -> AppResult<()> {
    let edition_id = edition.id;
    let reviewer_id = reviewer.id;
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            let first_round = super::rounds::get_first_round(edition_id, txn).await?;
            let first_review = registration.reviewed_at.is_none();

            let student = match behaviour {
                LinkBehaviour::CreateNew => Some(
                    student::ActiveModel::create(
                        registration.given_name.clone(),
                        registration.family_name.clone(),
                        registration.email.clone(),
                        registration.birthdate,
                    )
                    .insert(txn)
                    .await?,
                ),
                LinkBehaviour::Link(student) => Some(student),
                LinkBehaviour::Unlink => None,
            };

            if let Some(old_student) = registration.student_id {
                let round_ids = round::Entity::find_by_edition(edition_id)
                    .all(txn)
                    .await?
                    .into_iter()
                    .map(|x| x.id)
                    .collect::<Vec<_>>();
                round_participation::Entity::delete_many()
                    .filter(round_participation::COLUMN.student_id.eq(old_student))
                    .filter(round_participation::COLUMN.round_id.is_in(round_ids))
                    .exec(txn)
                    .await?;
            }

            let mut registration = registration.into_active_model();
            registration.student_id = ActiveValue::Set(student.as_ref().map(|s| s.id));
            registration.review_status = ActiveValue::Set(if student.is_some() {
                ReviewStatus::Validated
            } else {
                ReviewStatus::Pending
            });
            registration.reviewed_by_id = ActiveValue::Set(Some(reviewer_id));
            if first_review {
                registration.reviewed_at = ActiveValue::Set(chrono::Utc::now().into());
            }
            registration.save(txn).await?;

            if let Some(student) = student {
                round_participation::ActiveModel::create(
                    &first_round,
                    &student,
                    round_participation::ParticipationState::Accepted,
                )
                .insert(txn)
                .await?;
            }
            Ok(())
        })
    })
    .await
    .map_err(AppError::from)
}

pub fn calculate_match(registration: &registration::Model, candidate: &student::Model) -> u32 {
    if candidate.email == registration.email {
        return 100;
    }
    let mut score = 0f64;
    if candidate.birthdate == registration.birthdate {
        score += 1f64;
    }
    score += strsim::jaro_winkler(&candidate.family_name, &registration.family_name);
    score += strsim::jaro_winkler(&candidate.given_name, &registration.given_name);

    score /= 3.0;
    (score * 100.0).round() as u32
}

pub async fn student_candidates(
    registration: &registration::Model,
    db: &impl ConnectionTrait,
) -> AppResult<Vec<(u32, student::Model)>> {
    // fixme: should we not show only those students that have not another registration yet?
    let mut students = student::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .filter_map(|candidate| {
            let score = calculate_match(registration, &candidate);
            if score < 70 {
                None
            } else {
                Some((score, candidate))
            }
        })
        .collect::<Vec<_>>();
    students.sort_by_key(|x| x.0);
    students.reverse(); // is there no sort by descending?
    Ok(students)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::tests::{
        create_credential, create_edition, create_school, sample_new_registration, setup_test,
        today,
    };
    use chrono::Duration;

    #[tokio::test]
    async fn register_persists_registration_and_claims_credential() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(
            db,
            2026,
            true,
            today() - Duration::days(5),
            today() + Duration::days(5),
        )
        .await;
        let credential = create_credential(db, "user01", "pw", &edition).await;

        let registration = register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap();

        // Registration stored as pending and linked to the edition.
        assert_eq!(registration.edition_id, edition.id);
        assert_eq!(
            registration.review_status,
            registration::ReviewStatus::Pending
        );

        // School was created and reused via the FK.
        let school = school::Entity::find_by_name("Lycée A")
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(registration.school_id, school.id);

        // The free credential is now claimed for this registration.
        let credential = credentials::Entity::find_by_id(credential.id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(credential.registration_id, Some(registration.id));
    }

    #[tokio::test]
    async fn register_outside_window_is_gone() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(
            db,
            2026,
            true,
            today() - Duration::days(10),
            today() - Duration::days(5),
        )
        .await;

        let err = register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Gone(_)));

        // Nothing was written.
        assert_eq!(registration::Entity::find().all(db).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn register_reuses_existing_school() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(
            db,
            2026,
            true,
            today() - Duration::days(5),
            today() + Duration::days(5),
        )
        .await;
        let existing = create_school(db, "Lycée A").await;

        let registration = register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap();

        assert_eq!(registration.school_id, existing.id);
        assert_eq!(school::Entity::find().all(db).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn register_without_free_credentials_still_succeeds() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(
            db,
            2026,
            true,
            today() - Duration::days(5),
            today() + Duration::days(5),
        )
        .await;

        let registration = register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap();

        assert_eq!(registration.student_id, None);
        // No credential rows exist, so none could be claimed — but the registration stands.
        assert_eq!(credentials::Entity::find().all(db).await.unwrap().len(), 0);
    }
}
