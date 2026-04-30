//! Local-mode HTTP API. Surfaces the same operations as the CLI for use by
//! CI integrations, the desktop app's daemon mode, and remote tools.

use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
}

pub fn router() -> Router {
    Router::new().route("/healthz", get(health))
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}
