use crate::middleware::rate_limit::global_rate_limit;
use crate::state::AppState;
use axum::Router;

mod admin;
mod public;

pub fn build(state: AppState) -> Router {
    Router::new()
        .nest("/api", public::router())
        .nest("/api/admin", admin::router())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            global_rate_limit,
        ))
        .with_state(state)
}
