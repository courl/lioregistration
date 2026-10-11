use crate::entity::admin_user::UserRole;
use crate::entity::registration::ReviewStatus;
use crate::entity::round_participation::ParticipationState;
use crate::entity::{
    edition, registration, round, round_participation, school, student, task, task_score,
};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::routes::admin::edition::Edition;
use crate::routes::admin::registrations::Registration;
use crate::services;
use crate::services::student::AwardInfo;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::prelude::ChronoDate;
use sea_orm::{ModelTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub struct Student {
    pub id: i32,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub birthdate: ChronoDate,
    pub awards: Vec<AwardInfo>,
}

#[derive(Serialize)]
pub struct StudentRoundParticipationScore {
    pub task: String,
    pub score: f32,
    pub max_points: i32,
}

#[derive(Serialize)]
pub struct StudentRoundParticipation {
    pub id: i32,
    pub name: String,
    pub state: ParticipationState,
    pub scores: Vec<StudentRoundParticipationScore>,
}

#[derive(Serialize)]
pub struct StudentParticipation {
    pub edition: Edition,
    pub registration: Registration,
    pub rounds: Vec<StudentRoundParticipation>,
}

#[derive(Serialize)]
pub struct StudentDetails {
    #[serde(flatten)]
    pub student: Student,
    pub participations: Vec<StudentParticipation>,
}

impl From<(student::Model, Vec<AwardInfo>)> for Student {
    fn from((student, awards): (student::Model, Vec<AwardInfo>)) -> Self {
        Self {
            id: student.id,
            given_name: student.given_name,
            family_name: student.family_name,
            email: student.email,
            birthdate: student.birthdate,
            awards,
        }
    }
}

#[derive(Deserialize)]
pub struct StudentQuery {
    query: String,
}

pub async fn get_students(
    State(state): State<AppState>,
    auth: Auth,
    Query(query): Query<StudentQuery>,
) -> AppResult<Json<Vec<Student>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        services::student::find_students(&query.query, &state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_students_for_edition(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
) -> AppResult<Json<Vec<Student>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let students = registration::Entity::find()
        .filter(registration::COLUMN.edition_id.eq(edition_id))
        .filter(registration::COLUMN.review_status.ne(ReviewStatus::Pending))
        .filter(registration::COLUMN.student_id.is_not_null())
        .find_also_related(student::Entity)
        .all(&state.db)
        .await?
        .into_iter()
        .map(|(_, student)| {
            student.ok_or_else(|| AppError::Internal(anyhow::anyhow!("Database Corruption")))
        })
        .collect::<AppResult<Vec<_>>>()?;
    let mut awards = services::student::load_awards(
        students.iter().map(|x| x.id).collect::<Vec<_>>(),
        &state.db,
    )
    .await?;

    Ok(Json(
        students
            .into_iter()
            .map(|student| {
                let id = student.id;
                (
                    student,
                    awards.remove(&id).unwrap_or_else(Vec::<AwardInfo>::new),
                )
                    .into()
            })
            .collect(),
    ))
}

pub async fn get_student_details(
    State(state): State<AppState>,
    auth: Auth,
    Path(id): Path<i32>,
) -> AppResult<Json<StudentDetails>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let mut result = student::Entity::find_by_id(id)
        .find_also_related(registration::Entity)
        .and_also_related(school::Entity)
        .consolidate()
        .all(&state.db)
        .await?;

    if result.is_empty() {
        return Err(AppError::NotFound("Student does not exist".into()));
    }

    let (student, registrations) = result.swap_remove(0);
    let mut registrations: HashMap<i32, (registration::Model, Option<school::Model>)> =
        registrations
            .into_iter()
            .map(|(reg, mut schools)| {
                (
                    reg.edition_id,
                    (
                        reg,
                        if schools.len() != 1 {
                            None
                        } else {
                            Some(schools.swap_remove(0))
                        },
                    ),
                )
            })
            .collect();

    let participations = student
        .find_related(round_participation::Entity)
        .find_also_related(round::Entity)
        .all(&state.db)
        .await?
        .into_iter()
        .map(|(participation, rounds)| {
            let round = rounds.ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Database corruption, no rounds for participation"
                ))
            })?;
            Ok::<_, AppError>((participation, round))
        })
        .collect::<AppResult<Vec<_>>>()?;

    let editions = participations
        .iter()
        .map(|(_, round)| round.edition_id)
        .collect::<HashSet<_>>();
    let mut editions = edition::Entity::find()
        .filter(edition::COLUMN.id.is_in(editions))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|x| (x.id, x))
        .collect::<HashMap<_, _>>();

    let mut edition_participations: HashMap<i32, StudentParticipation> = HashMap::new();

    let tasks = task_score::Entity::find()
        .filter(
            task_score::COLUMN.round_participation_id.is_in(
                participations
                    .iter()
                    .map(|(participation, _)| participation.id),
            ),
        )
        .find_also_related(task::Entity)
        .all(&state.db)
        .await?
        .into_iter()
        .map(|(score, task)| {
            task.ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Database corruption: no task {} for task score",
                    score.id
                ))
            })
            .map(|task| (score, task))
        })
        .collect::<AppResult<Vec<_>>>()?;

    let mut task_map = HashMap::<i32, Vec<_>>::new();
    for (score, task) in tasks {
        task_map
            .entry(task.round_id)
            .or_insert_with(Vec::new)
            .push((task, score.points));
    }

    for (participation, round) in participations {
        let edition_id = round.edition_id;
        let edition_participation = match edition_participations.entry(edition_id) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let edition = editions.remove(&edition_id).ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "Database corruption: no edition {} for participation",
                        edition_id
                    ))
                })?;
                let registration = registrations.remove(&edition_id).ok_or_else(|| {
                    AppError::Internal(anyhow::anyhow!(
                        "Database corruption: no registration for edition {}",
                        edition_id
                    ))
                })?;
                e.insert(StudentParticipation {
                    edition: edition.into(),
                    registration: registration.into(),
                    rounds: vec![],
                })
            }
        };

        edition_participation
            .rounds
            .push(StudentRoundParticipation {
                id: round.id,
                name: round.name,
                state: participation.state,
                scores: task_map
                    .remove(&round.id)
                    .unwrap_or_else(Vec::new)
                    .into_iter()
                    .map(|(task, score)| StudentRoundParticipationScore {
                        task: task.name,
                        score,
                        max_points: task.max_points,
                    })
                    .collect(),
            })
    }

    let awards = services::student::load_awards_for_student(&student, &state.db).await?;

    Ok(Json(StudentDetails {
        student: (student, awards).into(),
        participations: edition_participations.into_values().collect(),
    }))
}
