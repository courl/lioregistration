use crate::entity::admin_user::UserRole;
use crate::entity::edition;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::services;
use crate::services::editions::EditionStatistics;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::prelude::ChronoDate;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Edition {
    pub id: i32,
    pub year: i32,
    pub name: String,
    pub active: bool,
    pub registration_opens_at: ChronoDate,
    pub registration_closes_at: ChronoDate,
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub name: String,
    pub year: u32,
    pub registration_opens_at: ChronoDate,
    pub registration_closes_at: ChronoDate,
}

impl From<edition::Model> for Edition {
    fn from(value: edition::Model) -> Self {
        Self {
            id: value.id,
            year: value.year,
            name: value.name,
            active: value.active,
            registration_opens_at: value.registration_opens_at,
            registration_closes_at: value.registration_closes_at,
        }
    }
}

impl From<&edition::Model> for Edition {
    fn from(value: &edition::Model) -> Self {
        Self {
            id: value.id,
            year: value.year,
            name: value.name.clone(),
            active: value.active,
            registration_opens_at: value.registration_opens_at,
            registration_closes_at: value.registration_closes_at,
        }
    }
}

pub async fn active_edition(State(state): State<AppState>, auth: Auth) -> AppResult<Json<Edition>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        services::editions::active_edition(&state.db).await?.into(),
    ))
}

pub async fn edition(
    Path(id): Path<i32>,
    auth: Auth,
    State(state): State<AppState>,
) -> AppResult<Json<Edition>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        services::editions::get_edition(id, &state.db).await?.into(),
    ))
}

pub async fn set_active(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let edition = services::editions::get_edition(id, &state.db).await?;
    services::editions::set_active(&edition, &state.db).await?;
    Ok(())
}

pub async fn create_edition(
    State(state): State<AppState>,
    auth: Auth,
    Json(body): Json<CreateRequest>,
) -> AppResult<Json<Edition>> {
    auth.verify(vec![UserRole::Admin])?;
    crate::utils::validation::ensure_date_range(
        body.registration_opens_at,
        body.registration_closes_at,
        "Registration period",
    )?;
    let model = edition::ActiveModel::create(
        &body.name,
        i32::try_from(body.year)
            .map_err(|_| AppError::BadRequest("Year is out of range".into()))?,
        body.registration_opens_at,
        body.registration_closes_at,
    )
    .insert(&state.db)
    .await?;
    Ok(Json(model.into()))
}

pub async fn editions(State(state): State<AppState>, auth: Auth) -> AppResult<Json<Vec<Edition>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        edition::Entity::find()
            .order_by_desc(edition::COLUMN.registration_opens_at)
            .all(&state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn statistics(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<Json<EditionStatistics>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let edition = services::editions::get_edition(id, &state.db).await?;
    Ok(Json(
        services::editions::statistics(&edition, &state.db).await?,
    ))
}
