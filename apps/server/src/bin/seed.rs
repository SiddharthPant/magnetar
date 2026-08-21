use anyhow::Context;
use server::{auth::password::hash_password, config::Config, utils::nanoid::prefixed_nanoid};
use sqlx::{PgPool, postgres::PgPoolOptions, query_as};
use time::OffsetDateTime;
use uuid::Uuid;

async fn seed_user(pool: &PgPool, password: &str) -> anyhow::Result<()> {
    let password_hash = hash_password(password)?;
    let id = Uuid::parse_str("01a02165-7ef5-77dd-8436-351807373dd3")?;

    let mut transaction = pool.begin().await?;

    let (created_at, updated_at): (OffsetDateTime, OffsetDateTime) = query_as(
        "
            INSERT INTO users (
                id,
                pid,
                full_name,
                email,
                password_hash
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
                full_name = excluded.full_name,
                email = excluded.email,
                password_hash = excluded.password_hash,
                updated_at = now()
            RETURNING created_at, updated_at
        ",
    )
    .bind(id)
    .bind(prefixed_nanoid("usr", 16, true))
    .bind("Development Admin")
    .bind("admin@example.test")
    .bind(password_hash)
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;

    println!("seeded user: created={created_at}, updated={updated_at}");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    seed_user(&pool, &config.seed_password).await
}
