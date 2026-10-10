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
fn default_secret_key() -> String {
    "2026-06-05-dev-secret-key-change-in-production".into()
}
fn default_assets_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
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
