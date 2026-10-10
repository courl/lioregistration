#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task_score")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "task_round_participation")]
    pub task_id: i32,
    #[sea_orm(unique_key = "task_round_participation")]
    pub round_participation_id: i32,
    pub points: f32,
    #[sea_orm(belongs_to, from = "task_id", to = "id")]
    pub task: HasOne<super::task::Entity>,
    #[sea_orm(belongs_to, from = "round_participation_id", to = "id")]
    pub participation: HasOne<super::round_participation::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(
        task: &super::task::Model,
        participation: &super::round_participation::Model,
        points: f32,
    ) -> Self {
        Self::create_from_ids(task.id, participation.id, points)
    }
    pub fn create_from_ids(task_id: i32, participation_id: i32, points: f32) -> Self {
        Self {
            id: Default::default(),
            task_id: ActiveValue::Set(task_id),
            round_participation_id: ActiveValue::Set(participation_id),
            points: ActiveValue::Set(points),
        }
    }
}
