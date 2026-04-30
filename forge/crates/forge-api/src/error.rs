use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<forge_core::Error> for ApiError {
    fn from(e: forge_core::Error) -> Self {
        match e {
            forge_core::Error::NotFound(_) => ApiError::NotFound,
            forge_core::Error::PolicyViolation(_) => ApiError::Forbidden,
            forge_core::Error::InvalidSpec(s) => ApiError::BadRequest(s),
            other => ApiError::Internal(anyhow::anyhow!(other)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            ApiError::BadRequest(s) => (StatusCode::BAD_REQUEST, s.clone()),
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        };
        (status, Json(json!({"error": msg}))).into_response()
    }
}
