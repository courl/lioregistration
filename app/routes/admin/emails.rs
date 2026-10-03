use crate::entity::admin_user::UserRole;
use crate::entity::email_template::TemplateType;
use crate::entity::{email_attachment_template, email_template, round};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::services;
use crate::state::AppState;
use crate::utils::email_template::{GlobalContext, InstanceContext};
use axum::Json;
use axum::extract::{Path, State};
use axum_extra::extract::Query;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait,
    QueryFilter,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct EmailTemplate {
    id: i32,
    name: String,
    template_type: Option<TemplateType>,
}

#[derive(Serialize)]
pub struct EmailTemplateDetails {
    #[serde(flatten)]
    email: EmailTemplate,
    template: String,
}

#[derive(Deserialize)]
pub struct TemplatesFilter {
    #[serde(default)]
    types: Vec<TemplateType>,
}

#[derive(Deserialize)]
pub struct PreviewRequest {
    body: String,
    global_context: GlobalContext,
    instance_context: InstanceContext,
}

#[derive(Deserialize)]
pub struct UpdateTemplateRequest {
    name: String,
    template: String,
    template_type: Option<TemplateType>,
}

#[derive(Deserialize)]
pub struct SendEmailRequest {
    round_id: i32,
    headline: String,
    body: String,
}

#[derive(Deserialize)]
pub struct UpdateAttachmentRequest {
    name: String,
    template: String,
}

#[derive(Serialize)]
pub struct EmailAttachment {
    id: i32,
    name: String,
    template: String,
}

impl From<email_template::Model> for EmailTemplate {
    fn from(model: email_template::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            template_type: model.template_type,
        }
    }
}

impl From<email_template::Model> for EmailTemplateDetails {
    fn from(model: email_template::Model) -> Self {
        let template = model.template.clone();
        Self {
            email: model.into(),
            template,
        }
    }
}

impl From<email_attachment_template::Model> for EmailAttachment {
    fn from(model: email_attachment_template::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            template: model.template,
        }
    }
}

pub async fn get_templates(
    State(state): State<AppState>,
    Query(filter): Query<TemplatesFilter>,
    auth: Auth,
) -> AppResult<Json<Vec<EmailTemplate>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let emails = if !filter.types.is_empty() {
        email_template::Entity::find()
            .filter(email_template::COLUMN.template_type.is_in(filter.types))
            .all(&state.db)
            .await?
    } else {
        email_template::Entity::find().all(&state.db).await?
    };
    Ok(Json(emails.into_iter().map(Into::into).collect()))
}

pub async fn create_template(
    State(state): State<AppState>,
    auth: Auth,
) -> AppResult<Json<EmailTemplate>> {
    auth.verify(vec![UserRole::Admin])?;
    Ok(Json(
        email_template::ActiveModel::create(
            "New template".into(),
            "<mjml></mjml>".into(),
            TemplateType::Round,
        )
        .insert(&state.db)
        .await?
        .into(),
    ))
}

pub async fn get_template(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<Json<EmailTemplateDetails>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        email_template::Entity::find_by_id(id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Template does not exist".into()))?
            .into(),
    ))
}

pub async fn update_template(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
    Json(request): Json<UpdateTemplateRequest>,
) -> AppResult<Json<EmailTemplateDetails>> {
    auth.verify(vec![UserRole::Admin])?;
    let mut template = email_template::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Template does not exist".into()))?
        .into_active_model();

    template.name = ActiveValue::Set(request.name);
    template.template = ActiveValue::Set(request.template);
    template.template_type = ActiveValue::Set(request.template_type);

    Ok(Json(template.update(&state.db).await?.into()))
}

pub async fn preview_template(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
    Json(request): Json<PreviewRequest>,
) -> AppResult<Json<services::email_generation::EmailPreview>> {
    auth.verify(vec![UserRole::Admin])?;
    let template = email_template::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Template does not exist".into()))?;

    let attachments = template
        .find_related(email_attachment_template::Entity)
        .all(&state.db)
        .await?;

    Ok(Json(
        services::email_generation::preview_email(
            &template,
            &attachments,
            request.body.as_str(),
            &request.global_context,
            &request.instance_context,
            &state.email_attachment_ctx,
        )
        .await,
    ))
}

pub async fn get_attachments(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<Json<Vec<EmailAttachment>>> {
    auth.verify(vec![UserRole::Admin])?;
    Ok(Json(
        email_attachment_template::Entity::find()
            .filter(email_attachment_template::COLUMN.template_id.eq(id))
            .all(&state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_attachment(
    State(state): State<AppState>,
    auth: Auth,
    Path((id, attachment_id)): Path<(i32, i32)>,
) -> AppResult<Json<EmailAttachment>> {
    auth.verify(vec![UserRole::Admin])?;
    Ok(Json(
        email_attachment_template::Entity::find_by_id(attachment_id)
            .filter(email_attachment_template::COLUMN.template_id.eq(id))
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Attachment does not exist".into()))?
            .into(),
    ))
}

pub async fn delete_attachment(
    State(state): State<AppState>,
    auth: Auth,
    Path((id, attachment_id)): Path<(i32, i32)>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    email_attachment_template::Entity::find_by_id(attachment_id)
        .filter(email_attachment_template::COLUMN.template_id.eq(id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Attachmen does not exist".into()))?
        .delete(&state.db)
        .await?;
    Ok(())
}

pub async fn add_attachment(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<Json<EmailAttachment>> {
    auth.verify(vec![UserRole::Admin])?;
    let email = email_template::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Email does not exist".into()))?;

    Ok(Json(
        email_attachment_template::ActiveModel::create("New attachment".into(), email, "".into())
            .insert(&state.db)
            .await?
            .into(),
    ))
}

pub async fn update_attachment(
    State(state): State<AppState>,
    auth: Auth,
    Path((id, attachment_id)): Path<(i32, i32)>,
    Json(request): Json<UpdateAttachmentRequest>,
) -> AppResult<Json<EmailAttachment>> {
    auth.verify(vec![UserRole::Admin])?;
    let mut attachment = email_attachment_template::Entity::find_by_id(attachment_id)
        .filter(email_attachment_template::COLUMN.template_id.eq(id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Attachmen does not exist".into()))?
        .into_active_model();

    attachment.name = ActiveValue::Set(request.name);
    attachment.template = ActiveValue::Set(request.template);

    Ok(Json(attachment.update(&state.db).await?.into()))
}

pub async fn send_email(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
    Json(request): Json<SendEmailRequest>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;

    let template = email_template::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Template does not exist".into()))?;

    let round = round::Entity::find_by_id(request.round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;

    services::email_generation::send_round_email(
        &template,
        &round,
        request.headline,
        request.body,
        &state.db,
    )
    .await
}
