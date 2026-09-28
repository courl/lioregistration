#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

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
}
