use crate::error::AppError;
use crate::middleware::rate_limit::global_rate_limit;
use crate::state::AppState;
use axum::Router;
use axum::routing::any;
use std::path::Path;
use tower_http::services::{ServeDir, ServeFile};

mod admin;
mod public;
#[cfg(test)]
mod tests;

pub fn build(state: AppState) -> Router {
    Router::new()
        .nest("/api", public::router())
        .nest("/api/admin", admin::router())
        // Unknown API paths are a JSON 404, never the SPA's index.html.
        .route(
            "/api/{*rest}",
            any(|| async { AppError::NotFound("No such endpoint".into()) }),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            global_rate_limit,
        ))
        .with_state(state)
}

/// Serves the compiled SPA from `dir` for every non-API path. Paths without a matching file
/// get `index.html` so client-side routes (`/admin/...`) survive a reload. Added after the
/// rate-limit layer, so static assets don't count against the API budget.
pub fn with_frontend(router: Router, dir: &Path) -> Router {
    let index = ServeFile::new(dir.join("index.html"));
    router.fallback_service(ServeDir::new(dir).fallback(index))
}
