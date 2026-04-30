//! Auto-updater. Fetches a signed JSON manifest from a feed URL and decides
//! whether a newer build is available for the current host platform.
//!
//! Signing model: the publisher signs the manifest with `cosign sign-blob`
//! using a long-lived key; the public key is bundled into the binary at
//! `cosign.pub`. Verification is delegated to the bundled `cosign verify-blob`
//! invocation so the same toolchain that secures images secures updates.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::process::{ProcessRunner, ProcessSpec};
use crate::toolchain::{host_platform, Toolchain};
use crate::{Error, Result};

/// Default public key shipped with the binary. Kept as a constant so the
/// build pipeline can replace it via `--cfg embed_pub_key` for a release.
pub const EMBEDDED_COSIGN_PUB: &str = ""; // populated at release time

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub published_at: String,
    pub channel: String, // "stable" | "beta"
    pub releases: Vec<UpdateRelease>,
    pub min_required: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRelease {
    pub platform: String, // e.g. "darwin/arm64"
    pub url: String,      // installer/binary URL
    pub sha256: String,   // hex digest of the artifact
    pub signature_url: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateDecision {
    UpToDate,
    UpdateAvailable {
        from: String,
        to: String,
        release: UpdateRelease,
    },
    UpgradeRequired {
        current: String,
        minimum: String,
    },
}

#[derive(Debug, Clone)]
pub struct UpdaterConfig {
    pub feed_url: String,
    pub current_version: String,
    pub channel: String,
    /// If set, manifest is verified with `cosign verify-blob --key=<path>`.
    /// If `None`, the embedded public key is used (or verification is skipped
    /// when both are unset, which is *only* acceptable in dev builds).
    pub cosign_key_path: Option<PathBuf>,
    pub allow_unsigned: bool,
}

pub struct Updater {
    runner: Arc<dyn ProcessRunner>,
    toolchain: Arc<Toolchain>,
    config: UpdaterConfig,
}

impl Updater {
    pub fn new(
        runner: Arc<dyn ProcessRunner>,
        toolchain: Arc<Toolchain>,
        config: UpdaterConfig,
    ) -> Self {
        Self {
            runner,
            toolchain,
            config,
        }
    }

    /// Pure decision logic broken out so tests can pass a manifest in directly
    /// without touching the network.
    pub fn decide(&self, manifest: &UpdateManifest) -> UpdateDecision {
        if let Some(min) = &manifest.min_required {
            if version_lt(&self.config.current_version, min) {
                return UpdateDecision::UpgradeRequired {
                    current: self.config.current_version.clone(),
                    minimum: min.clone(),
                };
            }
        }
        if manifest.channel != self.config.channel {
            return UpdateDecision::UpToDate;
        }
        if !version_lt(&self.config.current_version, &manifest.version) {
            return UpdateDecision::UpToDate;
        }
        let host = host_platform();
        match manifest
            .releases
            .iter()
            .find(|r| r.platform == host)
            .cloned()
        {
            Some(release) => UpdateDecision::UpdateAvailable {
                from: self.config.current_version.clone(),
                to: manifest.version.clone(),
                release,
            },
            None => UpdateDecision::UpToDate,
        }
    }

    /// Verify a manifest payload against an attached cosign signature using
    /// `cosign verify-blob`. Skipped only when `allow_unsigned` is true.
    #[allow(clippy::const_is_empty)] // EMBEDDED_COSIGN_PUB is replaced at release time.
    pub async fn verify_manifest(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        if self.config.allow_unsigned {
            return Ok(());
        }
        let cosign = self.toolchain.resolve("cosign")?;
        let dir = tempfile::tempdir()?;
        let payload_path = dir.path().join("manifest.json");
        let sig_path = dir.path().join("manifest.sig");
        std::fs::write(&payload_path, payload)?;
        std::fs::write(&sig_path, signature)?;

        let mut spec = ProcessSpec::new(cosign.to_string_lossy().to_string())
            .arg("verify-blob")
            .arg("--signature")
            .arg(sig_path.to_string_lossy().to_string())
            .arg("--insecure-ignore-tlog=true");
        if let Some(key) = &self.config.cosign_key_path {
            spec = spec.arg("--key").arg(key.to_string_lossy().to_string());
        } else if !EMBEDDED_COSIGN_PUB.is_empty() {
            let key_path = dir.path().join("cosign.pub");
            std::fs::write(&key_path, EMBEDDED_COSIGN_PUB)?;
            spec = spec
                .arg("--key")
                .arg(key_path.to_string_lossy().to_string());
        } else {
            return Err(Error::Internal(anyhow::anyhow!(
                "no cosign public key available for update verification"
            )));
        }
        spec = spec.arg(payload_path.to_string_lossy().to_string());

        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "cosign".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        Ok(())
    }

    pub fn config(&self) -> &UpdaterConfig {
        &self.config
    }
}

/// Compare semver-ish versions: lexicographic on numeric segments, ignoring
/// any "v" prefix. Sufficient for our linear "major.minor.patch" scheme.
pub fn version_lt(a: &str, b: &str) -> bool {
    let parse = |s: &str| {
        s.trim_start_matches('v')
            .split('.')
            .map(|p| p.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>()
    };
    parse(a) < parse(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(version: &str, channel: &str, has_host: bool) -> UpdateManifest {
        let host = host_platform();
        let releases = if has_host {
            vec![UpdateRelease {
                platform: host,
                url: "https://example.invalid/forge.bin".into(),
                sha256: "deadbeef".into(),
                signature_url: None,
                size_bytes: 1234,
            }]
        } else {
            vec![]
        };
        UpdateManifest {
            version: version.into(),
            published_at: "2025-01-01T00:00:00Z".into(),
            channel: channel.into(),
            releases,
            min_required: None,
        }
    }

    fn updater(current: &str, channel: &str) -> Updater {
        let runner: Arc<crate::process::TokioRunner> = Arc::new(crate::process::TokioRunner);
        Updater::new(
            runner,
            Arc::new(Toolchain::new(None)),
            UpdaterConfig {
                feed_url: "https://example.invalid/feed.json".into(),
                current_version: current.into(),
                channel: channel.into(),
                cosign_key_path: None,
                allow_unsigned: true,
            },
        )
    }

    #[test]
    fn version_lt_compares_numeric_segments() {
        assert!(version_lt("0.1.0", "0.2.0"));
        assert!(version_lt("v0.1.9", "v0.1.10"));
        assert!(!version_lt("1.0.0", "1.0.0"));
        assert!(!version_lt("1.0.1", "1.0.0"));
    }

    #[test]
    fn current_matches_latest_returns_up_to_date() {
        let u = updater("0.5.0", "stable");
        let d = u.decide(&manifest("0.5.0", "stable", true));
        assert_eq!(d, UpdateDecision::UpToDate);
    }

    #[test]
    fn newer_version_offers_update() {
        let u = updater("0.5.0", "stable");
        let m = manifest("0.6.0", "stable", true);
        match u.decide(&m) {
            UpdateDecision::UpdateAvailable { from, to, .. } => {
                assert_eq!(from, "0.5.0");
                assert_eq!(to, "0.6.0");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn channel_mismatch_yields_up_to_date() {
        let u = updater("0.5.0", "stable");
        let d = u.decide(&manifest("0.6.0", "beta", true));
        assert_eq!(d, UpdateDecision::UpToDate);
    }

    #[test]
    fn missing_host_release_falls_back_to_up_to_date() {
        let u = updater("0.5.0", "stable");
        let d = u.decide(&manifest("0.6.0", "stable", false));
        assert_eq!(d, UpdateDecision::UpToDate);
    }

    #[test]
    fn min_required_triggers_upgrade_required() {
        let u = updater("0.4.0", "stable");
        let mut m = manifest("0.6.0", "stable", true);
        m.min_required = Some("0.5.0".into());
        match u.decide(&m) {
            UpdateDecision::UpgradeRequired { current, minimum } => {
                assert_eq!(current, "0.4.0");
                assert_eq!(minimum, "0.5.0");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
