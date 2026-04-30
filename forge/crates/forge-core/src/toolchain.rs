//! Resolve external Apache-2.0 binaries from either a bundled prefix (set up
//! by `xtask bundle-buildkit`) or PATH. The vendor manifest pins versions and
//! sha256 digests so installs are reproducible.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::process::resolve_tool;
use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorManifest {
    pub generated_at: Option<String>,
    pub tools: Vec<VendorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorEntry {
    pub name: String,
    pub version: String,
    pub platform: String, // e.g. "darwin/arm64"
    pub sha256: String,
    pub relative_path: String, // path within the vendor prefix
}

/// Wraps a `vendor/` directory containing platform-specific tool binaries.
#[derive(Debug, Clone)]
pub struct Toolchain {
    bundled_prefix: Option<PathBuf>,
}

impl Toolchain {
    pub fn new(bundled_prefix: Option<PathBuf>) -> Self {
        Self { bundled_prefix }
    }

    pub fn from_env() -> Self {
        Self {
            bundled_prefix: std::env::var_os("FORGE_VENDOR_PREFIX").map(PathBuf::from),
        }
    }

    pub fn prefix(&self) -> Option<&Path> {
        self.bundled_prefix.as_deref()
    }

    /// Resolve a tool: try `<prefix>/<platform>/<tool>` first, then PATH.
    pub fn resolve(&self, tool: &str) -> Result<PathBuf> {
        let platform_prefix = self
            .bundled_prefix
            .as_ref()
            .map(|p| p.join(host_platform()));
        if let Some(p) = &platform_prefix {
            if let Ok(path) = resolve_tool(tool, Some(p.as_path())) {
                return Ok(path);
            }
        }
        resolve_tool(tool, None)
    }

    pub fn manifest_path(&self) -> Option<PathBuf> {
        self.bundled_prefix
            .as_ref()
            .map(|p| p.join("manifest.json"))
    }

    pub fn load_manifest(&self) -> Option<VendorManifest> {
        let path = self.manifest_path()?;
        let bytes = std::fs::read(&path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}

/// Returns "<os>/<arch>" matching the way Go releases name their archives, so
/// the vendor manifest can use canonical platform strings like "darwin/arm64".
pub fn host_platform() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}/{arch}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_platform_uses_canonical_strings() {
        let p = host_platform();
        assert!(p.contains('/'));
        assert!(!p.contains("x86_64"));
        assert!(!p.contains("aarch64"));
        assert!(!p.contains("macos"));
    }

    #[test]
    fn resolve_falls_back_to_path() {
        let tc = Toolchain::new(Some(PathBuf::from("/nonexistent")));
        // `sh` exists on every Unix; on Windows the test is skipped.
        if cfg!(unix) {
            assert!(tc.resolve("sh").is_ok());
        }
    }

    #[test]
    fn missing_manifest_returns_none() {
        let tc = Toolchain::new(Some(PathBuf::from("/nonexistent")));
        assert!(tc.load_manifest().is_none());
    }
}
