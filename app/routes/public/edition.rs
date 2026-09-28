use crate::error::{AppError, AppResult};
use crate::services;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use sea_orm::prelude::ChronoDate;
use serde::Serialize;

#[derive(Serialize)]
pub struct Edition {
    pub name: String,
    pub registration_opens_at: ChronoDate,
    pub registration_closes_at: ChronoDate,
}

#[axum::debug_handler]
pub async fn get_active_edition(State(state): State<AppState>) -> AppResult<Json<Option<Edition>>> {
    let edition = services::editions::active_edition(&state.db).await;
    match edition {
        Ok(edition) => Ok(Json(Some(Edition {
            name: edition.name,
            registration_opens_at: edition.registration_opens_at,
            registration_closes_at: edition.registration_closes_at,
        }))),
        Err(AppError::NotFound(_)) => Ok(Json(None)),
        Err(e) => Err(e),
    }
}
