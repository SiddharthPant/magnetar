use axum::{Router, http::StatusCode, routing::get};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
}
async fn index() -> &'static str {
    "Hello world!"
}

async fn health() -> StatusCode {
    StatusCode::OK
}
