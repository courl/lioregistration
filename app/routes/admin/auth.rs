use crate::entity::admin_user;
use crate::entity::admin_user::UserRole;
use crate::error::AppResult;
use crate::middleware::auth::Auth;
use crate::middleware::rate_limit::StrictRateLimit;
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
    _rl: StrictRateLimit,
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
    let user = services::auth::try_login(&body.email, &body.password, &state.db).await?;
    let token = state.token_generator.user_to_token(&user)?;

    let cookie = access_token_cookie(token, state.cross_site_frontend);
    Ok((jar.add(cookie), Json(user.into())))
}

pub async fn me(auth: Auth) -> AppResult<Json<UserResponse>> {
    Ok(Json(auth.user.into()))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, StatusCode) {
    // The removal must repeat SameSite/Partitioned, else the browser keeps a partitioned cookie
    // (and rejects a Lax Set-Cookie on a cross-site response).
    let jar = jar.remove(access_token_cookie(
        String::new(),
        state.cross_site_frontend,
    ));
    (jar, StatusCode::OK)
}

/// The auth cookie. Same-origin SPA (served by this binary or via the Vite proxy): `Lax` keeps
/// it off cross-site subrequests. A frontend on another site (`cors_origins`) needs `None` +
/// partitioned, or the browser never sends it back.
fn access_token_cookie(token: String, cross_site: bool) -> Cookie<'static> {
    Cookie::build(("access_token", token))
        .path("/")
        .http_only(true)
        .max_age(TOKEN_VALIDITY)
        .secure(true)
        .same_site(if cross_site {
            SameSite::None
        } else {
            SameSite::Lax
        })
        .partitioned(cross_site)
        .build()
}
