//! forge-core: domain model and orchestration primitives for SecureImage Forge.
//!
//! Adapter layout:
//!   * `tooling` — async traits for ImageBuilder, Scanner, SbomGenerator,
//!     Signer, PolicyEngine.
//!   * `process` — `ProcessRunner` abstraction (real + mock) that all
//!     subprocess-driven adapters use.
//!   * `dockerfile` — pure Dockerfile generation from a `BuildSpec`.
//!   * `adapters` — concrete buildkit/trivy/syft/cosign/opa implementations.
//!   * `repo` — SQLite-backed persistence for `BuildRecord`s.
//!   * `orchestrator` — drives a build through generate → build → scan → sbom
//!     → sign → policy, persisting the record.

pub mod adapters;
pub mod audit;
pub mod config;
pub mod dockerfile;
pub mod domain;
pub mod drift;
pub mod error;
pub mod logs;
pub mod metrics;
pub mod orchestrator;
#[cfg(feature = "pg")]
pub mod pg_storage;
pub mod process;
pub mod runtime;
pub mod provenance;
pub mod rbac;
pub mod registry;
pub mod repo;
pub mod sarif;
pub mod storage;
pub mod team;
pub mod telemetry;
pub mod toolchain;
pub mod tooling;
pub mod updater;
pub mod webhooks;

pub use error::{Error, Result};
