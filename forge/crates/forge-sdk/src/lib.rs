//! SecureImage Forge SDK — typed Rust client for the v1 HTTP API.
//!
//! ```no_run
//! # async fn run() -> forge_sdk::Result<()> {
//! let client = forge_sdk::Client::new("http://127.0.0.1:7878", "forge_xxxxxxxx")?;
//! let builds = client.list_builds().await?;
//! for b in &builds {
//!     println!("{} {}", b.id, b.status);
//! }
//! # Ok(()) }
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Retry policy applied to transient failures (5xx, 429, network errors).
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(5),
        }
    }
}

impl RetryPolicy {
    fn delay_for(&self, attempt: u32) -> Duration {
        let exp = attempt.saturating_sub(1).min(16);
        let scaled = self
            .initial_delay
            .checked_mul(1u32 << exp)
            .unwrap_or(self.max_delay);
        scaled.min(self.max_delay)
    }

    fn should_retry(&self, attempt: u32, status: Option<u16>, transport_err: bool) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        if transport_err {
            return true;
        }
        match status {
            Some(s) => s == 408 || s == 429 || s >= 500,
            None => false,
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid base URL: {0}")]
    InvalidBaseUrl(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("decode: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("api error {status}: {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, Clone, Deserialize)]
pub struct BuildSummary {
    pub id: String,
    pub name: String,
    pub runtime: String,
    pub base_image: String,
    pub status: String,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateBuildRequest {
    pub name: String,
    pub runtime: String,
    pub base: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compliance: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub architectures: Vec<String>,
    #[serde(default)]
    pub no_sbom: bool,
    #[serde(default)]
    pub no_sign: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatedBuild {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanResult {
    pub scanner: String,
    pub scanned_at: String,
    pub findings: Vec<Vulnerability>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriftSnapshot {
    pub id: i64,
    pub build_id: String,
    pub scanner: String,
    pub scanned_at: String,
    pub new_critical: i64,
    pub new_high: i64,
    pub findings_count: usize,
}

#[derive(Clone, Debug)]
pub struct Client {
    http: reqwest::Client,
    base: Url,
    token: String,
    retry: RetryPolicy,
}

impl Client {
    pub fn new(base_url: impl AsRef<str>, token: impl Into<String>) -> Result<Self> {
        Self::with_client(base_url, token, default_http()?)
    }

    pub fn with_client(
        base_url: impl AsRef<str>,
        token: impl Into<String>,
        http: reqwest::Client,
    ) -> Result<Self> {
        let base = Url::parse(base_url.as_ref())
            .map_err(|e| ClientError::InvalidBaseUrl(e.to_string()))?;
        Ok(Self {
            http,
            base,
            token: token.into(),
            retry: RetryPolicy::default(),
        })
    }

    /// Replace the retry policy. Returns self for fluent construction.
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    /// Disable retries entirely.
    pub fn without_retry(mut self) -> Self {
        self.retry = RetryPolicy {
            max_attempts: 1,
            ..self.retry
        };
        self
    }

    /// Run a request builder with the configured retry policy. The closure is
    /// invoked once per attempt to rebuild the `RequestBuilder` because
    /// `reqwest::RequestBuilder::send` consumes self.
    async fn send_with_retry<F>(&self, build: F) -> Result<reqwest::Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let result = build().send().await;
            let (status, transport_err) = match &result {
                Ok(r) => (Some(r.status().as_u16()), false),
                Err(_) => (None, true),
            };
            if !self.retry.should_retry(attempt, status, transport_err) {
                return result.map_err(ClientError::Http);
            }
            tokio::time::sleep(self.retry.delay_for(attempt)).await;
        }
    }

    fn url(&self, path: &str) -> Result<Url> {
        self.base
            .join(path)
            .map_err(|e| ClientError::InvalidBaseUrl(e.to_string()))
    }

    pub async fn list_builds(&self) -> Result<Vec<BuildSummary>> {
        self.get_json("/v1/builds").await
    }

    pub async fn get_build(&self, id: &str) -> Result<BuildSummary> {
        self.get_json(&format!("/v1/builds/{id}")).await
    }

    pub async fn create_build(&self, req: &CreateBuildRequest) -> Result<CreatedBuild> {
        let url = self.url("/v1/builds")?;
        let body = serde_json::to_vec(req)?;
        let resp = self
            .send_with_retry(|| {
                self.http
                    .post(url.clone())
                    .bearer_auth(&self.token)
                    .header("content-type", "application/json")
                    .body(body.clone())
            })
            .await?;
        decode_or_error(resp).await
    }

    pub async fn start_build(&self, id: &str) -> Result<CreatedBuild> {
        let url = self.url(&format!("/v1/builds/{id}/start"))?;
        let resp = self
            .send_with_retry(|| self.http.post(url.clone()).bearer_auth(&self.token))
            .await?;
        decode_or_error(resp).await
    }

    pub async fn get_scan(&self, id: &str) -> Result<ScanResult> {
        self.get_json(&format!("/v1/builds/{id}/scan")).await
    }

    pub async fn get_log(&self, id: &str) -> Result<String> {
        let url = self.url(&format!("/v1/builds/{id}/log"))?;
        let resp = self
            .send_with_retry(|| self.http.get(url.clone()).bearer_auth(&self.token))
            .await?;
        if !resp.status().is_success() {
            return Err(error_for_status(resp).await);
        }
        Ok(resp.text().await?)
    }

    pub async fn list_drift(&self, id: &str) -> Result<Vec<DriftSnapshot>> {
        self.get_json(&format!("/v1/builds/{id}/drift")).await
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = self.url(path)?;
        let resp = self
            .send_with_retry(|| self.http.get(url.clone()).bearer_auth(&self.token))
            .await?;
        decode_or_error(resp).await
    }
}

async fn decode_or_error<T: for<'de> Deserialize<'de>>(resp: reqwest::Response) -> Result<T> {
    if !resp.status().is_success() {
        return Err(error_for_status(resp).await);
    }
    let bytes = resp.bytes().await?;
    let value = serde_json::from_slice::<T>(&bytes)?;
    Ok(value)
}

async fn error_for_status(resp: reqwest::Response) -> ClientError {
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| body.trim().to_string());
    ClientError::Api { status, message }
}

fn default_http() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(format!("forge-sdk/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_base_url() {
        let err = Client::new("not a url", "tok").unwrap_err();
        assert!(matches!(err, ClientError::InvalidBaseUrl(_)));
    }

    #[test]
    fn url_join_handles_trailing_slash() {
        let c = Client::new("http://localhost:7878/", "tok").unwrap();
        assert_eq!(
            c.url("/v1/builds").unwrap().as_str(),
            "http://localhost:7878/v1/builds"
        );
    }

    #[test]
    fn retry_policy_should_retry_on_5xx_429_408_and_transport_error() {
        let policy = RetryPolicy::default();
        assert!(policy.should_retry(1, Some(500), false));
        assert!(policy.should_retry(1, Some(503), false));
        assert!(policy.should_retry(1, Some(429), false));
        assert!(policy.should_retry(1, Some(408), false));
        assert!(policy.should_retry(1, None, true));
        assert!(!policy.should_retry(1, Some(200), false));
        assert!(!policy.should_retry(1, Some(401), false));
        assert!(!policy.should_retry(1, Some(404), false));
    }

    #[test]
    fn retry_policy_stops_after_max_attempts() {
        let policy = RetryPolicy::default();
        assert!(!policy.should_retry(policy.max_attempts, Some(500), false));
    }

    #[test]
    fn retry_policy_backoff_is_capped() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
        };
        assert!(policy.delay_for(1) >= Duration::from_millis(100));
        assert!(policy.delay_for(20) <= policy.max_delay);
    }

    #[test]
    fn without_retry_disables_retries() {
        let c = Client::new("http://localhost:7878", "tok")
            .unwrap()
            .without_retry();
        assert_eq!(c.retry.max_attempts, 1);
    }
}
