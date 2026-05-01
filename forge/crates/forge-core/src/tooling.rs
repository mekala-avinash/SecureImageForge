//! Async trait abstractions over external Apache-2.0 tools.

use async_trait::async_trait;

use crate::domain::{BuildSpec, Sbom, ScanResult};
use crate::Result;

/// Output of a successful image build, prior to scan/sign/sbom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltImage {
    /// `sha256:...` digest reported by the build engine.
    pub digest: String,
    /// Local OCI tag or registry reference.
    pub reference: String,
    /// Aggregated build log (line-buffered).
    pub log: String,
}

#[async_trait]
pub trait ImageBuilder: Send + Sync {
    async fn build(&self, spec: &BuildSpec, dockerfile: &str) -> Result<BuiltImage>;
}

#[async_trait]
pub trait Scanner: Send + Sync {
    async fn scan(&self, image_ref: &str) -> Result<ScanResult>;
}

#[async_trait]
pub trait SbomGenerator: Send + Sync {
    async fn generate(&self, image_ref: &str) -> Result<Sbom>;
}

#[async_trait]
pub trait Signer: Send + Sync {
    async fn sign(&self, image_ref: &str) -> Result<()>;
}

#[async_trait]
pub trait Attestor: Send + Sync {
    async fn attest(&self, image_ref: &str, predicate_json: &str) -> Result<()>;
}

#[async_trait]
pub trait Verifier: Send + Sync {
    async fn verify(&self, image_ref: &str) -> Result<()>;
}

/// Result returned by the policy engine — separate enum lets the orchestrator
/// distinguish "no findings" from "findings but allowed by waiver".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { reasons: Vec<String> },
}

#[async_trait]
pub trait PolicyEngine: Send + Sync {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision>;
}
