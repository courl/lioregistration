#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use crate::error::{AppError, AppResult};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "edition")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub year: u32,
    pub name: String,
    pub active: bool,
    pub registration_opens_at: ChronoDate,
    pub registration_closes_at: ChronoDate,

    #[sea_orm(has_many)]
    pub credentials: HasMany<super::credentials::Entity>,

    #[sea_orm(has_many)]
    pub registrations: HasMany<super::registration::Entity>,

    #[sea_orm(has_many)]
    pub rounds: HasMany<super::round::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(
        name: &str,
        year: u32,
        registration_opens_at: ChronoDate,
        registration_closes_at: ChronoDate,
    ) -> Self {
        Self {
            name: ActiveValue::Set(name.into()),
            year: ActiveValue::Set(year),
            registration_opens_at: ActiveValue::Set(registration_opens_at),
            registration_closes_at: ActiveValue::Set(registration_closes_at),
            active: ActiveValue::Set(false),
            ..Default::default()
        }
    }
}

impl Model {
    #[must_use]
    pub fn is_registration_open(&self, today: chrono::NaiveDate) -> bool {
        today >= self.registration_opens_at && today <= self.registration_closes_at
    }

    /// Only the active edition may be changed, past ones are read-only records.
    pub fn ensure_editable(&self) -> AppResult<()> {
        if self.active {
            Ok(())
        } else {
            Err(AppError::Conflict(format!(
                "{} is not the active edition and is read-only",
                self.name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_active_edition_is_editable() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let mut edition = Model {
            id: 1,
            year: 2026,
            name: "LIO 2026".into(),
            active: false,
            registration_opens_at: date,
            registration_closes_at: date,
        };
        assert!(matches!(
            edition.ensure_editable(),
            Err(AppError::Conflict(_))
        ));
        edition.active = true;
        assert!(edition.ensure_editable().is_ok());
    }
}
