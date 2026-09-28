use crate::entity::admin_user::UserRole;
use crate::entity::school;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::services;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SchoolQuery {
    query: String,
}
#[derive(Serialize)]
pub struct School {
    id: i32,
    name: String,
}

impl From<school::Model> for School {
    fn from(value: school::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

pub async fn find_schools(
    State(state): State<AppState>,
    auth: Auth,
    Query(query): Query<SchoolQuery>,
) -> AppResult<Json<Vec<School>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        services::schools::find_schools(query.query.as_str(), &state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_school(
    State(state): State<AppState>,
    auth: Auth,
    Path(school_id): Path<i32>,
) -> AppResult<Json<School>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let school = school::Entity::find_by_id(school_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("School not found".into()))?;
    Ok(Json(school.into()))
}

pub async fn merge_schools(
    State(state): State<AppState>,
    auth: Auth,
    Path((school_id, other_school_id)): Path<(i32, i32)>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let schools = school::Entity::find()
        .filter(school::COLUMN.id.is_in(vec![school_id, other_school_id]))
        .all(&state.db)
        .await?;
    if schools.len() != 2 {
        return Err(AppError::NotFound("School not found".into()));
    }

    let (into_school, other_school) = if schools[0].id == school_id {
        (&schools[0], &schools[1])
    } else {
        (&schools[1], &schools[0])
    };
    services::schools::merge(into_school, other_school, &state.db).await
}
