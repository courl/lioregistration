use crate::entity::email_template::TemplateType;
use crate::entity::{
    credentials, edition, email_attachment_template, email_template, registration, round, school,
};
use crate::error::{AppError, AppResult};
use crate::jobs;
use crate::jobs::Job;
use crate::jobs::send_email::SendEmailPayload;
use crate::services::email_attachment::EmailAttachmentContext;
use crate::utils::email_template::{
    CustomValue, EmailTemplate, GlobalContext, InstanceContext, apply_global_ctx_typst,
    apply_instance_ctx_typst, render_subject,
};
use base64::Engine;
use chrono::Local;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, QueryFilter};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Serialize)]
pub struct AttachmentPreview {
    name: String,
    data_base64: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct EmailPreview {
    subject: String,
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
        let content = apply_instance_ctx_typst(
            apply_global_ctx_typst(attachment.template.as_str(), global_ctx).as_str(),
            instance_ctx,
        );
        let data =
            super::email_attachment::compile_typst_to_pdf(content.as_str(), email_attachment_ctx);
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

async fn preview_email(
    template: &email_template::Model,
    attachments: &Vec<email_attachment_template::Model>,
    global_ctx: &GlobalContext,
    instance_ctx: &InstanceContext,
    email_attachment_ctx: &Arc<EmailAttachmentContext>,
) -> EmailPreview {
    let subject = render_subject(&template.subject, global_ctx, instance_ctx);
    match EmailTemplate::from_template(template.template.as_str(), global_ctx) {
        Ok(template) => EmailPreview {
            subject,
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
            subject,
            html: None,
            attachments: Vec::new(),
            error: Some(e.to_string()),
        },
    }
}

async fn load_template_with_attachments(
    id: i32,
    db: &impl ConnectionTrait,
) -> AppResult<(email_template::Model, Vec<email_attachment_template::Model>)> {
    let template = email_template::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Template does not exist".into()))?;
    let attachments = template
        .find_related(email_attachment_template::Entity)
        .all(db)
        .await?;
    Ok((template, attachments))
}

/// Renders a template with made-up contexts (template editor); `custom` holds the test
/// values of its fields.
pub async fn preview_test_email(
    template_id: i32,
    mut global_ctx: GlobalContext,
    instance_ctx: &InstanceContext,
    custom: BTreeMap<String, String>,
    email_attachment_ctx: &Arc<EmailAttachmentContext>,
    db: &impl ConnectionTrait,
) -> AppResult<EmailPreview> {
    let (template, attachments) = load_template_with_attachments(template_id, db).await?;
    global_ctx.custom = email_template::resolve_values(&template.custom_fields, custom)?;
    Ok(preview_email(
        &template,
        &attachments,
        &global_ctx,
        instance_ctx,
        email_attachment_ctx,
    )
    .await)
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
    edition: &edition::Model,
    registration: &registration::Model,
    school: &school::Model,
    credentials: &credentials::Model,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let first_round = super::rounds::get_first_round(edition.id, db).await?;
    let template = load_template(TemplateType::Registration, db).await?;

    // Nobody fills fields for an automatic email: declared ones take their defaults.
    let custom = email_template::resolve_values(&template.custom_fields, BTreeMap::new())?;
    let global_ctx = GlobalContext {
        edition: edition.name.clone(),
        round: Some(first_round.name),
        round_starts_at: first_round.starts_at,
        round_ends_at: first_round.ends_at,
        students: None,
        custom,
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
            instances: Some(vec![(registration.email.clone(), instance_ctx)]),
        }),
        db,
    )
    .await?;

    Ok(())
}

/// One recipient of a round email.
pub struct RoundRecipient {
    pub student_id: i32,
    pub email: String,
    pub context: InstanceContext,
}

/// The shared context and the per-participant contexts of an email to a round's
/// participants. Custom field values are checked against the template's declaration.
async fn build_round_contexts(
    template: &email_template::Model,
    round: &round::Model,
    custom: BTreeMap<String, String>,
    db: &impl ConnectionTrait,
) -> AppResult<(GlobalContext, Vec<RoundRecipient>)> {
    let custom = email_template::resolve_values(&template.custom_fields, custom)?;
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
        custom,
    };

    let recipients = details
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
                .find(|s| s.id == registration.school_id)
                .ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "Database corruption: cannot find school for registration"
                    ))
                })?;
            Ok(RoundRecipient {
                student_id: participant.student.id,
                // The address given for this edition; the student's may be years old.
                email: registration.email.clone(),
                context: InstanceContext {
                    given_name: participant.student.given_name,
                    family_name: participant.student.family_name,
                    school: school.name.clone(),
                    grade: registration.grade.clone(),
                    credentials_username: None,
                    credentials_password: None,
                },
            })
        })
        .collect::<AppResult<_>>()?;

    Ok((global_ctx, recipients))
}

pub async fn send_round_email(
    template: &email_template::Model,
    round: &round::Model,
    custom: BTreeMap<String, String>,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    let (global_ctx, recipients) = build_round_contexts(template, round, custom, db).await?;

    jobs::enqueue(
        Job::SendEmail(SendEmailPayload {
            template_id: template.id,
            global_context: global_ctx,
            instances: Some(
                recipients
                    .into_iter()
                    .map(|r| (r.email, r.context))
                    .collect(),
            ),
        }),
        db,
    )
    .await?;

    Ok(())
}

#[derive(Serialize)]
pub struct PreviewRecipient {
    student_id: i32,
    name: String,
}

#[derive(Serialize)]
pub struct RoundEmailPreview {
    #[serde(flatten)]
    preview: EmailPreview,
    /// Who the preview was rendered for.
    student_id: i32,
    /// Everyone the email would go to, for picking another preview recipient.
    recipients: Vec<PreviewRecipient>,
}

/// Renders the email (and attachments) exactly as `student_id` — or the first participant —
/// would receive it from `send_round_email` with the same input.
pub async fn preview_round_email(
    template_id: i32,
    round: &round::Model,
    student_id: Option<i32>,
    custom: BTreeMap<String, String>,
    email_attachment_ctx: &Arc<EmailAttachmentContext>,
    db: &impl ConnectionTrait,
) -> AppResult<RoundEmailPreview> {
    let (template, attachments) = load_template_with_attachments(template_id, db).await?;
    let (global_ctx, recipients) = build_round_contexts(&template, round, custom, db).await?;
    let recipient = match student_id {
        Some(id) => recipients
            .iter()
            .find(|r| r.student_id == id)
            .ok_or_else(|| {
                AppError::NotFound("Student is not a participant of this round".into())
            })?,
        None => recipients
            .first()
            .ok_or_else(|| AppError::Conflict("The round has no participants to preview".into()))?,
    };

    let preview = preview_email(
        &template,
        &attachments,
        &global_ctx,
        &recipient.context,
        email_attachment_ctx,
    )
    .await;

    Ok(RoundEmailPreview {
        preview,
        student_id: recipient.student_id,
        recipients: recipients
            .iter()
            .map(|r| PreviewRecipient {
                student_id: r.student_id,
                name: format!("{} {}", r.context.given_name, r.context.family_name),
            })
            .collect(),
    })
}

pub async fn send_error_email(
    content_markdown: String,
    db: &impl ConnectionTrait,
) -> AppResult<()> {
    // fixme: what if loading fails, need to enqueue a dumb version
    let template = load_template(TemplateType::Error, db).await?;
    let today = Local::now().date_naive();
    // The Error template shows these as `{{?headline}}` and `{{?content}}`.
    let global_ctx = GlobalContext {
        edition: String::new(),
        round: None,
        round_starts_at: today,
        round_ends_at: today,
        students: None,
        custom: BTreeMap::from([
            ("headline".into(), CustomValue::Text("Error".into())),
            ("content".into(), CustomValue::Markdown(content_markdown)),
        ]),
    };

    jobs::enqueue(
        Job::SendEmail(SendEmailPayload {
            template_id: template.id,
            global_context: global_ctx,
            instances: None,
        }),
        db,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::round_participation::ParticipationState;
    use crate::utils::test_utils::tests::{
        create_edition, create_email_template, create_participation, create_registration,
        create_round, create_school, create_student, date, setup_test,
    };
    use sea_orm::{ActiveModelTrait, ActiveValue::Set, IntoActiveModel};

    #[tokio::test]
    async fn round_email_goes_to_the_editions_registration_address() {
        let app = setup_test().await;
        let db = &app.state.db;
        let edition = create_edition(db, 2026, true, date(1), date(28)).await;
        let round = create_round(db, edition.id, "Round 1", date(5), date(6)).await;
        let school = create_school(db, "Lycée A").await;
        let student = create_student(db, "Ada", "T", date(10)).await;
        let mut registration = create_registration(db, &edition, &school, Some(student.id))
            .await
            .into_active_model();
        registration.email = Set("new@lio.lu".into());
        registration.update(db).await.unwrap();
        create_participation(db, &round, &student, ParticipationState::Accepted).await;
        let template = create_email_template(db, TemplateType::Round).await;

        let (_, recipients) = build_round_contexts(&template, &round, BTreeMap::new(), db)
            .await
            .unwrap();
        assert_eq!(recipients[0].email, "new@lio.lu");
        assert_ne!(student.email, "new@lio.lu");
    }
}
