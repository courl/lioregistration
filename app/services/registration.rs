use crate::entity::registration::{NewRegistration, ReviewStatus};
use crate::entity::{
    admin_user, credentials, edition, registration, round, round_participation, school, student,
    task_score,
};
use crate::error::{AppError, AppResult};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DatabaseTransaction, IntoActiveModel, TransactionTrait};

pub enum LinkBehaviour {
    CreateNew,
    Link(student::Model),
    Unlink,
}

/// Public sign-up: stores the registration, claims a free credential, and enqueues the
/// credentials email. The registration is the product; emailing is best-effort and can never
/// roll it back (see [`notify_registration`]).
pub async fn register_student(
    edition: &edition::Model,
    mut new: NewRegistration,
    db: &DatabaseConnection,
) -> AppResult<registration::Model> {
    let today = chrono::Local::now().naive_local().date();
    if !edition.is_registration_open(today) {
        return Err(AppError::Gone("registration closed".to_string()));
    }
    new.validate(today)?;
    let edition = edition.clone();
    db.transaction::<_, _, AppError>(|txn| {
        Box::pin(async move {
            let already_registered = registration::Entity::find()
                .filter(registration::COLUMN.edition_id.eq(edition.id))
                .filter(registration::COLUMN.email.eq(new.email.as_str()))
                .count(txn)
                .await?;
            if already_registered > 0 {
                return Err(AppError::Conflict(
                    "This email address is already registered for this edition".into(),
                ));
            }

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
                .into_active_model(edition.id, &school)
                .insert(txn)
                .await?;

            let credential = match credentials::Entity::find_free(edition.id).one(txn).await? {
                Some(credential) => {
                    let mut credential = credential.into_active_model();
                    credential.set_registration(&registration);
                    Some(credential.update(txn).await?)
                }
                None => None,
            };

            notify_registration(&edition, &registration, &school, credential, txn).await;
            Ok(registration)
        })
    })
    .await
    .map_err(AppError::from)
}

/// Enqueues the credentials email, falling back to an admin alert. Each attempt runs in its own
/// savepoint so a failure (missing template, no round yet, ...) is rolled back on its own and
/// only logged, leaving the surrounding registration intact.
async fn notify_registration(
    edition: &edition::Model,
    registration: &registration::Model,
    school: &school::Model,
    credential: Option<credentials::Model>,
    txn: &DatabaseTransaction,
) {
    let problem = if let Some(credential) = credential {
        let (edition, registration, school) =
            (edition.clone(), registration.clone(), school.clone());
        txn.transaction::<_, _, AppError>(|savepoint| {
            Box::pin(async move {
                super::email_generation::send_registration_email(
                    &edition,
                    &registration,
                    &school,
                    &credential,
                    savepoint,
                )
                .await
            })
        })
        .await
        .err()
        .map(|e| format!("**Email failed**: {}", AppError::from(e)))
    } else {
        Some("**Out of credentials**".to_owned())
    };

    let Some(problem) = problem else { return };
    let message = format!(
        "{problem}: unable to send credentials to {} {} (registration id=`{}`)",
        registration.given_name, registration.family_name, registration.id
    );
    tracing::error!("{message}");
    if let Err(e) = txn
        .transaction::<_, _, AppError>(|savepoint| {
            Box::pin(
                async move { super::email_generation::send_error_email(message, savepoint).await },
            )
        })
        .await
    {
        tracing::error!("Could not enqueue admin alert: {}", AppError::from(e));
    }
}

/// Review decision for a registration: link it to a (new or existing) student, or unlink it.
///
/// Rules: a student holds at most one registration per edition; a linked student is entered
/// into the edition's first round (which must exist); switching students away from an old one
/// removes the old student's participations in this edition, refused if scores are recorded.
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
            let first_review = registration.reviewed_at.is_none();
            let old_student = registration.student_id;

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

            let new_student = student.as_ref().map(|s| s.id);

            if let Some(student_id) = new_student {
                let linked_elsewhere = registration::Entity::find()
                    .filter(registration::COLUMN.edition_id.eq(edition_id))
                    .filter(registration::COLUMN.student_id.eq(student_id))
                    .filter(registration::COLUMN.id.ne(registration.id))
                    .count(txn)
                    .await?;
                if linked_elsewhere > 0 {
                    return Err(AppError::Conflict(
                        "Student is already linked to another registration in this edition".into(),
                    ));
                }
            }

            if let Some(old_student) = old_student
                && new_student != Some(old_student)
            {
                remove_participations(edition_id, old_student, txn).await?;
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
                let first_round = super::rounds::get_first_round(edition_id, txn).await?;
                let participating = round_participation::Entity::find_by_round_student((
                    first_round.id,
                    student.id,
                ))
                .one(txn)
                .await?;
                if participating.is_none() {
                    round_participation::ActiveModel::create(
                        &first_round,
                        &student,
                        round_participation::ParticipationState::Accepted,
                    )
                    .insert(txn)
                    .await?;
                }
            }
            Ok(())
        })
    })
    .await
    .map_err(AppError::from)
}

/// Deletes `student_id`'s participations in `edition_id`, refusing if any has recorded scores.
async fn remove_participations(
    edition_id: i32,
    student_id: i32,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let round_ids: Vec<i32> = round::Entity::find_by_edition(edition_id)
        .all(db)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let participation_ids: Vec<i32> = round_participation::Entity::find()
        .filter(round_participation::COLUMN.student_id.eq(student_id))
        .filter(round_participation::COLUMN.round_id.is_in(round_ids))
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.id)
        .collect();
    if participation_ids.is_empty() {
        return Ok(());
    }
    let scores = task_score::Entity::find()
        .filter(
            task_score::COLUMN
                .round_participation_id
                .is_in(participation_ids.clone()),
        )
        .count(db)
        .await?;
    if scores > 0 {
        return Err(AppError::Conflict(
            "The linked student already has scores in this edition; it cannot be unlinked".into(),
        ));
    }
    round_participation::Entity::delete_many()
        .filter(round_participation::COLUMN.id.is_in(participation_ids))
        .exec(db)
        .await?;
    Ok(())
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
    students.sort_by_key(|x| std::cmp::Reverse(x.0));
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

#[cfg(test)]
mod review_tests {
    use super::*;
    use crate::entity::email_template::TemplateType;
    use crate::entity::{job, round_participation::ParticipationState};
    use crate::utils::test_utils::tests::{
        create_credential, create_edition, create_email_template, create_participation,
        create_registration, create_round, create_school, create_score, create_student,
        create_task, date, sample_new_registration, setup_test, today,
    };
    use chrono::Duration;
    use sea_orm::DatabaseConnection;

    async fn open_edition(db: &DatabaseConnection) -> edition::Model {
        create_edition(
            db,
            2026,
            true,
            today() - Duration::days(5),
            today() + Duration::days(5),
        )
        .await
    }

    async fn reviewer(db: &DatabaseConnection) -> admin_user::Model {
        admin_user::ActiveModel::create("r@lio.lu", "pw", "Reviewer", admin_user::UserRole::Admin)
            .unwrap()
            .insert(db)
            .await
            .unwrap()
    }

    async fn reload(db: &DatabaseConnection, id: i32) -> registration::Model {
        registration::Entity::find_by_id(id)
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn register_enqueues_credentials_email_when_configured() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;
        create_round(db, edition.id, "Round 1", today(), today()).await;
        create_email_template(db, TemplateType::Registration).await;
        create_credential(db, "user01", "pw", &edition).await;

        register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap();

        let jobs = job::Entity::find().all(db).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].payload.contains("user01"));
    }

    #[tokio::test]
    async fn register_falls_back_to_admin_alert_when_email_cannot_be_built() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;
        // Credential available but no round and no registration template: the credentials
        // email can't be built, so an admin alert (Error template) is queued instead.
        create_email_template(db, TemplateType::Error).await;
        create_credential(db, "user01", "pw", &edition).await;

        let registration = register_student(&edition, sample_new_registration("Lycée A"), db)
            .await
            .unwrap();

        assert_eq!(reload(db, registration.id).await.id, registration.id);
        let jobs = job::Entity::find().all(db).await.unwrap();
        assert_eq!(jobs.len(), 1);
        assert!(jobs[0].payload.contains("Email failed"));
    }

    #[tokio::test]
    async fn register_validates_and_normalizes_input() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;

        let mut new = sample_new_registration("  Lycée A ");
        new.email = " Ada@Example.COM ".into();
        new.given_name = " Ada ".into();
        let registration = register_student(&edition, new, db).await.unwrap();
        assert_eq!(registration.email, "ada@example.com");
        assert_eq!(registration.given_name, "Ada");
        assert!(
            school::Entity::find_by_name("Lycée A")
                .one(db)
                .await
                .unwrap()
                .is_some()
        );

        // Same email again (case-insensitive) in the same edition.
        let mut again = sample_new_registration("Lycée A");
        again.email = "ADA@example.com".into();
        assert!(matches!(
            register_student(&edition, again, db).await,
            Err(AppError::Conflict(_))
        ));

        for broken in [
            |r: &mut NewRegistration| r.given_name = "   ".into(),
            |r: &mut NewRegistration| r.email = "not-an-email".into(),
            |r: &mut NewRegistration| r.phone = "1".repeat(500),
            |r: &mut NewRegistration| r.birthdate = today() + Duration::days(1),
        ] {
            let mut new = sample_new_registration("Lycée A");
            new.email = "other@example.com".into();
            broken(&mut new);
            assert!(matches!(
                register_student(&edition, new, db).await,
                Err(AppError::BadRequest(_))
            ));
        }
        assert_eq!(registration::Entity::find().count(db).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn link_requires_a_round_and_enters_the_first_round() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;
        let school = create_school(db, "Lycée A").await;
        let reviewer = reviewer(db).await;
        let registration = create_registration(db, &edition, &school, None).await;

        let err = link_registration(
            &edition,
            registration.clone(),
            &reviewer,
            LinkBehaviour::CreateNew,
            db,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)), "{err:?}");
        assert_eq!(
            student::Entity::find().count(db).await.unwrap(),
            0,
            "rolled back"
        );

        let round1 = create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        create_round(db, edition.id, "Round 2", date(10), date(11)).await;
        link_registration(
            &edition,
            registration.clone(),
            &reviewer,
            LinkBehaviour::CreateNew,
            db,
        )
        .await
        .unwrap();

        let registration = reload(db, registration.id).await;
        assert_eq!(registration.review_status, ReviewStatus::Validated);
        let student_id = registration.student_id.unwrap();
        let participations = round_participation::Entity::find().all(db).await.unwrap();
        assert_eq!(participations.len(), 1);
        assert_eq!(participations[0].round_id, round1.id);
        assert_eq!(participations[0].student_id, student_id);

        // Re-linking the same student is idempotent.
        let student = student::Entity::find_by_id(student_id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        link_registration(
            &edition,
            registration,
            &reviewer,
            LinkBehaviour::Link(student),
            db,
        )
        .await
        .unwrap();
        assert_eq!(
            round_participation::Entity::find().count(db).await.unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn student_is_linked_at_most_once_per_edition() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;
        create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        let school = create_school(db, "Lycée A").await;
        let reviewer = reviewer(db).await;
        let student = create_student(db, "Ada", "Lovelace", date(2)).await;
        create_registration(db, &edition, &school, Some(student.id)).await;
        let second = create_registration(db, &edition, &school, None).await;

        let err = link_registration(
            &edition,
            second,
            &reviewer,
            LinkBehaviour::Link(student),
            db,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn unlink_is_refused_once_scores_exist() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = open_edition(db).await;
        let round = create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        let task = create_task(db, round.id, "A", 1, 10).await;
        let school = create_school(db, "Lycée A").await;
        let reviewer = reviewer(db).await;
        let student = create_student(db, "Ada", "Lovelace", date(2)).await;
        let registration = create_registration(db, &edition, &school, Some(student.id)).await;
        let participation =
            create_participation(db, &round, &student, ParticipationState::Accepted).await;

        // Without scores, unlinking removes the participation.
        link_registration(
            &edition,
            registration.clone(),
            &reviewer,
            LinkBehaviour::Unlink,
            db,
        )
        .await
        .unwrap();
        assert_eq!(
            round_participation::Entity::find().count(db).await.unwrap(),
            0
        );
        assert_eq!(reload(db, registration.id).await.student_id, None);

        // Re-link, score, then unlinking must be refused.
        let registration = reload(db, registration.id).await;
        link_registration(
            &edition,
            registration.clone(),
            &reviewer,
            LinkBehaviour::Link(student),
            db,
        )
        .await
        .unwrap();
        let participation = round_participation::Entity::find_by_round_student((
            round.id,
            participation.student_id,
        ))
        .one(db)
        .await
        .unwrap()
        .unwrap();
        create_score(db, &task, &participation, 5.0).await;
        let registration = reload(db, registration.id).await;
        let err = link_registration(&edition, registration, &reviewer, LinkBehaviour::Unlink, db)
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)), "{err:?}");
    }
}
