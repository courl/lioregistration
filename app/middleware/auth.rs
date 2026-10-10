use crate::error::AppResult;
use crate::{entity::admin_user, error::AppError, state::AppState};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;

pub struct Auth {
    pub user: admin_user::Model,
}

impl Auth {
    pub fn verify(&self, roles: Vec<admin_user::UserRole>) -> AppResult<()> {
        if !roles.contains(&self.user.role) {
            Err(AppError::Forbidden)
        } else {
            Ok(())
        }
    }
}

impl<S> FromRequestParts<S> for Auth
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> AppResult<Self> {
        let app_state = AppState::from_ref(state);

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let access_token = jar
            .get("access_token")
            .map(|c| c.value().to_owned())
            .ok_or(AppError::Unauthorized)?;

        let user = app_state
            .token_generator
            .token_to_user(&access_token)
            .await?;

        Ok(Self { user })
    }
}
