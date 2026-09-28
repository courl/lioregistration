use crate::entity::admin_user;
use crate::entity::admin_user::UserRole;
use crate::error::AppResult;
use crate::middleware::auth::Auth;
use crate::services;
use crate::state::AppState;
use crate::utils::token_generator::TOKEN_VALIDITY;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
    pub role: String,
}

impl From<admin_user::Model> for UserResponse {
    fn from(user: admin_user::Model) -> Self {
        Self {
            email: user.email,
            name: user.name,
            role: match user.role {
                UserRole::Admin => "admin",
                UserRole::Observer => "observer",
            }
            .to_string(),
        }
    }
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
    let user = services::auth::try_login(&body.email, &body.password, &state.db).await?;
    let token = state.token_generator.user_to_token(&user)?;

    let cookie = Cookie::build(("access_token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::None) // todo: fixme
        .max_age(TOKEN_VALIDITY)
        .secure(true)
        .partitioned(true)
        .build();

    Ok((jar.add(cookie), Json(user.into())))
}

pub async fn me(auth: Auth) -> AppResult<Json<UserResponse>> {
    Ok(Json(auth.user.into()))
}

pub async fn logout(jar: CookieJar) -> (CookieJar, StatusCode) {
    let jar = jar.remove(Cookie::build("access_token").path("/").build());
    (jar, StatusCode::OK)
}
