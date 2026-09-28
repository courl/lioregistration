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
pub enum TemplateType {
    Registration,
    Round,
    Error,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub template: String,
    pub template_type: Option<TemplateType>,

    #[sea_orm(has_many)]
    pub attachments: HasMany<super::email_attachment_template::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(name: String, template: String, template_type: TemplateType) -> Self {
        Self {
            id: Default::default(),
            name: ActiveValue::Set(name),
            template: ActiveValue::Set(template),
            template_type: ActiveValue::Set(Some(template_type)),
        }
    }
}
