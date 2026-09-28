use crate::entity::admin_user::UserRole;
use crate::entity::round_participation::ParticipationState;
use crate::entity::{
    credentials, edition, registration, round, round_participation, student, task, task_score,
};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::routes::admin::edition::Edition;
use crate::routes::admin::student::Student;
use crate::services;
use crate::services::rounds::Statistics;
use crate::services::student::AwardInfo;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::prelude::ChronoDate;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel, ModelTrait};
use sea_orm::{QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct CreateRoundRequest {
    pub name: String,
    pub starts_at: ChronoDate,
    pub ends_at: ChronoDate,
}

#[derive(Deserialize)]
pub struct UpdateRoundRequest {
    pub name: Option<String>,
    pub starts_at: Option<ChronoDate>,
    pub ends_at: Option<ChronoDate>,
}

#[derive(Serialize)]
pub struct Round {
    pub id: i32,
    pub name: String,
    pub edition: Edition,
    pub starts_at: ChronoDate,
    pub ends_at: ChronoDate,
    pub total_points: u32,
    pub task_count: u32,
    pub participant_count: u32,
}

#[derive(Serialize)]
pub struct RoundDetail {
    #[serde(flatten)]
    pub round: Round,
    pub tasks: Vec<Task>,
    pub participants: Vec<Participant>,
}

#[derive(Serialize)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub ordinal: i32,
    pub max_points: u32,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub ordinal: i32,
    pub max_points: u32,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub ordinal: Option<i32>,
    pub max_points: Option<u32>,
}

#[derive(Serialize)]
pub struct Participant {
    pub student: Student,
    pub state: ParticipationState,
    pub points: Vec<f32>, // points per task, aligned to round.tasks order (by ordinal)
    pub total: f32,
}

#[derive(Deserialize)]
pub struct TaskPoint(i32, f32);

#[derive(Deserialize)]
pub struct UpdateParticipantRequest {
    pub state: Option<ParticipationState>,
    pub points: Vec<TaskPoint>,
}

#[derive(Deserialize)]
pub struct UpdateScoreRequest {
    credentials_username: String,
    task_scores: Vec<(i32, f32)>,
}

impl Task {
    fn create(task: &task::Model) -> Self {
        Self {
            id: task.id,
            name: task.name.clone(),
            ordinal: task.ordinal,
            max_points: task.max_points,
        }
    }
}

impl Participant {
    fn create(
        participant: services::rounds::Participant,
        tasks: &Vec<task::Model>,
        awards: Vec<AwardInfo>,
    ) -> Self {
        let mut points = 0.0;
        let mut scores = Vec::new();
        for task in tasks {
            let score = participant
                .scores
                .iter()
                .find_map(|x| {
                    if x.task_id == task.id {
                        Some(x.points)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0);
            points += score;
            scores.push(score);
        }
        Self {
            student: (participant.student, awards).into(),
            state: participant.participation.state,
            points: scores,
            total: points,
        }
    }
}

impl RoundDetail {
    //fixme, we bleed here -> award is hashmap
    fn create(
        round: round::Model,
        details: services::rounds::Details,
        mut award: HashMap<i32, Vec<AwardInfo>>,
        edition: edition::Model,
    ) -> Self {
        let mut participants = details.participants;
        participants.sort_by_key(|participant| {
            participant
                .scores
                .iter()
                .map(|score| (score.points * 100.) as u32)
                .sum::<u32>()
        });
        participants.reverse();
        Self {
            round: Round::create(
                round,
                Statistics {
                    total_points: details.tasks.iter().map(|x| x.max_points).sum(),
                    task_count: details.tasks.len() as u32,
                    participant_count: participants
                        .iter()
                        .filter(|x| x.participation.state == ParticipationState::Accepted)
                        .count() as u32,
                },
                &edition,
            ),
            tasks: details.tasks.iter().map(Task::create).collect(),
            participants: participants
                .into_iter()
                .map(|x| {
                    let id = x.student.id;
                    Participant::create(x, &details.tasks, award.remove(&id).unwrap_or_default())
                })
                .collect(),
        }
    }
}

impl Round {
    fn create(value: round::Model, stats: Statistics, edition: &edition::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            edition: edition.into(),
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            total_points: stats.total_points,
            task_count: stats.task_count,
            participant_count: stats.participant_count,
        }
    }
}

pub async fn rounds(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
) -> AppResult<Json<Vec<Round>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let edition = edition::Entity::find_by_id(edition_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Edition does not exist".into()))?;

    let rounds = round::Entity::find_by_edition(edition_id)
        .all(&state.db)
        .await?;
    let round_ids: Vec<_> = rounds.iter().map(|r| r.id).collect();
    let mut stats_map = services::rounds::statistics_for_rounds(&round_ids, &state.db).await?;

    let rounds = rounds
        .into_iter()
        .map(|round| {
            let stats = stats_map.remove(&round.id).unwrap_or_default();
            Round::create(round, stats, &edition)
        })
        .collect();

    Ok(Json(rounds))
}

pub async fn create_round(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
    Json(request): Json<CreateRoundRequest>,
) -> AppResult<Json<Round>> {
    auth.verify(vec![UserRole::Admin])?;
    let edition = services::editions::get_edition(edition_id, &state.db).await?;
    let round =
        round::ActiveModel::create(&edition, request.name, request.starts_at, request.ends_at)
            .insert(&state.db)
            .await?;
    let statistics = services::rounds::statistics_for_round(&round, &state.db).await?;
    Ok(Json(Round::create(round, statistics, &edition)))
}

pub async fn update_round(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
    Json(request): Json<UpdateRoundRequest>,
) -> AppResult<Json<Round>> {
    auth.verify(vec![UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;
    let statistics = services::rounds::statistics_for_round(&round, &state.db).await?;
    let edition = services::editions::get_edition(round.edition_id, &state.db).await?;

    let mut round = round.into_active_model();

    if let Some(name) = request.name {
        round.name = ActiveValue::Set(name);
    }

    if let Some(starts_at) = request.starts_at {
        round.starts_at = ActiveValue::Set(starts_at);
    }

    if let Some(ends_at) = request.ends_at {
        round.ends_at = ActiveValue::Set(ends_at);
    }
    let round = round.update(&state.db).await?;
    Ok(Json(Round::create(round, statistics, &edition)))
}

pub async fn delete_round(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;
    round.into_active_model().delete(&state.db).await?;
    // Todo: propagate delete through participations etc.? or block it?
    Ok(())
}

pub async fn round(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
) -> AppResult<Json<RoundDetail>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;
    let edition = round
        .find_related(edition::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "Database corruption, round {round_id} does not have an edition"
            ))
        })?;
    let details = services::rounds::details(&round, &state.db).await?;
    let awards = services::student::load_awards(
        details.participants.iter().map(|x| x.student.id).collect(),
        &state.db,
    )
    .await?;

    Ok(Json(RoundDetail::create(round, details, awards, edition)))
}

pub async fn update_participation_details(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
    Path(student_id): Path<i32>,
    Json(request): Json<UpdateParticipantRequest>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let participation = round_participation::Entity::find_by_round_student((round_id, student_id))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;

    services::rounds::update_participation(
        participation,
        request.state,
        request.points.iter().map(|x| (x.0, x.1)).collect(),
        &state.db,
    )
    .await?;
    Ok(())
}

pub async fn bulk_update_scores(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
    Json(request): Json<Vec<UpdateScoreRequest>>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;

    if request.is_empty() {
        return Ok(());
    }

    let usernames: Vec<String> = request
        .iter()
        .map(|r| r.credentials_username.clone())
        .collect();

    // 1. Batch-fetch credentials for all usernames in this edition
    let credentials = credentials::Entity::find()
        .filter(credentials::COLUMN.edition_id.eq(round.edition_id))
        .filter(credentials::COLUMN.username.is_in(usernames))
        .all(&state.db)
        .await?;

    let registration_ids: Vec<i32> = credentials
        .iter()
        .filter_map(|c| c.registration_id)
        .collect();

    // 2. Batch-fetch students for those registrations
    let students: HashMap<i32, student::Model> = registration::Entity::find()
        .filter(registration::COLUMN.id.is_in(registration_ids))
        .find_also_related(student::Entity)
        .all(&state.db)
        .await?
        .into_iter()
        .filter_map(|(reg, student)| Some((reg.id, student?)))
        .collect();

    // registration_id -> student_id map, via credentials
    let username_to_student_id: HashMap<String, i32> = credentials
        .into_iter()
        .filter_map(|c| {
            let reg_id = c.registration_id?;
            let student = students.get(&reg_id)?;
            Some((c.username, student.id))
        })
        .collect();

    let student_ids: Vec<i32> = username_to_student_id.values().copied().collect();

    // 3. Batch-fetch participations for this round + these students
    let participations: HashMap<i32, round_participation::Model> =
        round_participation::Entity::find()
            .filter(round_participation::COLUMN.round_id.eq(round.id))
            .filter(round_participation::COLUMN.student_id.is_in(student_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|p| (p.student_id, p))
            .collect();

    // 4. Build the full (participation, task_points) batch, skipping unmatched entries
    let mut updates = Vec::new();
    for score_update in request {
        let Some(&student_id) = username_to_student_id.get(&score_update.credentials_username)
        else {
            continue;
        };
        let Some(participation) = participations.get(&student_id) else {
            continue;
        };
        updates.push((participation.clone(), score_update.task_scores));
    }

    // 5. Run all updates inside one transaction
    let txn = state.db.begin().await?;
    for (participation, task_points) in updates {
        services::rounds::update_participation(participation, None, task_points, &txn).await?;
    }
    txn.commit().await?;

    Ok(())
}

pub async fn add_students(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
    Json(student_ids): Json<Vec<i32>>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;

    services::rounds::move_students(round, student_ids, &state.db).await?;
    Ok(())
}

pub async fn add_task(
    State(state): State<AppState>,
    auth: Auth,
    Path(round_id): Path<i32>,
    Json(request): Json<CreateTaskRequest>,
) -> AppResult<Json<Task>> {
    auth.verify(vec![UserRole::Admin])?;
    let round = round::Entity::find_by_id(round_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Round does not exist".into()))?;

    let task = task::ActiveModel::create(round, request.name, request.ordinal, request.max_points)
        .insert(&state.db)
        .await?;

    Ok(Json(Task::create(&task)))
}

pub async fn update_task(
    State(state): State<AppState>,
    auth: Auth,
    Path(task_id): Path<i32>,
    Json(request): Json<UpdateTaskRequest>,
) -> AppResult<Json<Task>> {
    auth.verify(vec![UserRole::Admin])?;
    let mut task = task::Entity::find_by_id(task_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Task does not exist".into()))?
        .into_active_model();

    if let Some(name) = request.name {
        task.name = ActiveValue::Set(name);
    }

    if let Some(ordinal) = request.ordinal {
        task.ordinal = ActiveValue::Set(ordinal);
    }

    if let Some(max_points) = request.max_points {
        task.max_points = ActiveValue::Set(max_points);
    }
    let task = task.update(&state.db).await?;
    Ok(Json(Task::create(&task)))
}

pub async fn delete_task(
    State(state): State<AppState>,
    auth: Auth,
    Path(task_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let task = task::Entity::find_by_id(task_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Task does not exist".into()))?;

    task.delete(&state.db).await?;
    // todo: delete task scores?
    Ok(())
}

pub async fn get_task_scores(
    State(state): State<AppState>,
    auth: Auth,
    Path(task_id): Path<i32>,
) -> AppResult<Json<Vec<f32>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let task = task::Entity::find_by_id(task_id)
        .find_with_related(task_score::Entity)
        .all(&state.db)
        .await?;
    if task.len() != 1 {
        return Err(AppError::NotFound("Task does not exist".into()));
    }
    let (_, scores) = &task[0];
    Ok(Json(scores.iter().map(|s| s.points).collect()))
}
