use axum::{Router, http::StatusCode, response::Html, routing::get};

use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let app = Router::new()
        .route("/", get(home))
        .route("/healthz", get(healthz));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn home() -> Html<&'static str> {
    Html("<h1>magnetar online</h1>")
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}
