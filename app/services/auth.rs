use crate::entity::admin_user;
use crate::error::{AppError, AppResult};
use sea_orm::ConnectionTrait;

pub async fn try_login(
    email: &str,
    password: &str,
    db: &impl ConnectionTrait,
) -> AppResult<admin_user::Model> {
    let user = admin_user::Entity::find_by_email(email)
        .one(db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    user.verify_password(password)?;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::tests::setup_test;
    use std::assert_matches;

    #[tokio::test]
    async fn login_with_user() {
        let app = setup_test().await.with_admin("test@test.com", "123").await;

        assert!(
            try_login("test@test.com", "123", &app.state.db)
                .await
                .is_ok()
        );
        assert_matches!(
            try_login("test@test.com", "1234", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
        assert_matches!(
            try_login("test@tes.com", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
        assert_matches!(
            try_login("", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
        assert_matches!(
            try_login("*", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
    }

    #[tokio::test]
    async fn login_without_user() {
        let app = setup_test().await;

        assert_matches!(
            try_login("test@test.com", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
        assert_matches!(
            try_login("", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
        assert_matches!(
            try_login("*", "123", &app.state.db).await,
            Err(AppError::Unauthorized)
        );
    }
}
