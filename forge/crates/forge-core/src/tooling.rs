//! Trait abstractions over external Apache-2.0 tools.
//!
//! Concrete adapters are filled in during Phase 1. The trait surface lives here
//! so dependent crates (cli, api, desktop) can compile against the contract
//! without pulling in process-spawning code.
//!
//! Phase 0 keeps these synchronous to avoid an `async-trait` dep until the
//! adapters land. Phase 1 promotes them to async with `async_trait`.

use crate::domain::{BuildRecord, BuildSpec, Sbom, ScanResult};
use crate::Result;

pub trait ImageBuilder: Send + Sync {
    fn build(&self, spec: &BuildSpec) -> Result<BuildRecord>;
}

pub trait Scanner: Send + Sync {
    fn scan(&self, image_ref: &str) -> Result<ScanResult>;
}

pub trait SbomGenerator: Send + Sync {
    fn generate(&self, image_ref: &str) -> Result<Sbom>;
}

pub trait Signer: Send + Sync {
    fn sign(&self, image_ref: &str) -> Result<()>;
}

pub trait PolicyEngine: Send + Sync {
    fn evaluate(&self, record: &BuildRecord) -> Result<()>;
}
