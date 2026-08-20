use anyhow::{Context, Result};
use axum::{extract::Request, http::HeaderName};
use server::{config::Config, router::build_app};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{
    LatencyUnit,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnEos, DefaultOnResponse, TraceLayer},
};
use tracing::info_span;
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_FILTER: &str = concat!(
    env!("CARGO_CRATE_NAME"),
    "=debug,tower_http=debug,axum=trace"
);
const REQUEST_ID_HEADER: &str = "x-request-id";

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .context("failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal = signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("failed to install termination signal handler")?;
        signal.recv().await;
        anyhow::Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<()>>();

    let result = tokio::select! {
        result = ctrl_c => result,
        result = terminate => result,
    };

    if let Err(error) = result {
        tracing::error!(?error, "failed to listen for shutdown signal");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER)),
        )
        .init();

    let listener = TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| "failed to bind HTTP listener")?;
    let local_address = listener
        .local_addr()
        .context("failed to read listener address")?;

    tracing::info!(url = %format_args!("http://{local_address}/"), "server is ready");

    let x_request_id = HeaderName::from_static(REQUEST_ID_HEADER);
    let tracing_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = request.headers().get(REQUEST_ID_HEADER);
                    request_id.map_or_else(
                        || {
                            tracing::error!("could not extract request_id");
                            info_span!(
                                "request",
                                method = %request.method(),
                                uri = %request.uri().path(),
                            )
                        },
                        |request_id| {
                            info_span!(
                                "request",
                                request_id = ?request_id,
                                method = %request.method(),
                                uri = %request.uri().path(),
                            )
                        },
                    )
                })
                .on_response(DefaultOnResponse::new().latency_unit(LatencyUnit::Micros))
                .on_eos(DefaultOnEos::new().latency_unit(LatencyUnit::Micros)),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let app = build_app().layer(tracing_middleware);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server stopped unexpectedly")
}
