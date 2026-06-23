use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub interval_seconds: i32,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

pub async fn load_monitors(db: &PgPool) -> anyhow::Result<Vec<Monitor>> {
    let monitors = sqlx::query_as!(Monitor, r#"select id, name, url, interval_seconds, created_at, updated_at from monitors order by created_at desc"#).fetch_all(db).await?;
    Ok(monitors)
}

pub async fn create_monitor(db: &PgPool, name: &str, url: &str) -> anyhow::Result<()> {
    sqlx::query!(
        "insert into monitors (name, url) values ($1, $2)",
        name,
        url
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_monitor(db: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!("delete from monitors where id = $1", id)
        .execute(db)
        .await?;

    Ok(())
}
