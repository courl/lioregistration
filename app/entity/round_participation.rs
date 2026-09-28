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
pub enum ParticipationState {
    Accepted,
    Invited,
    Declined,
    NoShow,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, Hash)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum Award {
    Gold,
    Silver,
    Bronze,
    HonorableMention,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "round_participation")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "round_student")]
    pub round_id: i32,
    #[sea_orm(unique_key = "round_student")]
    pub student_id: i32,
    pub state: ParticipationState,
    pub award: Option<Award>,
    #[sea_orm(belongs_to, from = "round_id", to = "id")]
    pub round: HasOne<super::round::Entity>,
    #[sea_orm(belongs_to, from = "student_id", to = "id")]
    pub student: HasOne<super::student::Entity>,
    #[sea_orm(has_many)]
    pub task_score: HasMany<super::task_score::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    #[must_use]
    pub fn create(
        round: &super::round::Model,
        student: &super::student::Model,
        participation_state: ParticipationState,
    ) -> Self {
        Self {
            id: Default::default(),
            round_id: ActiveValue::Set(round.id),
            student_id: ActiveValue::Set(student.id),
            state: ActiveValue::Set(participation_state),
            award: ActiveValue::Set(None),
        }
    }
}
