mod edition;
mod registration;
mod schools;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/edition", get(edition::get_active_edition))
        .route("/schools", get(schools::find_schools))
        .route("/register", post(registration::register))
}
