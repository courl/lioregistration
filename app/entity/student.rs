#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub birthdate: ChronoDate,
    #[sea_orm(has_many)]
    pub registrations: HasMany<super::registration::Entity>,
    #[sea_orm(has_many)]
    pub participations: HasMany<super::round_participation::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(
        given_name: String,
        family_name: String,
        email: String,
        birthdate: ChronoDate,
    ) -> Self {
        Self {
            id: Default::default(),
            given_name: ActiveValue::Set(given_name),
            family_name: ActiveValue::Set(family_name),
            email: ActiveValue::Set(email),
            birthdate: ActiveValue::Set(birthdate),
        }
    }
}
