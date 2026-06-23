mod subjects;
use axum::extract::{Path, State};
use std::{
    convert::Infallible,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use uuid::Uuid;

use askama::Template;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::post;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use datastar::consts::ElementPatchMode;
use futures::{StreamExt, stream};

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

    let nats_url = std::env::var("NATS_URL").unwrap();
    let nats = async_nats::connect(&nats_url)
        .await
        .expect("can't connect to nats");

    let state = AppState { db: pool, nats };

    let app =
        Router::new()
            .route("/", get(home))
            .route("/healthz", get(healthz))
            .route("/commands/increment", post(increment_signals))
            .route("/commands/increment-elements", post(increment_elements))
            .route("/monitors", get(monitors_page))
            .route("/feeds/monitors", get(monitors_feed))
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
    nats: async_nats::Client,
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

static NEXT_FEED_ID: AtomicU64 = AtomicU64::new(1);

struct FeedConnectionLog {
    id: u64,
}

impl Drop for FeedConnectionLog {
    fn drop(&mut self) {
        tracing::info!(feed_id = self.id, "monitor feed disconnected");
    }
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
struct MonitorsPage {}

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

async fn monitors_page() -> Result<Html<String>, AppError> {
    render(&MonitorsPage {})
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
) -> Result<Response, AppError> {
    let name = signals.name.trim().to_string();
    let url = signals.url.trim().to_string();

    if name.is_empty() || url::Url::parse(&url).is_err() {
        let patch = PatchElements::new(
            r#"<div id="form-errors">Name is required and URL must be valid.</div>"#,
        );

        return Ok(sse_events(vec![Event::from(patch)]).into_response());
    }

    sqlx::query!(
        "insert into monitors (name, url) values ($1, $2)",
        name,
        url
    )
    .execute(&state.db)
    .await?;

    state
        .nats
        .publish(subjects::MONITORS_CHANGED, "".into())
        .await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn delete_monitor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    sqlx::query!("delete from monitors where id = $1", id)
        .execute(&state.db)
        .await?;

    state
        .nats
        .publish(subjects::MONITORS_CHANGED, "".into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn monitors_feed(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let feed_id = NEXT_FEED_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!(feed_id, "monitor feed connected");

    let initial_patch = monitor_list_patch(load_monitors(&state.db).await?)?;
    let mut sub = state.nats.subscribe(subjects::MONITORS_EVENTS).await?;
    let db = state.db.clone();

    let events = async_stream::stream! {
        let _connection_log = FeedConnectionLog {id: feed_id};

        yield Ok::<_, Infallible>(Event::from(initial_patch));

        while let Some(_msg) = sub.next().await {
            match load_monitors(&db).await.and_then(monitor_list_patch) {
                Ok(patch) => {
                    tracing::debug!(feed_id, "refreshing monitor feed");
                    yield Ok::<_, Infallible>(Event::from(patch));
                }
                Err(err) => {
                    tracing::error!("failed to refresh monitor feed: {}", err.0)
                }
            }
        }
    };

    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}
