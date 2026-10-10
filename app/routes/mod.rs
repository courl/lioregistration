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
    // Unknown API paths are a JSON 404, never the SPA's index.html. The nested fallback covers
    // `/api` and `/api/...`, but not `/api/` itself.
    let api = public::router()
        .nest("/admin", admin::router())
        .fallback(api_not_found);
    Router::new()
        .nest("/api", api)
        .route("/api/", any(api_not_found))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            global_rate_limit,
        ))
        .with_state(state)
}

async fn api_not_found() -> AppError {
    AppError::NotFound("No such endpoint".into())
}

/// Serves the compiled SPA from `dir` for every non-API path. Paths without a matching file
/// get `index.html` so client-side routes (`/admin/...`) survive a reload. Added after the
/// rate-limit layer, so static assets don't count against the API budget.
pub fn with_frontend(router: Router, dir: &Path) -> Router {
    let index = ServeFile::new(dir.join("index.html"));
    router.fallback_service(ServeDir::new(dir).fallback(index))
}
