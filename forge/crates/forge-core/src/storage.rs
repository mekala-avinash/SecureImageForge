//! SQLite-backed persistence. Migrations live in `migrations/` and are run on
//! every `Storage::open` via the embedded migrator.
//!
//! Phase 6 introduces `Backend::detect` so a single `database_url` config
//! string can route to SQLite (single-user desktop) or Postgres (team mode).
//! The Postgres CRUD adapters land in 6.5; this commit only wires the
//! discriminator and a clear error when the operator points at Postgres.

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

use crate::Result;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    /// Open or create the database at `path`. Runs all pending migrations.
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let url = format!("sqlite://{}", path.display());
        let opts = SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(opts)
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    /// In-memory storage for tests.
    pub async fn open_memory() -> Result<Self> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?.foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Backend selected from a connection string. SQLite is the only fully
/// supported backend in Phase 6; Postgres is recognized so configuration is
/// stable across the upcoming Phase 6.5 work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Sqlite,
    Postgres,
}

impl Backend {
    /// Map a URL to a backend. Empty/`None` defaults to SQLite to keep the
    /// single-user happy path zero-config.
    pub fn detect(url: Option<&str>) -> Self {
        match url {
            Some(u) if u.starts_with("postgres://") || u.starts_with("postgresql://") => {
                Backend::Postgres
            }
            _ => Backend::Sqlite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Backend;

    #[test]
    fn detect_defaults_to_sqlite() {
        assert_eq!(Backend::detect(None), Backend::Sqlite);
        assert_eq!(Backend::detect(Some("sqlite:///tmp/x.db")), Backend::Sqlite);
        assert_eq!(Backend::detect(Some("file:./x.db")), Backend::Sqlite);
    }

    #[test]
    fn detect_recognizes_postgres_schemes() {
        assert_eq!(
            Backend::detect(Some("postgres://u:p@h/db")),
            Backend::Postgres
        );
        assert_eq!(
            Backend::detect(Some("postgresql://h/db")),
            Backend::Postgres
        );
    }
}
