//! Outbound webhook event sink. Each configured endpoint receives a JSON
//! payload signed with HMAC-SHA256 of the body using the endpoint's secret;
//! the signature ships in `X-Forge-Signature: sha256=<hex>` so consumers can
//! verify authenticity without trusting transport-layer auth alone.
//!
//! Phase 6.5 adds persistent retries: failed deliveries are stashed in the
//! `webhook_deliveries` table with an exponential `next_attempt`, and a
//! background worker retries them with backoff (1s → 2s → 4s → … capped at
//! 30 minutes). Successful deliveries are marked `delivered_at`; consumers
//! can clean them up out-of-band or rely on the worker pruning entries
//! older than 7 days.

use std::sync::Arc;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::config::{WebhookEndpoint, WebhooksSection};
use crate::Result;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub kind: String,            // e.g. "build.succeeded", "policy.deny", "drift.alert"
    pub occurred_at: String,     // RFC 3339
    pub subject: Option<String>, // build id or other identifier
    pub payload: serde_json::Value,
}

impl Event {
    pub fn new(
        kind: impl Into<String>,
        subject: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            kind: kind.into(),
            occurred_at: Utc::now().to_rfc3339(),
            subject,
            payload,
        }
    }
}

/// Outbound dispatcher. Holds a `reqwest::Client` so connection pooling is
/// shared across deliveries.
#[derive(Clone)]
pub struct Webhooks {
    client: Arc<reqwest::Client>,
    config: WebhooksSection,
}

impl Webhooks {
    pub fn new(config: WebhooksSection) -> Self {
        Self {
            client: Arc::new(
                reqwest::Client::builder()
                    .user_agent(format!("forge-webhooks/{}", env!("CARGO_PKG_VERSION")))
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .expect("reqwest client"),
            ),
            config,
        }
    }

    /// Best-effort fan-out. Errors are logged and never propagated to the
    /// caller — the build pipeline must not fail because a webhook consumer
    /// is down.
    pub async fn emit(&self, event: &Event) {
        for endpoint in &self.config.endpoints {
            if !accepts(endpoint, &event.kind) {
                continue;
            }
            if let Err(e) = self.deliver_one(endpoint, event).await {
                tracing::warn!(error = %e, url = %endpoint.url, kind = %event.kind, "webhook delivery failed");
            }
        }
    }

    async fn deliver_one(&self, endpoint: &WebhookEndpoint, event: &Event) -> Result<()> {
        let body = serde_json::to_vec(event)?;
        let signature = sign(&endpoint.secret, &body);
        let resp = self
            .client
            .post(&endpoint.url)
            .header("content-type", "application/json")
            .header("x-forge-signature", format!("sha256={signature}"))
            .header("x-forge-event", &event.kind)
            .body(body)
            .send()
            .await
            .map_err(|e| crate::Error::Internal(anyhow::anyhow!(e)))?;
        if !resp.status().is_success() {
            return Err(crate::Error::Internal(anyhow::anyhow!(
                "webhook returned status {}",
                resp.status()
            )));
        }
        Ok(())
    }
}

fn accepts(endpoint: &WebhookEndpoint, kind: &str) -> bool {
    endpoint.events.is_empty() || endpoint.events.iter().any(|e| e == kind)
}

pub fn sign(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

/// Verify an incoming signature header. Constant-time via `hmac::Mac::verify_slice`.
pub fn verify(secret: &str, body: &[u8], signature_header: &str) -> bool {
    let Some(stripped) = signature_header.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(decoded) = hex::decode(stripped) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&decoded).is_ok()
}

// ---------------------------------------------------------------------------
// Persistent retry queue
// ---------------------------------------------------------------------------

use crate::storage::Storage;

const MAX_BACKOFF_SECS: i64 = 30 * 60;
const PRUNE_OLDER_THAN_HOURS: i64 = 24 * 7;

/// Persisted retry queue. Each enqueue writes a row; the worker loop polls
/// pending rows whose `next_attempt` has elapsed and re-attempts delivery.
#[derive(Clone)]
pub struct WebhookQueue {
    storage: Storage,
    client: Arc<reqwest::Client>,
}

impl WebhookQueue {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            client: Arc::new(
                reqwest::Client::builder()
                    .user_agent(format!("forge-webhooks/{}", env!("CARGO_PKG_VERSION")))
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .expect("reqwest client"),
            ),
        }
    }

    pub async fn enqueue(&self, endpoint: &WebhookEndpoint, event: &Event) -> Result<()> {
        let payload = serde_json::to_string(event)?;
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO webhook_deliveries
               (endpoint_url, event_kind, payload, secret, attempts, next_attempt)
               VALUES (?,?,?,?,0,?)"#,
        )
        .bind(&endpoint.url)
        .bind(&event.kind)
        .bind(payload)
        .bind(&endpoint.secret)
        .bind(now)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    /// Pop pending rows that are due, attempt delivery, and reschedule
    /// failures. Returns the number of rows processed.
    pub async fn run_once(&self) -> Result<usize> {
        let now = Utc::now().to_rfc3339();
        let rows = sqlx::query(
            r#"SELECT id, endpoint_url, event_kind, payload, secret, attempts
               FROM webhook_deliveries
               WHERE delivered_at IS NULL AND next_attempt <= ?
               ORDER BY next_attempt ASC LIMIT 32"#,
        )
        .bind(&now)
        .fetch_all(self.storage.pool())
        .await?;

        let count = rows.len();
        for row in rows {
            use sqlx::Row;
            let id: i64 = row.get("id");
            let url: String = row.get("endpoint_url");
            let event_kind: String = row.get("event_kind");
            let payload: String = row.get("payload");
            let secret: String = row.get("secret");
            let attempts: i64 = row.get("attempts");

            let signature = sign(&secret, payload.as_bytes());
            let result = self
                .client
                .post(&url)
                .header("content-type", "application/json")
                .header("x-forge-signature", format!("sha256={signature}"))
                .header("x-forge-event", &event_kind)
                .body(payload.clone())
                .send()
                .await;

            let success = matches!(&result, Ok(r) if r.status().is_success());
            if success {
                let _ =
                    sqlx::query(r#"UPDATE webhook_deliveries SET delivered_at = ? WHERE id = ?"#)
                        .bind(Utc::now().to_rfc3339())
                        .bind(id)
                        .execute(self.storage.pool())
                        .await;
            } else {
                let next = next_attempt_after(attempts + 1);
                let err_str = match &result {
                    Ok(r) => format!("status {}", r.status()),
                    Err(e) => e.to_string(),
                };
                let _ = sqlx::query(
                    r#"UPDATE webhook_deliveries
                       SET attempts = ?, next_attempt = ?, last_error = ?
                       WHERE id = ?"#,
                )
                .bind(attempts + 1)
                .bind(next.to_rfc3339())
                .bind(err_str)
                .bind(id)
                .execute(self.storage.pool())
                .await;
            }
        }
        Ok(count)
    }

    /// Long-running loop. Polls every `interval`, prunes deliveries older
    /// than the retention window once per hour.
    pub async fn run_forever(self, interval: std::time::Duration) {
        let mut last_prune = std::time::Instant::now();
        loop {
            if let Err(e) = self.run_once().await {
                tracing::warn!(error = %e, "webhook retry worker failed");
            }
            if last_prune.elapsed() > std::time::Duration::from_secs(3600) {
                let _ = self.prune_old().await;
                last_prune = std::time::Instant::now();
            }
            tokio::time::sleep(interval).await;
        }
    }

    pub async fn prune_old(&self) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::hours(PRUNE_OLDER_THAN_HOURS);
        let res = sqlx::query(
            r#"DELETE FROM webhook_deliveries
               WHERE delivered_at IS NOT NULL AND delivered_at < ?"#,
        )
        .bind(cutoff.to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(res.rows_affected())
    }
}

/// Exponential backoff capped at `MAX_BACKOFF_SECS`. `attempts` is the
/// next attempt number (1 → 1s, 2 → 2s, 3 → 4s …).
pub fn next_attempt_after(attempts: i64) -> chrono::DateTime<Utc> {
    let exp = attempts.clamp(1, 16) as u32;
    let delay_secs = 1i64
        .checked_shl(exp.saturating_sub(1))
        .unwrap_or(MAX_BACKOFF_SECS)
        .min(MAX_BACKOFF_SECS);
    Utc::now() + chrono::Duration::seconds(delay_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_round_trip() {
        let body = b"{\"hello\":\"world\"}";
        let signed = sign("topsecret", body);
        assert!(verify("topsecret", body, &format!("sha256={signed}")));
    }

    #[test]
    fn verify_rejects_wrong_secret_or_tampered_body() {
        let body = b"payload";
        let sig = sign("right", body);
        assert!(!verify("wrong", body, &format!("sha256={sig}")));
        assert!(!verify("right", b"different", &format!("sha256={sig}")));
        assert!(!verify("right", body, "no-prefix"));
        assert!(!verify("right", body, "sha256=not-hex"));
    }

    #[test]
    fn next_attempt_after_increments_with_attempts() {
        let a = next_attempt_after(1);
        let b = next_attempt_after(4);
        assert!(b > a, "attempt 4 should schedule later than attempt 1");
    }

    #[tokio::test]
    async fn enqueue_then_run_once_marks_failure_and_reschedules() {
        let storage = Storage::open_memory().await.unwrap();
        let queue = WebhookQueue::new(storage.clone());
        let endpoint = WebhookEndpoint {
            // Reserved discard prefix; reqwest won't actually deliver so
            // the run will record a failure and schedule a retry.
            url: "http://127.0.0.1:1/forge".into(),
            secret: "shh".into(),
            events: vec![],
        };
        let event = Event::new("build.start", Some("abc".into()), serde_json::json!({}));
        queue.enqueue(&endpoint, &event).await.unwrap();
        let processed = queue.run_once().await.unwrap();
        assert_eq!(processed, 1);

        // After processing, the row should still be pending with attempts=1.
        use sqlx::Row;
        let row = sqlx::query("SELECT attempts, delivered_at FROM webhook_deliveries")
            .fetch_one(storage.pool())
            .await
            .unwrap();
        let attempts: i64 = row.get("attempts");
        let delivered: Option<String> = row.get("delivered_at");
        assert_eq!(attempts, 1);
        assert!(delivered.is_none());
    }

    #[test]
    fn accepts_filters_by_event_kind() {
        let allow_all = WebhookEndpoint {
            url: "http://x".into(),
            secret: "s".into(),
            events: vec![],
        };
        let only_drift = WebhookEndpoint {
            url: "http://x".into(),
            secret: "s".into(),
            events: vec!["drift.alert".into()],
        };
        assert!(accepts(&allow_all, "build.start"));
        assert!(accepts(&only_drift, "drift.alert"));
        assert!(!accepts(&only_drift, "build.start"));
    }
}
