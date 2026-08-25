use anyhow::Context;
use server::{auth::password::hash_password, config::Config, utils::nanoid::prefixed_nanoid};
use sqlx::{PgPool, postgres::PgPoolOptions};
use time::OffsetDateTime;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
struct User {
    id: Uuid,
    pid: String,
    full_name: String,
    email: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy)]
struct UserFixture {
    name: &'static str,
    email: &'static str,
}

async fn seed_user(pool: &PgPool, password: &str) -> anyhow::Result<()> {
    let password_hash = hash_password(password)?;
    let user_fixtures: [UserFixture; 3] = [
        UserFixture {
            name: "Admin User",
            email: "admin@example.test",
        },
        UserFixture {
            name: "Test User",
            email: "test@example.test",
        },
        UserFixture {
            name: "Guest User",
            email: "guest@example.test",
        },
    ];

    let mut transaction = pool.begin().await?;
    for (i, user_data) in user_fixtures.iter().enumerate() {
        let user = sqlx::query_as!(
            User,
            r#"
            insert into users (
                id,
                pid,
                full_name,
                email,
                password_hash
            )
            values ($1, $2, $3, $4, $5)
            returning id, pid, full_name, email, created_at, updated_at
        "#,
            Uuid::now_v7(),
            prefixed_nanoid("usr", Some(i.try_into()?)),
            user_data.name,
            user_data.email,
            password_hash
        )
        .fetch_one(&mut *transaction)
        .await
        .context(format!("inserting failed for user_data: {user_data:?}"))?;

        println!("seeded user: {user:?}");
    }
    transaction.commit().await?;

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
