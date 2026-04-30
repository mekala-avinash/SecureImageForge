//! Domain model — values are immutable. Constructors return new values rather
//! than mutating in place to align with the project's immutability rules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    Java,
    Dotnet,
    Go,
    Node,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseImage {
    Alpine,
    Debian,
    Distroless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceProfile {
    Hipaa,
    Soc2,
    PciDss,
    Cis,
    FedrampModerate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    Amd64,
    Arm64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardeningOptions {
    pub remove_shells: bool,
    pub remove_pkg_managers: bool,
    pub readonly_rootfs: bool,
    pub non_root_user: bool,
}

impl HardeningOptions {
    pub fn strict() -> Self {
        Self {
            remove_shells: true,
            remove_pkg_managers: true,
            readonly_rootfs: true,
            non_root_user: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildSpec {
    pub name: String,
    pub runtime: Runtime,
    pub base_image: BaseImage,
    pub architectures: BTreeSet<Architecture>,
    pub compliance: BTreeSet<ComplianceProfile>,
    pub hardening: HardeningOptions,
    pub generate_sbom: bool,
    pub sign: bool,
}

impl BuildSpec {
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::Error::InvalidSpec("name must not be empty".into()));
        }
        if self.architectures.is_empty() {
            return Err(crate::Error::InvalidSpec(
                "at least one architecture required".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: Severity,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanner: String,
    pub scanned_at: DateTime<Utc>,
    pub findings: Vec<Vulnerability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sbom {
    pub format: String, // "cyclonedx" | "spdx"
    pub document: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub digest: String,
    pub registry_ref: Option<String>,
    pub bytes: u64,
    pub architecture: Architecture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildRecord {
    pub id: Uuid,
    pub spec: BuildSpec,
    pub status: BuildStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub artifacts: Vec<BuildArtifact>,
    pub scan: Option<ScanResult>,
    pub sbom: Option<Sbom>,
    pub log_path: Option<String>,
}

impl BuildRecord {
    pub fn new(spec: BuildSpec) -> Self {
        Self {
            id: Uuid::new_v4(),
            spec,
            status: BuildStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            artifacts: Vec::new(),
            scan: None,
            sbom: None,
            log_path: None,
        }
    }

    pub fn with_status(&self, status: BuildStatus) -> Self {
        Self {
            status,
            ..self.clone()
        }
    }
}
