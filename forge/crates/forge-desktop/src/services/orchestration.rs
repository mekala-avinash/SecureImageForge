//! Write-side: dispatch a new build through the orchestrator. The UI calls
//! `start_build` from a button handler; we spawn the work onto our shared
//! runtime so the UI thread is never blocked.

use std::sync::Arc;

use forge_core::adapters::buildkit::{BuildkitBuilder, BuildkitConfig};
use forge_core::adapters::cosign::{CosignConfig, CosignSigner};
use forge_core::adapters::grype::{GrypeConfig, GrypeScanner, MergedScanner};
use forge_core::adapters::opa::{OpaConfig, OpaPolicyEngine};
use forge_core::adapters::syft::{SyftConfig, SyftSbomGenerator};
use forge_core::adapters::trivy::{TrivyConfig, TrivyScanner};
use forge_core::domain::BuildSpec;
use forge_core::orchestrator::BuildOrchestrator;
use forge_core::process::TokioRunner;
use forge_core::provenance::ProvenanceRepo;

use crate::state::{runtime, AppState};

/// Spawn a build on the shared runtime; returns immediately with a join
/// handle so callers can show "running" UI without blocking.
pub fn start_build(state: &AppState, spec: BuildSpec) -> tokio::task::JoinHandle<()> {
    let cfg = state.config.clone();
    let repo = state.repo.clone();
    let logs = state.logs.clone();
    let toolchain = state.toolchain.clone();

    runtime().spawn(async move {
        let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
        let bundled_prefix = toolchain.prefix().map(|p| p.to_path_buf());

        let orchestrator = BuildOrchestrator {
            builder: Arc::new(BuildkitBuilder::new(
                runner.clone(),
                BuildkitConfig {
                    addr: cfg.buildkit.addr.clone(),
                    bundled_prefix: bundled_prefix.clone(),
                    registry_target: cfg.registry.default_target.clone(),
                    push: cfg.registry.default_push,
                    ..Default::default()
                },
            )),
            scanner: Arc::new(MergedScanner {
                primary: Arc::new(TrivyScanner::new(
                    runner.clone(),
                    TrivyConfig {
                        bundled_prefix: bundled_prefix.clone(),
                        ..Default::default()
                    },
                )),
                secondary: Arc::new(GrypeScanner::new(
                    runner.clone(),
                    GrypeConfig {
                        bundled_prefix: bundled_prefix.clone(),
                        ..Default::default()
                    },
                )),
            }),
            sbom: Arc::new(SyftSbomGenerator::new(
                runner.clone(),
                SyftConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            )),
            signer: Arc::new(CosignSigner::new(
                runner.clone(),
                CosignConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            )),
            attestor: Some(Arc::new(CosignSigner::new(
                runner.clone(),
                CosignConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            ))),
            policy: Arc::new(OpaPolicyEngine::new(
                runner.clone(),
                OpaConfig {
                    bundled_prefix,
                    profiles: spec.compliance.iter().copied().collect(),
                    ..Default::default()
                },
            )),
            provenance: Some(ProvenanceRepo::new(repo.storage().clone())),
            repo,
            logs,
        };

        if let Err(e) = orchestrator.run(spec).await {
            tracing::error!(error = %e, "desktop build failed");
        }
    })
}
