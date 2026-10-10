use std::path::PathBuf;

use anyhow::Result;
use config::Config;
use serde::Deserialize;

fn default_host() -> String {
    "0.0.0.0".into()
}
const fn default_port() -> u16 {
    3001
}
fn default_database_url() -> String {
    "sqlite://./lio.db?mode=rwc".into()
}
pub const DEV_SECRET_KEY: &str = "2026-06-05-dev-secret-key-change-in-production";

fn default_secret_key() -> String {
    DEV_SECRET_KEY.into()
}
fn default_assets_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
}

const fn default_global_max_requests() -> usize {
    300
}
const fn default_global_window_secs() -> u64 {
    60
}
const fn default_strict_max_requests() -> usize {
    5
}
const fn default_strict_window_secs() -> u64 {
    4 * 60 * 60
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitSettings {
    /// Limit applied to every request, per client IP.
    #[serde(default = "default_global_max_requests")]
    pub global_max_requests: usize,
    #[serde(default = "default_global_window_secs")]
    pub global_window_secs: u64,
    /// Limit applied to sensitive endpoints (login, register), per client IP.
    #[serde(default = "default_strict_max_requests")]
    pub strict_max_requests: usize,
    #[serde(default = "default_strict_window_secs")]
    pub strict_window_secs: u64,
    /// Take the client IP from `X-Forwarded-For`. Only enable behind a reverse proxy
    /// that sets this header, otherwise clients can spoof their IP.
    #[serde(default)]
    pub trust_proxy: bool,
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            global_max_requests: default_global_max_requests(),
            global_window_secs: default_global_window_secs(),
            strict_max_requests: default_strict_max_requests(),
            strict_window_secs: default_strict_window_secs(),
            trust_proxy: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    pub implicit_tls: bool,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_secret_key")]
    pub secret_key: String,
    #[serde(default)]
    pub smtp: Option<SmtpSettings>,
    pub admin_email: String,
    #[serde(default = "default_assets_root")]
    pub assets_root: PathBuf,
    #[serde(default)]
    pub rate_limit: RateLimitSettings,
    /// Directory of the compiled SPA (`frontend_lio/dist`). When set, every non-API path is
    /// served from it, making frontend and API same-origin. Unset: API only.
    #[serde(default)]
    pub frontend_dir: Option<PathBuf>,
    /// Extra browser origins allowed to call the API with credentials. Only needed when the
    /// frontend is hosted elsewhere; empty disables CORS. Non-empty also switches the auth
    /// cookie from `SameSite=Lax` to `None` + partitioned.
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let settings = Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("LIO").separator("__"))
            .build()?;
        Ok(settings.try_deserialize()?)
    }
}
