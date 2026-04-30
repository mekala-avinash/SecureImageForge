use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("storage error: {0}")]
    Storage(#[from] sqlx::Error),

    #[error("storage migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("required tool not found on PATH or bundled prefix: {tool}")]
    ToolMissing { tool: String },

    #[error("external tool '{tool}' failed (exit {code}): {stderr}")]
    ToolFailure {
        tool: String,
        code: i32,
        stderr: String,
    },

    #[error("invalid build spec: {0}")]
    InvalidSpec(String),

    #[error("policy violation: {0}")]
    PolicyViolation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
}
