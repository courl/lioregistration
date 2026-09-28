use crate::entity::admin_user::UserRole;
use crate::entity::{credentials, edition, registration};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::EntityTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, IntoActiveModel, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CredentialsRegistration {
    pub id: i32,
    pub given_name: String,
    pub family_name: String,
}

#[derive(Serialize)]
pub struct Credentials {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub registration: Option<CredentialsRegistration>,
}

#[derive(Deserialize)]
pub struct CreateCredentials {
    pub username: String,
    pub password: String,
}

impl From<(credentials::Model, Option<registration::Model>)> for Credentials {
    fn from(
        (credentials, registration): (credentials::Model, Option<registration::Model>),
    ) -> Self {
        Self {
            id: credentials.id,
            username: credentials.username,
            password: credentials.password,
            registration: registration.map(|x| CredentialsRegistration {
                id: x.id,
                given_name: x.given_name,
                family_name: x.family_name,
            }),
        }
    }
}

pub async fn get_credentials(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
) -> AppResult<Json<Vec<Credentials>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        credentials::Entity::find()
            .filter(credentials::COLUMN.edition_id.eq(edition_id))
            .find_also_related(registration::Entity)
            .all(&state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn create_credentials(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
    Json(credentials): Json<Vec<CreateCredentials>>,
) -> AppResult<Json<Vec<Credentials>>> {
    auth.verify(vec![UserRole::Admin])?;
    let edition = edition::Entity::find_by_id(edition_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Edition does not exists".into()))?;

    let credentials = credentials
        .iter()
        .map(|x| credentials::ActiveModel::create(&x.username, &x.password, &edition))
        .collect::<Vec<_>>();
    credentials::Entity::insert_many(credentials)
        .exec(&state.db)
        .await?;

    Ok(Json(
        credentials::Entity::find()
            .filter(credentials::COLUMN.edition_id.eq(edition_id))
            .find_also_related(registration::Entity)
            .all(&state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn unassign_credentials(
    State(state): State<AppState>,
    auth: Auth,
    Path(credentials_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let mut credentials = credentials::Entity::find_by_id(credentials_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Credentials do not exist".into()))?
        .into_active_model();

    credentials.registration_id = ActiveValue::Set(None);
    credentials.save(&state.db).await?;
    Ok(())
}

pub async fn delete_credentials(
    State(state): State<AppState>,
    auth: Auth,
    Path(credentials_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    credentials::Entity::delete_by_id(credentials_id)
        .exec(&state.db)
        .await?;
    Ok(())
}

pub async fn assign_free_credentials(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let (registration, credentials) = registration::Entity::find_by_id(registration_id)
        .find_also_related(credentials::Entity)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;

    if credentials.is_some() {
        return Err(AppError::BadRequest("Registration already assigned".into()));
    }

    state
        .db
        .transaction::<_, _, AppError>(|txn| {
            Box::pin(async move {
                let mut credentials = credentials::Entity::find_free(registration.edition_id)
                    .one(txn)
                    .await?
                    .ok_or(AppError::NotFound("No free credentials".into()))?
                    .into_active_model();

                credentials.registration_id = ActiveValue::Set(Some(registration.id));
                credentials.save(txn).await?;
                Ok(())
            })
        })
        .await
        .map_err(AppError::from)
}
