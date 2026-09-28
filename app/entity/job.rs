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
pub enum JobType {
    SendEmail,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum JobState {
    Pending,
    Running,
    Failed,
    Completed,
    PartiallyCompleted,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "job")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub job_type: JobType,
    pub state: JobState,
    pub payload: String,
    pub created_at: ChronoDateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(job_type: JobType, payload: String) -> Self {
        Self {
            id: Default::default(),
            job_type: ActiveValue::Set(job_type),
            state: ActiveValue::Set(JobState::Pending),
            payload: ActiveValue::Set(payload),
            created_at: ActiveValue::Set(chrono::Utc::now()),
        }
    }
}
