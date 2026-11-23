use crate::entity::registration::NewRegistration;
use crate::error::AppResult;
use crate::middleware::rate_limit::RegisterRateLimit;
use crate::services;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;

#[axum::debug_handler]
pub async fn register(
    _rl: RegisterRateLimit,
    State(state): State<AppState>,
    Json(request): Json<NewRegistration>,
) -> AppResult<()> {
    let edition = services::editions::active_edition(&state.db).await?;
    services::registration::register_student(&edition, request, &state.db).await?;
    Ok(())
}
