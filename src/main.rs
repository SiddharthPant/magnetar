use axum::extract::{Path, State};
use std::{convert::Infallible, time::Duration};
use uuid::Uuid;

use askama::Template;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use datastar::consts::ElementPatchMode;
use futures::stream;

use datastar::patch_elements::PatchElements;
use datastar::{axum::ReadSignals, patch_signals::PatchSignals};
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    services::ServeDir,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let db_connection_str = std::env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let state = AppState { db: pool };

    let app =
        Router::new()
            .route("/", get(home))
            .route("/healthz", get(healthz))
            .route("/commands/increment", post(increment_signals))
            .route("/commands/increment-elements", post(increment_elements))
            .route("/monitors", get(monitors_page))
            .route("/commands/monitors/create", post(create_monitor))
            .route("/commands/monitors/{id}/delete", post(delete_monitor))
            .nest_service("/assets", ServeDir::new("assets"))
            .layer(TraceLayer::new_for_http().on_response(
                DefaultOnResponse::new().latency_unit(tower_http::LatencyUnit::Micros),
            ))
            .with_state(state);

    let port: u16 = std::env::var("PORT").unwrap().parse().unwrap();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("failed to bind on given host/port");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

struct AppError(anyhow::Error);

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

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

fn render<T: Template>(t: &T) -> Result<Html<String>, AppError> {
    t.render().map(Html).map_err(|e| AppError(e.into()))
}

fn sse_events(events: Vec<Event>) -> impl IntoResponse {
    Sse::new(stream::iter(events.into_iter().map(Ok::<_, Infallible>)))
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomePage {
    count: i64,
}

#[derive(Deserialize)]
struct CounterSignals {
    count: i64,
}

#[derive(Template)]
#[template(path = "fragments/count.html")]
struct CountFragment {
    count: i64,
}

#[derive(Debug, Clone)]
struct Monitor {
    id: uuid::Uuid,
    name: String,
    url: String,
    interval_seconds: i32,
    created_at: time::OffsetDateTime,
    updated_at: time::OffsetDateTime,
}

#[derive(Template)]
#[template(path = "pages/monitors.html")]
struct MonitorsPage {
    monitors: Vec<Monitor>,
}

#[derive(Template)]
#[template(path = "fragments/monitor_list.html")]
struct MonitorListFragment {
    monitors: Vec<Monitor>,
}

#[derive(Deserialize)]
struct MonitorSignals {
    name: String,
    url: String,
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn home() -> Result<Html<String>, AppError> {
    render(&HomePage { count: 0 })
}

async fn increment_signals(
    ReadSignals(signals): ReadSignals<CounterSignals>,
) -> Result<impl IntoResponse, AppError> {
    let new_count = signals.count + 1;
    let patch = PatchSignals::new(format!(r#"{{"count":{new_count}}}"#));
    Ok(sse_events(vec![Event::from(patch)]))
}

async fn increment_elements(
    ReadSignals(signals): ReadSignals<CounterSignals>,
) -> Result<impl IntoResponse, AppError> {
    let new_count = signals.count + 1;
    let html = render(&CountFragment { count: new_count })?;
    Ok(sse_events(vec![
        Event::from(PatchSignals::new(format!(r#"{{"count":{new_count}}}"#))),
        Event::from(PatchElements::new(html.0)),
    ]))
}

async fn load_monitors(db: &PgPool) -> Result<Vec<Monitor>, AppError> {
    let monitors = sqlx::query_as!(
        Monitor,
        r#"select id, name, url, interval_seconds, created_at, updated_at from monitors order by created_at desc"#
    )
    .fetch_all(db)
    .await?;

    Ok(monitors)
}

async fn monitors_page(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let monitors = load_monitors(&state.db).await?;
    render(&MonitorsPage { monitors })
}

fn monitor_list_patch(monitors: Vec<Monitor>) -> Result<PatchElements, AppError> {
    let html = render(&MonitorListFragment { monitors })?;

    Ok(PatchElements::new(html.0)
        .selector("#monitor-list")
        .mode(ElementPatchMode::Inner))
}

async fn create_monitor(
    State(state): State<AppState>,
    ReadSignals(signals): ReadSignals<MonitorSignals>,
) -> Result<impl IntoResponse, AppError> {
    let name = signals.name.trim().to_string();
    let url = signals.url.trim().to_string();

    if name.is_empty() || url::Url::parse(&url).is_err() {
        let patch = PatchElements::new(
            r#"<div id="form-errors">Name is required and URL must be valid.</div>"#,
        );

        return Ok(sse_events(vec![Event::from(patch)]));
    }

    sqlx::query!(
        "insert into monitors (name, url) values ($1, $2)",
        name,
        url
    )
    .execute(&state.db)
    .await?;

    let monitors = load_monitors(&state.db).await?;
    let list_patch = monitor_list_patch(monitors)?;
    let clear_signals = PatchSignals::new(r#"{"name": "", "url": ""}"#);
    let clear_errors = PatchElements::new(r#"<div id="form-errors"></div>"#);

    Ok(sse_events(vec![
        Event::from(list_patch),
        Event::from(clear_signals),
        Event::from(clear_errors),
    ]))
}

async fn delete_monitor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!("delete from monitors where id = $1", id)
        .execute(&state.db)
        .await?;
    let monitors = load_monitors(&state.db).await?;
    let patch = monitor_list_patch(monitors)?;

    Ok(sse_events(vec![Event::from(patch)]))
}
