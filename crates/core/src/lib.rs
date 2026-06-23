use std::time::Duration;

use axum::{http::StatusCode, response::IntoResponse};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub nats_url: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct Ctx {
    pub db: PgPool,
    pub nats: async_nats::Client,
    pub cfg: Config,
}

pub async fn bootstrap() -> anyhow::Result<Ctx> {
    let cfg = Config {
        database_url: std::env::var("DATABASE_URL")?,
        nats_url: std::env::var("NATS_URL")?,
        port: std::env::var("PORT")?.parse()?,
    };

    let db = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&cfg.database_url)
        .await?;

    let nats = async_nats::connect(&cfg.nats_url).await?;

    Ok(Ctx { db, nats, cfg })
}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
