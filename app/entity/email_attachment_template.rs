#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_attachment_template")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub template_id: i32,
    pub name: String,
    pub template: String,
    #[sea_orm(belongs_to, from = "template_id", to = "id")]
    pub email: HasOne<super::email_template::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(name: String, email: super::email_template::Model, template: String) -> Self {
        Self {
            id: Default::default(),
            name: ActiveValue::Set(name),
            template_id: ActiveValue::Set(email.id),
            template: ActiveValue::Set(template),
        }
    }
}
