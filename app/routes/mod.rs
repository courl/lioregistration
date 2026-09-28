use crate::state::AppState;
use axum::Router;

mod admin;
mod public;

pub fn build(state: AppState) -> Router {
    Router::new()
        .nest("/api", public::router())
        .nest("/api/admin", admin::router())
        .with_state(state)
}
