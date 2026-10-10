use crate::entity::admin_user::UserRole;
use crate::entity::registration::{FirstContact, Gender, ReviewStatus};
use crate::entity::{credentials, edition, registration, school, student};
use crate::error::{AppError, AppResult};
use crate::middleware::auth::Auth;
use crate::routes::admin::credentials::Credentials;
use crate::routes::admin::edition::Edition;
use crate::routes::admin::student::Student;
use crate::services;
use crate::services::student::AwardInfo;
use crate::state::AppState;
use anyhow::anyhow;
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::QueryFilter;
use sea_orm::prelude::ChronoDate;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel};
use serde::Serialize;
// TODO: The flow registration -[link to]-> student -[link to]-> round_participation is good but maybe a bit opaque

#[derive(Serialize)]
pub struct Registration {
    pub id: i32,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub school: String,
    pub grade: String,
    pub registration_date: ChronoDate,
    pub review_status: ReviewStatus,
}

impl From<(registration::Model, Option<school::Model>)> for Registration {
    fn from((reg, school): (registration::Model, Option<school::Model>)) -> Self {
        Self {
            id: reg.id,
            given_name: reg.given_name,
            family_name: reg.family_name,
            email: reg.email,
            school: school.map_or_else(String::new, |s| s.name),
            grade: reg.grade,
            registration_date: reg.registered_at.date_naive(),
            review_status: reg.review_status,
        }
    }
}

#[derive(Serialize)]
pub struct RegistrationDetail {
    #[serde(flatten)]
    pub registration: Registration,
    pub edition: Edition,
    pub birthdate: ChronoDate,
    pub gender: Gender,
    pub address_street: String,
    pub address_postal_code: String,
    pub address_locality: String,
    pub phone: String,
    pub student: Option<Student>,
    pub credentials: Option<Credentials>,
    pub programming_languages: Vec<String>,
    pub first_contact_source: FirstContact,
    pub first_contact_other: Option<String>,
}

impl
    From<(
        registration::Model,
        Option<school::Model>,
        Option<student::Model>,
        Vec<AwardInfo>,
        Option<credentials::Model>,
        edition::Model,
    )> for RegistrationDetail
{
    fn from(
        (reg, school, student, awards, credentials, edition): (
            registration::Model,
            Option<school::Model>,
            Option<student::Model>,
            Vec<AwardInfo>,
            Option<credentials::Model>,
            edition::Model,
        ),
    ) -> Self {
        let mut programming_languages = Vec::new();
        if reg.programming_language_python {
            programming_languages.push(String::from("Python"));
        }
        if reg.programming_language_cpp {
            programming_languages.push(String::from("C/C++"));
        }
        if reg.programming_language_java {
            programming_languages.push(String::from("Java"));
        }
        Self {
            registration: (reg.clone(), school).into(),
            edition: edition.into(),
            birthdate: reg.birthdate,
            gender: reg.gender,
            address_street: reg.address_street,
            address_postal_code: reg.address_postal_code,
            address_locality: reg.address_locality,
            phone: reg.phone,
            student: student.map(|student| (student, awards).into()),
            credentials: credentials.map(|x| (x, None).into()),
            programming_languages,
            first_contact_source: reg.first_contact_source,
            first_contact_other: reg.first_contact_other,
        }
    }
}

pub async fn get_registrations(
    State(state): State<AppState>,
    auth: Auth,
    Path(edition_id): Path<i32>,
) -> AppResult<Json<Vec<Registration>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    Ok(Json(
        registration::Entity::find()
            .filter(registration::COLUMN.edition_id.eq(edition_id))
            .find_also_related(school::Entity)
            .all(&state.db)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_registration(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<Json<RegistrationDetail>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let (reg, school, student, credentials, edition) =
        registration::Entity::find_by_id(registration_id)
            .find_also_related(school::Entity)
            .find_also_related(student::Entity)
            .find_also(registration::Entity, credentials::Entity)
            .find_also(registration::Entity, edition::Entity)
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound("Registration does not exist".into()))?;

    let edition = edition.ok_or(AppError::Internal(anyhow!(
        "Database Corruption: Edition does not exist"
    )))?;

    let awards = if let Some(student) = &student {
        services::student::load_awards_for_student(student, &state.db).await?
    } else {
        Vec::new()
    };

    Ok(Json(
        (reg, school, student, awards, credentials, edition).into(),
    ))
}

pub async fn get_candidates(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<Json<Vec<(u32, Student)>>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let registration = registration::Entity::find_by_id(registration_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;

    let candidates = services::registration::student_candidates(&registration, &state.db).await?;
    let mut awards =
        services::student::load_awards(candidates.iter().map(|x| x.1.id).collect(), &state.db)
            .await?;

    Ok(Json(
        candidates
            .into_iter()
            .map(|(score, student)| {
                let id = student.id;
                (
                    score,
                    (student, awards.remove(&id).unwrap_or_else(Vec::new)).into(),
                )
            })
            .collect(),
    ))
}

pub async fn get_candidate(
    State(state): State<AppState>,
    auth: Auth,
    Path((registration_id, student_id)): Path<(i32, i32)>,
) -> AppResult<Json<(u32, Student)>> {
    auth.verify(vec![UserRole::Observer, UserRole::Admin])?;
    let registration = registration::Entity::find_by_id(registration_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;

    let student = student::Entity::find_by_id(student_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Student does not exist".into()))?;

    let mut awards = services::student::load_awards(vec![student.id], &state.db).await?;

    let score = services::registration::calculate_match(&registration, &student);
    Ok(Json((
        score,
        (student, awards.remove(&student_id).unwrap_or_else(Vec::new)).into(),
    )))
}

pub async fn validate_and_create_new_student(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let (registration, edition) = registration::Entity::find_by_id(registration_id)
        .find_also_related(edition::Entity)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;
    let edition = edition.ok_or(AppError::Internal(anyhow::anyhow!(
        "Database corruption: No edition for registration"
    )))?;
    edition.ensure_editable()?;

    services::registration::link_registration(
        &edition,
        registration,
        &auth.user,
        services::registration::LinkBehaviour::CreateNew,
        &state.db,
    )
    .await
}

pub async fn link_to_student(
    State(state): State<AppState>,
    auth: Auth,
    Path((registration_id, student_id)): Path<(i32, i32)>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let (registration, edition) = registration::Entity::find_by_id(registration_id)
        .find_also_related(edition::Entity)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;

    let student = student::Entity::find_by_id(student_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Student does not exist".into()))?;

    let edition = edition.ok_or(AppError::Internal(anyhow::anyhow!(
        "Database corruption: No edition for registration"
    )))?;
    edition.ensure_editable()?;

    services::registration::link_registration(
        &edition,
        registration,
        &auth.user,
        services::registration::LinkBehaviour::Link(student),
        &state.db,
    )
    .await
}

pub async fn unlink_from_student(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let (registration, edition) = registration::Entity::find_by_id(registration_id)
        .find_also_related(edition::Entity)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;
    let edition = edition.ok_or(AppError::Internal(anyhow::anyhow!(
        "Database corruption: No edition for registration"
    )))?;
    edition.ensure_editable()?;

    services::registration::link_registration(
        &edition,
        registration,
        &auth.user,
        services::registration::LinkBehaviour::Unlink,
        &state.db,
    )
    .await
}

pub async fn refuse(
    State(state): State<AppState>,
    auth: Auth,
    Path(registration_id): Path<i32>,
) -> AppResult<()> {
    auth.verify(vec![UserRole::Admin])?;
    let registration = registration::Entity::find_by_id(registration_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Registration does not exist".into()))?;
    services::editions::get_editable_edition(registration.edition_id, &state.db).await?;
    let first_review = registration.reviewed_at.is_none();
    let mut registration = registration.into_active_model();

    registration.review_status = ActiveValue::Set(ReviewStatus::Refused);
    registration.reviewed_by_id = ActiveValue::Set(Some(auth.user.id));
    if first_review {
        registration.reviewed_at = ActiveValue::Set(chrono::Utc::now().into());
    }
    registration.save(&state.db).await?;

    // TODO: if it is linked to a student, unlink it and remove round registrations?

    Ok(())
}
