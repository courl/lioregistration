#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum ReviewStatus {
    Pending,
    Validated,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum FirstContact {
    Teachers,
    Homepage,
    Friends,
    LinkedIn,
    Poster,
    Instagram,
    Other,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "registration")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub edition_id: i32,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub birthdate: ChronoDate,
    pub gender: Gender,
    pub address_street: String,
    pub address_postal_code: String,
    pub address_locality: String,
    pub phone: String,
    pub programming_language_python: bool,
    pub programming_language_cpp: bool,
    pub programming_language_java: bool,
    pub first_contact_source: FirstContact,
    pub first_contact_other: Option<String>,
    pub grade: String,
    pub registered_at: ChronoDateTimeUtc,
    pub review_status: ReviewStatus,
    pub reviewed_at: Option<ChronoDateTimeUtc>,
    pub reviewed_by_id: Option<i32>,
    pub school_id: i32,
    pub student_id: Option<i32>,
    #[sea_orm(belongs_to, from = "edition_id", to = "id")]
    pub edition: HasOne<super::edition::Entity>,
    #[sea_orm(belongs_to, from = "school_id", to = "id")]
    pub school: HasOne<super::school::Entity>,
    #[sea_orm(belongs_to, from = "reviewed_by_id", to = "id")]
    pub reviewed_by: HasOne<super::admin_user::Entity>,
    #[sea_orm(belongs_to, from = "student_id", to = "id")]
    pub student: HasOne<super::student::Entity>,
    #[sea_orm(has_one)]
    pub credentials: HasOne<super::credentials::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Deserialize)]
pub struct NewRegistration {
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub birthdate: chrono::NaiveDate,
    pub gender: Gender,
    pub address_street: String,
    pub address_postal_code: String,
    pub address_locality: String,
    pub phone: String,
    pub grade: String,
    pub school_name: String,
    pub programming_languages: Vec<String>,
    pub first_contact_source: FirstContact,
    pub first_contact_other: Option<String>,
}

impl NewRegistration {
    /// Normalizes (trim, lowercase email) and validates the public form input.
    pub fn validate(&mut self, today: chrono::NaiveDate) -> crate::error::AppResult<()> {
        use crate::error::AppError;
        use crate::utils::validation::{MAX_FIELD_LEN, ensure_email, required_text};

        required_text(&mut self.given_name, "given_name")?;
        required_text(&mut self.family_name, "family_name")?;
        required_text(&mut self.email, "email")?;
        self.email = self.email.to_lowercase();
        ensure_email(&self.email)?;
        required_text(&mut self.address_street, "address_street")?;
        required_text(&mut self.address_postal_code, "address_postal_code")?;
        required_text(&mut self.address_locality, "address_locality")?;
        required_text(&mut self.phone, "phone")?;
        required_text(&mut self.grade, "grade")?;
        required_text(&mut self.school_name, "school_name")?;
        if let Some(other) = &mut self.first_contact_other {
            *other = other.trim().to_owned();
            if other.chars().count() > MAX_FIELD_LEN {
                return Err(AppError::BadRequest(format!(
                    "first_contact_other must be at most {MAX_FIELD_LEN} characters"
                )));
            }
        }
        let oldest = chrono::NaiveDate::from_ymd_opt(1900, 1, 1).unwrap_or(chrono::NaiveDate::MIN);
        if self.birthdate >= today || self.birthdate < oldest {
            return Err(AppError::BadRequest("birthdate is not plausible".into()));
        }
        Ok(())
    }

    #[must_use]
    pub fn into_active_model(self, edition_id: i32, school: &super::school::Model) -> ActiveModel {
        ActiveModel {
            id: Default::default(),
            edition_id: ActiveValue::Set(edition_id),
            given_name: ActiveValue::Set(self.given_name),
            family_name: ActiveValue::Set(self.family_name),
            email: ActiveValue::Set(self.email),
            birthdate: ActiveValue::Set(self.birthdate),
            gender: ActiveValue::Set(self.gender),
            address_street: ActiveValue::Set(self.address_street),
            address_postal_code: ActiveValue::Set(self.address_postal_code),
            address_locality: ActiveValue::Set(self.address_locality),
            phone: ActiveValue::Set(self.phone),
            programming_language_python: ActiveValue::Set(
                self.programming_languages.contains(&String::from("Python")),
            ),
            programming_language_cpp: ActiveValue::Set(
                self.programming_languages.contains(&String::from("C/C++")),
            ),
            programming_language_java: ActiveValue::Set(
                self.programming_languages.contains(&String::from("Java")),
            ),
            first_contact_source: ActiveValue::Set(self.first_contact_source),
            first_contact_other: ActiveValue::Set(self.first_contact_other),
            grade: ActiveValue::Set(self.grade),
            registered_at: ActiveValue::Set(chrono::Utc::now()),
            review_status: ActiveValue::Set(ReviewStatus::Pending),
            reviewed_at: ActiveValue::Set(None),
            reviewed_by_id: ActiveValue::Set(None),
            school_id: ActiveValue::Set(school.id),
            student_id: ActiveValue::Set(None),
        }
    }
}
