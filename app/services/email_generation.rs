use crate::entity::email_template::TemplateType;
use crate::entity::{credentials, email_template, registration, round, school};
use crate::error::{AppError, AppResult};
use crate::jobs;
use crate::jobs::Job;
use crate::jobs::send_email::SendEmailPayload;
use crate::utils::email_template::{EmailTemplate, GlobalContext, InstanceContext};
use chrono::Local;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde::Serialize;

#[derive(Serialize)]
pub struct EmailPreview {
    html: Option<String>,
    error: Option<String>,
}

pub async fn preview_email(
    template: &email_template::Model,
    body_markdown: &str,
    global_ctx: &GlobalContext,
    instance_ctx: &InstanceContext,
) -> EmailPreview {
    match EmailTemplate::from_template(template.template.as_str(), global_ctx, body_markdown) {
        Ok(template) => EmailPreview {
            html: Some(template.instantiate(instance_ctx)),
            error: None,
        },
        Err(e) => EmailPreview {
            html: None,
            error: Some(e.to_string()),
        },
    }
}

async fn load_template(
    template_type: TemplateType,
    db: &impl ConnectionTrait,
) -> AppResult<email_template::Model> {
    email_template::Entity::find()
        .filter(email_template::COLUMN.template_type.eq(template_type))
        .one(db)
        .await?
        .ok_or(AppError::NotFound(
            "requested email template does not exist".into(),
        ))
}

pub async fn send_registration_email(
    registration: &registration::Model,
    school: &school::Model,
    credentials: &credentials::Model,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let edition = super::editions::active_edition(db).await?;
    let first_round = super::rounds::get_first_round(edition.id, db).await?;
    let template = load_template(TemplateType::Registration, db).await?;

    let global_ctx = GlobalContext {
        headline: "Registration".into(),
        edition: edition.name,
        round: Some(first_round.name),
        round_starts_at: first_round.starts_at,
        round_ends_at: first_round.ends_at,
        students: None,
    };

    let instance_ctx = InstanceContext {
        given_name: registration.given_name.clone(),
        family_name: registration.family_name.clone(),
        school: school.name.clone(),
        grade: registration.grade.clone(),
        credentials_username: Some(credentials.username.clone()),
        credentials_password: Some(credentials.password.clone()),
    };

    jobs::enqueue(
        Job::SendEmail(SendEmailPayload {
            template_id: template.id,
            global_context: global_ctx,
            body_markdown: String::new(),
            instances: Some(vec![(registration.email.clone(), instance_ctx)]),
        }),
        db,
    )
    .await?;

    Ok(())
}

pub async fn send_round_email(
    template: &email_template::Model,
    round: &round::Model,
    headline: String,
    body: String,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let details = super::rounds::details(round, db).await?;
    let edition = super::editions::get_edition(round.edition_id, db).await?;
    let registrations = super::student::load_registrations(
        edition.id,
        details.participants.iter().map(|x| &x.student).collect(),
        db,
    )
    .await?;
    let schools = school::Entity::find()
        .filter(
            school::COLUMN.id.is_in(
                registrations
                    .iter()
                    .map(|r| r.1.school_id)
                    .collect::<Vec<i32>>(),
            ),
        )
        .all(db)
        .await?;

    let global_ctx = GlobalContext {
        headline,
        edition: edition.name,
        round: Some(round.name.clone()),
        round_starts_at: round.starts_at,
        round_ends_at: round.ends_at,
        students: Some(
            details
                .participants
                .iter()
                .map(|participant| {
                    (
                        participant.student.given_name.clone()
                            + " "
                            + participant.student.family_name.as_str(),
                        participant.scores.iter().map(|x| x.points).sum(),
                    )
                })
                .collect(),
        ),
    };

    let instances = details
        .participants
        .into_iter()
        .map(|participant| {
            let registration = registrations.get(&participant.student.id).ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Database corruption: cannot find registraiton for student"
                ))
            })?;
            let school = schools
                .iter()
                .find(|s| s.id == registration.id)
                .ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "Database corruption: cannot find school for registration"
                    ))
                })?;
            Ok((
                participant.student.email,
                InstanceContext {
                    given_name: participant.student.given_name.clone(),
                    family_name: participant.student.family_name.clone(),
                    school: school.name.clone(),
                    grade: registration.grade.clone(),
                    credentials_username: None,
                    credentials_password: None,
                },
            ))
        })
        .collect::<AppResult<_>>()?;

    jobs::enqueue(
        Job::SendEmail(SendEmailPayload {
            template_id: template.id,
            global_context: global_ctx,
            body_markdown: body,
            instances: Some(instances),
        }),
        db,
    )
    .await?;

    Ok(())
}

pub async fn send_error_email(
    content_markdown: String,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    // fixme: what if loading fails, need to enqueue a dumb version
    let template = load_template(TemplateType::Error, db).await?;
    let today = Local::now().date_naive();
    let global_ctx = GlobalContext {
        headline: "Error".into(),
        edition: String::new(),
        round: None,
        round_starts_at: today,
        round_ends_at: today,
        students: None,
    };

    jobs::enqueue(
        Job::SendEmail(SendEmailPayload {
            template_id: template.id,
            global_context: global_ctx,
            body_markdown: content_markdown,
            instances: None,
        }),
        db,
    )
    .await?;

    Ok(())
}
