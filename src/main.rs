use axum::{Router, http::StatusCode, response::Html, routing::get};

use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let app = Router::new()
        .route("/", get(home))
        .route("/healthz", get(healthz))
        .layer(
            TraceLayer::new_for_http().on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(tower_http::LatencyUnit::Micros),
            ),
        );

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on http://{}/", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind on given host/port");
    axum::serve(listener, app).await.unwrap()
}

async fn home() -> Html<&'static str> {
    Html("<h1>magnetar online</h1>")
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}
