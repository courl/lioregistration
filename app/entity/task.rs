#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub round_id: i32,
    pub name: String,
    pub ordinal: i32,
    pub max_points: i32,
    #[sea_orm(belongs_to, from = "round_id", to = "id")]
    pub round: HasOne<super::round::Entity>,
    #[sea_orm(has_many)]
    pub scores: HasMany<super::task_score::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(round: super::round::Model, name: String, ordinal: i32, max_points: i32) -> Self {
        Self {
            id: ActiveValue::NotSet,
            round_id: ActiveValue::Set(round.id),
            name: ActiveValue::Set(name),
            ordinal: ActiveValue::Set(ordinal),
            max_points: ActiveValue::Set(max_points),
        }
    }
}
