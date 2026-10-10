use crate::error::AppError;
use crate::middleware::rate_limit::global_rate_limit;
use crate::state::AppState;
use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::any;
use std::convert::Infallible;
use std::path::Path;
use tower::{ServiceExt, service_fn};
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
/// get `index.html` so client-side routes (`/admin/...`) survive a reload, except missing
/// files (see [`looks_like_file`]). Added after the rate-limit layer, so static assets don't
/// count against the API budget.
pub fn with_frontend(router: Router, dir: &Path) -> Router {
    let index = ServeFile::new(dir.join("index.html"));
    let spa_fallback = service_fn(move |request: Request| {
        let index = index.clone();
        async move {
            if looks_like_file(request.uri().path()) {
                return Ok::<_, Infallible>(StatusCode::NOT_FOUND.into_response());
            }
            index
                .oneshot(request)
                .await
                .map(IntoResponse::into_response)
        }
    });
    router.fallback_service(ServeDir::new(dir).fallback(spa_fallback))
}

/// A path whose last segment has an extension names a static file (`/assets/index-3f2a.js`),
/// so a miss is a real 404. Answering it with `index.html` would hand a browser that still
/// runs an old build HTML for its old bundle hash, and a blank page.
fn looks_like_file(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'))
}
