mod router;
use router::router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let ctx = magnetar_core::bootstrap().await?;

    let app = router(ctx.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", ctx.cfg.port)).await?;

    tracing::info!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
