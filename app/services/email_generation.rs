use crate::entity::email_template::TemplateType;
use crate::entity::{
    credentials, email_attachment_template, email_template, registration, round, school,
};
use crate::error::{AppError, AppResult};
use crate::jobs::Job;
use crate::jobs::send_email::SendEmailPayload;
use crate::services::email_attachment::EmailAttachmentContext;
use crate::utils::email_template::{
    EmailTemplate, GlobalContext, InstanceContext, apply_global_ctx, apply_instance_ctx,
};
use crate::{jobs, services};
use base64::Engine;
use chrono::Local;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize)]
pub struct AttachmentPreview {
    name: String,
    data_base64: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct EmailPreview {
    html: Option<String>,
    attachments: Vec<AttachmentPreview>,
    error: Option<String>,
}

async fn create_attachment_preview(
    attachments: &Vec<email_attachment_template::Model>,
    global_ctx: &GlobalContext,
    instance_ctx: &InstanceContext,
    email_attachment_ctx: &Arc<EmailAttachmentContext>,
) -> Vec<AttachmentPreview> {
    let mut result = Vec::new();
    for attachment in attachments {
        let content = apply_instance_ctx(
            apply_global_ctx(attachment.template.as_str(), global_ctx).as_str(),
            instance_ctx,
        );
        let data =
            super::email_attachment::compile_typst_to_pdf(content.as_str(), &email_attachment_ctx);
        result.push(match data {
            Ok(bytes) => AttachmentPreview {
                name: attachment.name.clone(),
                data_base64: Some(base64::prelude::BASE64_STANDARD.encode(bytes)),
                error: None,
            },
            Err(error) => AttachmentPreview {
                name: attachment.name.clone(),
                data_base64: None,
                error: Some(error.to_string()),
            },
        });
    }

    result
}

pub async fn preview_email(
    template: &email_template::Model,
    attachments: &Vec<email_attachment_template::Model>,
    body_markdown: &str,
    global_ctx: &GlobalContext,
    instance_ctx: &InstanceContext,
    email_attachment_ctx: &Arc<EmailAttachmentContext>,
) -> EmailPreview {
    match EmailTemplate::from_template(template.template.as_str(), global_ctx, body_markdown) {
        Ok(template) => EmailPreview {
            html: Some(template.instantiate(instance_ctx)),
            attachments: create_attachment_preview(
                attachments,
                global_ctx,
                instance_ctx,
                email_attachment_ctx,
            )
            .await,
            error: None,
        },
        Err(e) => EmailPreview {
            html: None,
            attachments: Vec::new(),
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
