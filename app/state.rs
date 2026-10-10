use crate::services::email_attachment::EmailAttachmentContext;
use crate::utils::rate_limiter::RateLimiter;
use crate::utils::token_generator::LoginTokenGenerator;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    /// Applied to every request by the `global_rate_limit` middleware.
    pub rate_limiter: Arc<RateLimiter>,
    /// Applied to sensitive endpoints via the `StrictRateLimit` extractor.
    pub strict_rate_limiter: Arc<RateLimiter>,
    pub trust_proxy: bool,
    pub token_generator: Arc<LoginTokenGenerator>,
    pub email_attachment_ctx: Arc<EmailAttachmentContext>,
}
