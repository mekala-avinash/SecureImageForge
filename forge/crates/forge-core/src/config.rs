//! User configuration loaded from `<data_dir>/config.toml` with env overrides.
//!
//! Layered precedence (highest first):
//!   1. CLI flags
//!   2. Environment variables (`FORGE_*`)
//!   3. Config file
//!   4. Built-in defaults

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub buildkit: BuildkitSection,
    #[serde(default)]
    pub registry: RegistrySection,
    #[serde(default)]
    pub vendor: VendorSection,
    #[serde(default)]
    pub updater: UpdaterSection,
    #[serde(default)]
    pub drift: DriftSection,
    #[serde(default)]
    pub storage: StorageSection,
    #[serde(default)]
    pub telemetry: TelemetrySection,
    #[serde(default)]
    pub webhooks: WebhooksSection,
    #[serde(default)]
    pub auth: AuthSection,
    #[serde(default)]
    pub workers: WorkersSection,
    #[serde(default)]
    pub features: FeatureFlagsSection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageSection {
    /// Optional override for the database URL. Defaults to `sqlite://<data_dir>/forge.sqlite`
    /// when unset. `postgres://...` is recognized and reserved for Phase 6.5.
    pub database_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySection {
    /// Listen address for the Prometheus metrics scrape endpoint. When `None`,
    /// metrics are still collected but not exposed.
    pub metrics_addr: Option<String>,
    /// OTLP collector endpoint (e.g. `http://otel:4317`). When `None`, tracing
    /// emits stdout JSON only.
    pub otlp_endpoint: Option<String>,
    /// Service name reported to the collector.
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebhooksSection {
    /// Where to deliver event payloads. Empty list disables webhooks.
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub url: String,
    /// Hex-encoded HMAC-SHA256 secret. Sent as `X-Forge-Signature: sha256=<hex>`
    /// over the JSON payload.
    pub secret: String,
    /// Filter: only these event names are delivered. Empty = all.
    #[serde(default)]
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthSection {
    #[serde(default)]
    pub mode: AuthMode,
    #[serde(default)]
    pub oidc: OidcSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    Local,
    Oidc,
    #[default]
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcSection {
    pub enabled: bool,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub jwks_refresh_seconds: u64,
    pub allowed_clock_skew_seconds: u64,
}

impl Default for OidcSection {
    fn default() -> Self {
        Self {
            enabled: false,
            issuer: None,
            audience: None,
            jwks_refresh_seconds: 300,
            allowed_clock_skew_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkersSection {
    pub concurrency: usize,
    pub lease_seconds: u64,
    pub max_retries: u32,
    pub backoff_strategy: String,
}

impl Default for WorkersSection {
    fn default() -> Self {
        Self {
            concurrency: 2,
            lease_seconds: 60,
            max_retries: 3,
            backoff_strategy: "exponential".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsSection {
    pub oidc_auth: bool,
    pub project_scoping: bool,
    pub durable_queue: bool,
    pub secret_providers: bool,
}

impl Default for FeatureFlagsSection {
    fn default() -> Self {
        Self {
            oidc_auth: false,
            project_scoping: true,
            durable_queue: true,
            secret_providers: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildkitSection {
    pub addr: String,
}

impl Default for BuildkitSection {
    fn default() -> Self {
        Self {
            addr: "unix:///run/buildkit/buildkitd.sock".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistrySection {
    pub default_push: bool,
    pub default_target: Option<String>,
    #[serde(default)]
    pub auth: crate::registry::RegistryAuth,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorSection {
    pub prefix: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterSection {
    pub feed_url: String,
    pub channel: String,
    pub cosign_key_path: Option<PathBuf>,
    pub allow_unsigned: bool,
    pub auto_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSection {
    pub scheduler_enabled: bool,
    pub interval_seconds: u64,
}

impl Default for DriftSection {
    fn default() -> Self {
        Self {
            scheduler_enabled: false,
            interval_seconds: 3600,
        }
    }
}

impl Default for UpdaterSection {
    fn default() -> Self {
        Self {
            feed_url: "https://updates.secureimage-forge.dev/manifest.json".into(),
            channel: "stable".into(),
            cosign_key_path: None,
            allow_unsigned: false,
            auto_check: true,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;
        let parsed: Config = toml::from_str(&text)
            .map_err(|e| crate::Error::Internal(anyhow::anyhow!("config parse: {e}")))?;
        Ok(parsed.with_env_overrides())
    }

    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(addr) = std::env::var("FORGE_BUILDKIT_ADDR") {
            self.buildkit.addr = addr;
        }
        if let Ok(prefix) = std::env::var("FORGE_VENDOR_PREFIX") {
            self.vendor.prefix = Some(PathBuf::from(prefix));
        }
        if let Ok(target) = std::env::var("FORGE_REGISTRY_TARGET") {
            self.registry.default_target = Some(target);
        }
        if let Ok(push) = std::env::var("FORGE_REGISTRY_PUSH") {
            self.registry.default_push = matches!(push.as_str(), "1" | "true" | "TRUE" | "yes");
        }
        if let Ok(feed) = std::env::var("FORGE_UPDATER_FEED") {
            self.updater.feed_url = feed;
        }
        if let Ok(channel) = std::env::var("FORGE_UPDATER_CHANNEL") {
            self.updater.channel = channel;
        }
        if let Ok(enabled) = std::env::var("FORGE_DRIFT_SCHEDULER") {
            self.drift.scheduler_enabled =
                matches!(enabled.as_str(), "1" | "true" | "TRUE" | "yes");
        }
        if let Ok(seconds) = std::env::var("FORGE_DRIFT_INTERVAL_SECONDS") {
            if let Ok(seconds) = seconds.parse() {
                self.drift.interval_seconds = seconds;
            }
        }
        if let Ok(url) =
            std::env::var("FORGE_DATABASE_URL").or_else(|_| std::env::var("DATABASE_URL"))
        {
            self.storage.database_url = Some(url);
        }
        if let Ok(addr) = std::env::var("FORGE_METRICS_ADDR") {
            self.telemetry.metrics_addr = Some(addr);
        }
        if let Ok(endpoint) = std::env::var("FORGE_OTLP_ENDPOINT") {
            self.telemetry.otlp_endpoint = Some(endpoint);
        }
        if let Ok(mode) = std::env::var("FORGE_AUTH_MODE") {
            self.auth.mode = match mode.as_str() {
                "local" => AuthMode::Local,
                "oidc" => AuthMode::Oidc,
                _ => AuthMode::Hybrid,
            };
        }
        if let Ok(enabled) = std::env::var("FORGE_OIDC_ENABLED") {
            self.auth.oidc.enabled = matches!(enabled.as_str(), "1" | "true" | "TRUE" | "yes");
        }
        if let Ok(issuer) = std::env::var("FORGE_OIDC_ISSUER") {
            self.auth.oidc.issuer = Some(issuer);
        }
        if let Ok(audience) = std::env::var("FORGE_OIDC_AUDIENCE") {
            self.auth.oidc.audience = Some(audience);
        }
        if let Ok(v) = std::env::var("FORGE_WORKER_CONCURRENCY") {
            if let Ok(v) = v.parse::<usize>() {
                self.workers.concurrency = v.max(1);
            }
        }
        if let Ok(v) = std::env::var("FORGE_WORKER_LEASE_SECONDS") {
            if let Ok(v) = v.parse::<u64>() {
                self.workers.lease_seconds = v.max(1);
            }
        }
        if let Ok(v) = std::env::var("FORGE_WORKER_MAX_RETRIES") {
            if let Ok(v) = v.parse::<u32>() {
                self.workers.max_retries = v;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn missing_file_returns_defaults() {
        let dir = TempDir::new().unwrap();
        let cfg = Config::load(&dir.path().join("missing.toml")).unwrap();
        assert!(cfg.buildkit.addr.contains("buildkit"));
    }

    #[test]
    fn parses_toml_and_applies_overrides() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [buildkit]
            addr = "tcp://127.0.0.1:1234"

            [registry]
            default_push = true
            default_target = "ghcr.io/example/forge"
            "#,
        )
        .unwrap();
        let cfg = Config::load(&path).unwrap();
        assert_eq!(cfg.buildkit.addr, "tcp://127.0.0.1:1234");
        assert!(cfg.registry.default_push);
        assert_eq!(
            cfg.registry.default_target.as_deref(),
            Some("ghcr.io/example/forge")
        );
    }
}
