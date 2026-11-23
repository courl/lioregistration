#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use crate::error::{AppError, AppResult};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{LockBehavior, LockType};
use sea_orm::{ActiveValue, ConnectionTrait, QueryOrder, QuerySelect};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "credentials")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "edition_username")]
    pub edition_id: i32,
    #[sea_orm(unique_key = "edition_username")]
    pub username: String,
    pub password: String,
    #[sea_orm(unique)]
    // todo: validate, maybe this needs to be removed, when collision on null with pg
    pub registration_id: Option<i32>,
    #[sea_orm(belongs_to, from = "edition_id", to = "id")]
    pub edition: HasOne<super::edition::Entity>,
    #[sea_orm(belongs_to, from = "registration_id", to = "id")]
    pub registration: HasOne<super::registration::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    /// Unassigned credentials, oldest first. Locks the row on Postgres, where two concurrent
    /// claims would otherwise read the same one (SQLite has a single writer and ignores it).
    pub fn find_free(edition_id: i32) -> Select<Entity> {
        Entity::find()
            .filter(Column::EditionId.eq(edition_id))
            .filter(Column::RegistrationId.is_null())
            .order_by_asc(Column::Id)
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
    }
}

impl ActiveModel {
    #[must_use]
    pub fn create(user: &str, password: &str, edition: &super::edition::Model) -> Self {
        Self {
            username: ActiveValue::Set(user.into()),
            password: ActiveValue::Set(password.into()),
            edition_id: ActiveValue::Set(edition.id),
            registration_id: ActiveValue::Set(None),
            ..Default::default()
        }
    }

    pub fn set_registration(&mut self, registration: &super::registration::Model) {
        self.registration_id = ActiveValue::Set(Some(registration.id));
    }
}

impl Model {
    pub async fn get_credentials_for_edition(
        edition_id: i32,
        usernames: &Vec<String>,
        db: &impl ConnectionTrait,
    ) -> AppResult<Vec<Self>> {
        Entity::find()
            .filter(Column::EditionId.eq(edition_id))
            .filter(Column::Username.is_in(usernames))
            .all(db)
            .await
            .map_err(AppError::from)
    }
}
