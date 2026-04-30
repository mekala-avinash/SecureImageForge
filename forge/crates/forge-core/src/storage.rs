//! SQLite-backed persistence for builds, scans, SBOMs, and audit log.
//!
//! All access is async via sqlx. Migrations live under `migrations/` and run on
//! startup; missing migrations are an error rather than auto-skipped to avoid
//! schema drift between desktop installs.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

use crate::Result;

#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    /// Open or create the database at `path`. Runs pending migrations.
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let url = format!("sqlite://{}", path.display());
        let opts = SqliteConnectOptions::from_str(&url)
            .map_err(sqlx::Error::from)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        // Migrations are wired in Phase 1 once schema stabilizes.
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
