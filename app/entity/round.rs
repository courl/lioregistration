#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use crate::entity::edition;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "round")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub edition_id: i32,
    pub name: String,
    pub starts_at: ChronoDate,
    pub ends_at: ChronoDate,
    #[sea_orm(belongs_to, from = "edition_id", to = "id")]
    pub edition: HasOne<super::edition::Entity>,
    #[sea_orm(has_many)]
    pub round_participations: HasMany<super::round_participation::Entity>,
    #[sea_orm(has_many)]
    pub tasks: HasMany<super::task::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(
        edition: &edition::Model,
        name: String,
        starts_at: ChronoDate,
        ends_at: ChronoDate,
    ) -> Self {
        Self {
            id: Default::default(),
            edition_id: ActiveValue::Set(edition.id),
            name: ActiveValue::Set(name),
            starts_at: ActiveValue::Set(starts_at),
            ends_at: ActiveValue::Set(ends_at),
        }
    }
}

impl Entity {
    pub fn find_by_edition(edition_id: i32) -> Select<Entity> {
        Self::find().filter(COLUMN.edition_id.eq(edition_id))
    }
}
