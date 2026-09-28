use crate::error::AppResult;
use crate::services;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SchoolQuery {
    query: String,
}

pub async fn find_schools(
    State(state): State<AppState>,
    Query(query): Query<SchoolQuery>,
) -> AppResult<Json<Vec<String>>> {
    Ok(Json(
        services::schools::find_schools(query.query.as_str(), &state.db)
            .await?
            .into_iter()
            .map(|s| s.name)
            .collect(),
    ))
}
