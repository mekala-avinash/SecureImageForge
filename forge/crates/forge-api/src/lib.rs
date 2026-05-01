//! Local-mode HTTP API. Surfaces builds, scans, SBOMs, audit, and RBAC over
//! axum so CI integrations and the desktop daemon can drive forge-core
//! without invoking subprocess tools directly.

pub mod auth;
pub mod error;
pub mod metrics;
pub mod openapi;
pub mod routes;
pub mod state;
pub mod worker;

use std::time::Instant;

use axum::extract::MatchedPath;
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub use state::{ApiState, make_scanner};

pub fn router(state: ApiState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health))
        .route("/metrics", get(routes::metrics))
        .route(
            "/v1/builds",
            get(routes::list_builds).post(routes::create_build),
        )
        .route("/v1/builds/:id", get(routes::get_build))
        .route(
            "/v1/builds/:id/start",
            axum::routing::post(routes::start_build),
        )
        .route(
            "/v1/projects/:project_id/builds",
            get(routes::list_builds_in_project).post(routes::create_build_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id",
            get(routes::get_build_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/start",
            axum::routing::post(routes::start_build_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/scan",
            get(routes::get_scan_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/sbom",
            get(routes::get_sbom_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/log",
            get(routes::get_log_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/log/stream",
            get(routes::stream_log_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/provenance",
            get(routes::get_provenance_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/drift",
            get(routes::list_drift_in_project),
        )
        .route(
            "/v1/projects/:project_id/builds/:id/cancel",
            axum::routing::post(routes::cancel_build_in_project),
        )
        .route(
            "/v1/projects/:project_id/jobs",
            get(routes::list_jobs_in_project),
        )
        .route("/v1/builds/:id/scan", get(routes::get_scan))
        .route("/v1/builds/:id/sbom", get(routes::get_sbom))
        .route("/v1/builds/:id/log", get(routes::get_log))
        .route("/v1/builds/:id/log/stream", get(routes::stream_log))
        .route("/v1/builds/:id/provenance", get(routes::get_provenance))
        .route("/v1/builds/:id/drift", get(routes::list_drift))
        .route("/v1/audit", get(routes::list_audit))
        .route(
            "/v1/principals",
            get(routes::list_principals).post(routes::create_principal),
        )
        .route(
            "/v1/principals/:id",
            axum::routing::delete(routes::revoke_principal),
        )
        .route("/v1/auth/config", get(routes::get_auth_config))
        .route(
            "/v1/rbac/bindings",
            get(routes::list_rbac_bindings).post(routes::create_rbac_binding),
        )
        .route(
            "/v1/scopes",
            get(routes::list_scope_grants).post(routes::create_scope_grant),
        )
        .route("/v1/openapi.json", get(routes::openapi))
        .layer(middleware::from_fn(observe_request))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        // Enforce a global rate limit: max 50 concurrent requests.
        .layer(tower::limit::GlobalConcurrencyLimitLayer::new(50))
        .with_state(state)
}

pub async fn serve(state: ApiState, addr: std::net::SocketAddr) -> anyhow::Result<()> {
    forge_core::telemetry::init_with_endpoint(
        state.config.telemetry.otlp_endpoint.as_deref(),
        state
            .config
            .telemetry
            .service_name
            .as_deref()
            .or(Some("forge-api")),
    );
    metrics::install_recorder();
    let _drift_scheduler = state.start_drift_scheduler();
    let _workers = worker::start_workers(state.clone());
    let app = router(state);
    tracing::info!(%addr, "forge-api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn observe_request(req: Request<axum::body::Body>, next: Next) -> Response {
    let started = Instant::now();
    let method = req.method().clone();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| "<unmatched>".to_string());
    let resp = next.run(req).await;
    let duration = started.elapsed().as_secs_f64();
    let status = resp.status().as_u16().to_string();
    ::metrics::counter!(
        "forge_api_requests_total",
        "method" => method.to_string(),
        "route" => route.clone(),
        "status" => status.clone(),
    )
    .increment(1);
    ::metrics::histogram!(
        "forge_api_request_duration_seconds",
        "method" => method.to_string(),
        "route" => route,
        "status" => status,
    )
    .record(duration);
    resp
}

// Re-export the imperative one-shot used by `forge-cli serve`.
pub use serve as run;
