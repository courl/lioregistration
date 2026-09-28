mod cli;
pub mod entity;
mod error;
mod jobs;
pub mod middleware;
mod routes;
mod services;
mod settings;
mod state;
pub mod utils;

use crate::state::AppState;
use crate::utils::{rate_limiter::RateLimiter, token_generator::LoginTokenGenerator};
use sea_orm::Database;
use settings::Settings;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //todo: remove debug/info switches
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lio=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::load()?;
    tracing::info!("Connecting to database: {}", settings.database_url);
    let db = &Database::connect(&settings.database_url).await?;
    tracing::info!("Running migrations");
    let module = format!("{}::entity::*", module_path!().split("::").next().unwrap());
    tracing::info!(module);
    db.get_schema_registry(&module).sync(db).await?;

    let config = Arc::new(settings.clone());

    let state = AppState {
        db: db.clone(),
        smtp: Arc::new(config.smtp.clone()),
        admin_email: settings.admin_email.clone(),
        rate_limiter: Arc::new(RateLimiter::new(5, Duration::from_hours(4))),
        token_generator: Arc::new(LoginTokenGenerator::from_secret(
            &settings.secret_key,
            db.clone(),
        )),
    };

    cli::run_command(settings, state).await
}
