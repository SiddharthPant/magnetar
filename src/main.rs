use axum::{Router, response::Html, routing::get};

use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let app = Router::new().route("/", get(home));

    tracing::info!("Starting server on 0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn home() -> Html<&'static str> {
    Html("<h1>magnetar online</h1>")
}
