use std::{
    convert::Infallible,
    sync::atomic::{AtomicU64, Ordering},
};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        Html, IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use datastar::{axum::ReadSignals, patch_elements::PatchElements, patch_signals::PatchSignals};
use futures::{StreamExt, stream};
use magnetar_core::{AppError, Ctx};
use serde::Deserialize;
use tower_http::{
    services::ServeDir,
    trace::{DefaultOnResponse, TraceLayer},
};
use uuid::Uuid;

static NEXT_FEED_ID: AtomicU64 = AtomicU64::new(1);

pub fn router(ctx: Ctx) -> Router {
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
        .layer(
            TraceLayer::new_for_http().on_response(
                DefaultOnResponse::new().latency_unit(tower_http::LatencyUnit::Micros),
            ),
        )
        .with_state(ctx)
}

#[derive(Deserialize)]
struct CounterSignals {
    count: i64,
}

#[derive(Deserialize)]
struct MonitorSignals {
    name: String,
    url: String,
}

struct FeedConnectionLog {
    id: u64,
}

impl Drop for FeedConnectionLog {
    fn drop(&mut self) {
        tracing::info!(feed_id = self.id, "monitor feed disconnected");
    }
}

fn sse_events(events: Vec<Event>) -> impl IntoResponse {
    Sse::new(stream::iter(events.into_iter().map(Ok::<_, Infallible>)))
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn home() -> Result<Html<String>, AppError> {
    Ok(magnetar_web::home_page(0)?)
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
    let html = magnetar_web::count_fragment(new_count)?;

    Ok(sse_events(vec![
        Event::from(PatchSignals::new(format!(r#"{{"count":{new_count}}}"#))),
        Event::from(PatchElements::new(html.0)),
    ]))
}

async fn monitors_page() -> Result<Html<String>, AppError> {
    Ok(magnetar_web::monitors_page()?)
}

async fn create_monitor(
    State(ctx): State<Ctx>,
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

    magnetar_db::create_monitor(&ctx.db, &name, &url).await?;
    magnetar_bus::publish_monitors_changed(&ctx.nats).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn delete_monitor(
    State(ctx): State<Ctx>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    magnetar_db::delete_monitor(&ctx.db, id).await?;
    magnetar_bus::publish_monitors_changed(&ctx.nats).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn monitors_feed(State(ctx): State<Ctx>) -> Result<impl IntoResponse, AppError> {
    let feed_id = NEXT_FEED_ID.fetch_add(1, Ordering::Relaxed);
    tracing::info!(feed_id, "monitor feed connected");

    let initial_patch =
        magnetar_web::monitor_list_patch(magnetar_db::load_monitors(&ctx.db).await?)?;

    let mut sub = ctx.nats.subscribe(magnetar_bus::MONITORS_EVENTS).await?;
    let db = ctx.db.clone();

    let events = async_stream::stream! {
        let _connection_log = FeedConnectionLog { id: feed_id };

        yield Ok::<_, Infallible>(Event::from(initial_patch));

        while let Some(_msg) = sub.next().await {
            match magnetar_db::load_monitors(&db)
                .await
                .and_then(magnetar_web::monitor_list_patch)
            {
                Ok(patch) => {
                    tracing::debug!(feed_id, "refreshing monitor feed");
                    yield Ok::<_, Infallible>(Event::from(patch));
                }
                Err(err) => {
                    tracing::error!("failed to refresh monitor feed: {err}");
                }
            }
        }
    };

    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}
