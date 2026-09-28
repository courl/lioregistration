use crate::settings::SmtpSettings;
use crate::utils::rate_limiter::RateLimiter;
use crate::utils::token_generator::LoginTokenGenerator;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub smtp: Arc<SmtpSettings>,
    pub admin_email: Option<String>,
    pub rate_limiter: Arc<RateLimiter>,
    pub token_generator: Arc<LoginTokenGenerator>,
}
