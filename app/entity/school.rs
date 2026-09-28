#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "school")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(has_many)]
    pub registrations: HasMany<super::registration::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    #[must_use]
    pub fn create(name: &str) -> Self {
        Self {
            name: ActiveValue::Set(name.into()),
            ..Default::default()
        }
    }
}
