use askama::Template;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use futures::stream;

use datastar::patch_elements::PatchElements;
use datastar::{axum::ReadSignals, patch_signals::PatchSignals};
use serde::Deserialize;
use tower_http::{
    services::ServeDir,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, fmt};

struct AppError(anyhow::Error);

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let app =
        Router::new()
            .route("/", get(home))
            .route("/healthz", get(healthz))
            .route("/commands/increment", post(increment_signals))
            .route("/commands/increment-elements", post(increment_elements))
            .nest_service("/assets", ServeDir::new("assets"))
            .layer(TraceLayer::new_for_http().on_response(
                DefaultOnResponse::new().latency_unit(tower_http::LatencyUnit::Micros),
            ));

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

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
    }
}

fn render<T: Template>(t: &T) -> Result<Html<String>, AppError> {
    t.render().map(Html).map_err(|e| AppError(e.into()))
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomePage;

#[derive(Deserialize)]
struct CounterSignals {
    count: i64,
}

#[derive(Template)]
#[template(path = "fragments/count.html")]
struct CountFragment {
    count: i64,
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn home() -> Result<Html<String>, AppError> {
    render(&HomePage)
}

async fn increment_signals(
    ReadSignals(signals): ReadSignals<CounterSignals>,
) -> Result<impl IntoResponse, AppError> {
    let new_count = signals.count + 1;
    let patch = PatchSignals::new(format!(r#"{{"count":{new_count}}}"#));
    let body = Sse::new(stream::once(async {
        Ok::<_, std::convert::Infallible>(Event::from(patch))
    }));
    Ok(body)
}

async fn increment_elements(
    ReadSignals(signals): ReadSignals<CounterSignals>,
) -> Result<impl IntoResponse, AppError> {
    let new_count = signals.count + 1;
    let html = render(&CountFragment { count: new_count })?;
    let patch = PatchElements::new(html.0);
    let body = Sse::new(stream::iter([
        Ok::<_, std::convert::Infallible>(Event::from(PatchSignals::new(format!(
            r#"{{"count":{new_count}}}"#
        )))),
        Ok::<_, std::convert::Infallible>(Event::from(patch)),
    ]));
    Ok(body)
}
