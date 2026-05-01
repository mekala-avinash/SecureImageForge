//! Append-only audit log. Every privileged action (build start, policy
//! decision, registry push, principal mutation) goes through here.

use chrono::Utc;
use serde::Serialize;

use crate::storage::Storage;
use crate::Result;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Success,
    Denied,
    Error,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Success => "success",
            Outcome::Denied => "denied",
            Outcome::Error => "error",
        }
    }
}

#[async_trait::async_trait]
pub trait AuditLog: Send + Sync {
    async fn record(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        outcome: Outcome,
        details: Option<serde_json::Value>,
    ) -> Result<()>;

    async fn recent(&self, limit: i64) -> Result<Vec<AuditEntry>>;
}

#[derive(Clone)]
pub struct SqliteAuditLog {
    storage: Storage,
}

impl SqliteAuditLog {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl AuditLog for SqliteAuditLog {
    async fn record(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        outcome: Outcome,
        details: Option<serde_json::Value>,
    ) -> Result<()> {
        let details = details
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into()));
        sqlx::query(
            r#"INSERT INTO audit_events (actor, action, target, outcome, details, created_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(actor)
        .bind(action)
        .bind(target)
        .bind(outcome.as_str())
        .bind(details)
        .bind(Utc::now().to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn recent(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        use sqlx::Row;
        let rows = sqlx::query(
            r#"SELECT id, actor, action, target, outcome, details, created_at
               FROM audit_events ORDER BY id DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| AuditEntry {
                id: r.get::<i64, _>("id"),
                actor: r.get("actor"),
                action: r.get("action"),
                target: r.get("target"),
                outcome: r.get("outcome"),
                details: r.get("details"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub actor: String,
    pub action: String,
    pub target: Option<String>,
    pub outcome: String,
    pub details: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn record_and_recent_round_trip() {
        let storage = Storage::open_memory().await.unwrap();
        let log = SqliteAuditLog::new(storage);
        log.record("admin", "build.start", Some("foo"), Outcome::Success, None)
            .await
            .unwrap();
        log.record("admin", "policy.deny", Some("foo"), Outcome::Denied, None)
            .await
            .unwrap();
        let recent = log.recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].action, "policy.deny");
        assert_eq!(recent[0].outcome, "denied");
    }
}
