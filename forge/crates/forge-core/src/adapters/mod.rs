//! Concrete adapters. Each module wraps an external Apache-2.0 binary through
//! the `ProcessRunner` abstraction so that:
//!   * Bundled (rootless) and host-installed binaries are interchangeable.
//!   * Tests can use `MockRunner` instead of requiring the real tool.

pub mod buildkit;
pub mod cosign;
pub mod grype;
pub mod opa;
pub mod syft;
pub mod trivy;
