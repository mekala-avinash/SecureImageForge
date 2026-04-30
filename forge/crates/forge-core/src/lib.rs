//! forge-core: domain model and orchestration primitives for SecureImage Forge.
//!
//! This crate is intentionally pure-Rust and side-effect free at the API layer;
//! external tool integrations (buildkit, trivy, syft, cosign, opa) live in
//! sibling modules and are invoked through the `tooling` trait abstractions so
//! they can be mocked in unit tests and swapped per-platform.

pub mod domain;
pub mod error;
pub mod storage;
pub mod telemetry;
pub mod tooling;

pub use error::{Error, Result};
