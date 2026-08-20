use anyhow::{Context, Result};
use std::{env, net::SocketAddr};

#[derive(Clone, Debug)]
pub struct Config {
    pub app_display_name: String,
    pub bind_addr: SocketAddr,
    pub database_url: String,
}

fn env_get(name: &str, default: &str) -> Result<String> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    anyhow::ensure!(
        !value.trim().is_empty(),
        format!("{name} must not be empty")
    );
    Ok(value)
}

impl Config {
    /// # Errors
    ///
    /// Will error out if config invalid
    pub fn from_env() -> Result<Self> {
        let app_display_name = env_get("APP_DISPLAY_NAME", "App")?;
        let host = env_get("APP_HOST", "127.0.0.1")?;
        let port = env_get("APP_PORT", "3000")?
            .parse::<u16>()
            .context("APP_PORT must be a valid port number")?;
        let bind_addr: SocketAddr = format!("{host}:{port}")
            .parse()
            .context("invalid APP_HOST or APP_PORT")?;
        let database_url = env_get("DATABASE_URL", "sqlite://.locals/data/life.db?mode=rwc")?;

        Ok(Self {
            app_display_name,
            bind_addr,
            database_url,
        })
    }
}
