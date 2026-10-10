#![allow(clippy::future_not_send)]
#![allow(clippy::derive_partial_eq_without_eq)]

use crate::error::{AppError, AppResult};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "camelCase"
)]
pub enum UserRole {
    Admin,
    Observer,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "admin_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: UserRole,
}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    pub fn create(email: &str, password: &str, name: &str, role: UserRole) -> AppResult<Self> {
        let mut user = Self {
            email: ActiveValue::Set(email.into()),
            name: ActiveValue::Set(name.into()),
            role: ActiveValue::Set(role),
            ..Default::default()
        };
        user.set_password(password)?;
        Ok(user)
    }

    pub fn set_password(&mut self, password: &str) -> AppResult<()> {
        let salt = SaltString::generate(&mut OsRng);
        self.password_hash = ActiveValue::Set(
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|_| AppError::Internal(anyhow::anyhow!("cannot hash")))?
                .to_string(),
        );

        Ok(())
    }
}

impl Model {
    pub fn verify_password(&self, password: &str) -> AppResult<()> {
        let parsed = PasswordHash::new(self.password_hash.as_ref())
            .map_err(|_| AppError::Internal(anyhow::anyhow!("invalid password hash")))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::TryIntoModel;

    #[test]
    fn check_password_generation() {
        let mut user = ActiveModel {
            id: ActiveValue::Set(1),
            name: ActiveValue::Set(String::from("test")),
            email: ActiveValue::Set("123".into()),
            role: ActiveValue::Set(UserRole::Admin),
            ..Default::default()
        };
        user.set_password("123").unwrap();
        assert!(user.password_hash.is_set());
        assert!(!user.password_hash.as_ref().is_empty());
        assert_ne!(user.password_hash.as_ref(), "123");

        let user = user.try_into_model().unwrap();
        assert!(user.verify_password("123").is_ok());
    }
}
