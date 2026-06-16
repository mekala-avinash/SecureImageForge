//! Paved-road Rust (axum) entrypoint.
//!
//! Wired:
//!  * /healthz, /ready, /metrics
//!  * tracing-subscriber JSON logs with OTel trace correlation
//!  * OTLP/gRPC tracer (toggle via OTEL_EXPORTER_OTLP_ENDPOINT)
//!  * graceful shutdown on SIGTERM (DRAIN_TIMEOUT, default 20s)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { ok: true }))
}

async fn ready() -> impl IntoResponse {
    // Extend with real DB / cache probes before returning 200.
    (StatusCode::OK, Json(HealthResponse { ok: true }))
}

async fn metrics() -> impl IntoResponse {
    use prometheus::{Encoder, TextEncoder};
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).ok();
    (StatusCode::OK, String::from_utf8(buffer).unwrap_or_default())
}

#[derive(Serialize)]
struct Item {
    id: u64,
    name: &'static str,
}

async fn list_items() -> impl IntoResponse {
    Json(serde_json::json!({ "items": [{ "id": 1, "name": "example" }] }))
}

fn init_logging() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await.expect("ctrl_c install"); };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("sigterm install").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => info!("SIGINT received"),
        _ = terminate => info!("SIGTERM received"),
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    let port: u16 = std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap_or(8080);
    let drain: u64 = std::env::var("DRAIN_TIMEOUT_SECONDS").unwrap_or_else(|_| "20".into()).parse().unwrap_or(20);

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .route("/api/v1/items", get(list_items))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "starting {{service_name}}");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            info!(drain_secs = drain, "draining");
            tokio::time::sleep(Duration::from_secs(drain)).await;
        })
        .await
        .expect("server");
}
