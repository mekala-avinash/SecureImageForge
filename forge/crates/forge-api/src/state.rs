use std::sync::Arc;
use std::time::Duration;

use forge_core::adapters::buildkit::{BuildkitBuilder, BuildkitConfig};
use forge_core::adapters::cosign::{CosignConfig, CosignSigner};
use forge_core::adapters::grype::{GrypeConfig, GrypeScanner, MergedScanner};
use forge_core::adapters::opa::{OpaConfig, OpaPolicyEngine};
use forge_core::adapters::syft::{SyftConfig, SyftSbomGenerator};
use forge_core::adapters::trivy::{TrivyConfig, TrivyScanner};
use forge_core::audit::AuditLog;
use forge_core::config::Config;
use forge_core::drift::{run_scheduler, DriftDetector};
use forge_core::logs::LogStore;
use forge_core::orchestrator::BuildOrchestrator;
use forge_core::process::TokioRunner;
use forge_core::provenance::ProvenanceRepo;
use forge_core::rbac::PrincipalRepo;
use forge_core::repo::BuildRepo;
use forge_core::storage::Storage;
use forge_core::team::{BuildQueueRepo, ScopeRepo, TeamRepo};
use forge_core::toolchain::Toolchain;

#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub builds: Arc<dyn BuildRepo>,
    pub logs: Arc<dyn LogStore>,
    pub audit: Arc<dyn AuditLog>,
    pub principals: Arc<dyn PrincipalRepo>,
    pub provenance: Arc<dyn ProvenanceRepo>,
    pub toolchain: Arc<Toolchain>,
    pub drift: Arc<dyn DriftDetector>,
    pub team: Arc<dyn TeamRepo>,
    pub scopes: Arc<dyn ScopeRepo>,
    pub queue: Arc<dyn BuildQueueRepo>,
}

impl ApiState {
    pub fn new(
        config: Arc<Config>,
        builds: Arc<dyn BuildRepo>,
        logs: Arc<dyn LogStore>,
        audit: Arc<dyn AuditLog>,
        principals: Arc<dyn PrincipalRepo>,
        provenance: Arc<dyn ProvenanceRepo>,
        team: Arc<dyn TeamRepo>,
        scopes: Arc<dyn ScopeRepo>,
        queue: Arc<dyn BuildQueueRepo>,
        drift: Arc<dyn DriftDetector>,
        toolchain: Arc<Toolchain>,
    ) -> Self {
        Self {
            builds: builds.clone(),
            audit: audit.clone(),
            principals,
            provenance,
            team,
            scopes,
            queue,
            drift,
            logs,
            toolchain,
            config,
        }
    }

    pub async fn orchestrator(&self) -> BuildOrchestrator {
        let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
        let bundled_prefix = self.toolchain.prefix().map(|p| p.to_path_buf());
        let signer = Arc::new(CosignSigner::new(
            runner.clone(),
            CosignConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        ));

        let registry_auth = forge_core::registry::resolve(runner.as_ref(), &self.config.registry.auth)
            .await
            .unwrap_or_default();
        BuildOrchestrator {
            builder: Arc::new(BuildkitBuilder::new(
                runner.clone(),
                BuildkitConfig {
                    addr: self.config.buildkit.addr.clone(),
                    bundled_prefix: bundled_prefix.clone(),
                    registry_target: self.config.registry.default_target.clone(),
                    push: self.config.registry.default_push,
                    registry_auth,
                    ..Default::default()
                },
            )),
            scanner: make_scanner(&self.toolchain),
            sbom: Arc::new(SyftSbomGenerator::new(
                runner.clone(),
                SyftConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            )),
            signer: signer.clone(),
            attestor: Some(signer.clone()),
            policy: Arc::new(OpaPolicyEngine::new(
                runner,
                OpaConfig {
                    bundled_prefix,
                    ..Default::default()
                },
            )),
            verifier: Some(signer.clone()),
            provenance: Some(self.provenance.clone()),
            repo: self.builds.clone(),
            logs: self.logs.clone(),
        }
    }

    pub fn start_drift_scheduler(&self) -> Option<tokio::task::JoinHandle<()>> {
        if !self.config.drift.scheduler_enabled {
            return None;
        }
        let interval = Duration::from_secs(self.config.drift.interval_seconds.max(1));
        let detector = self.drift.clone();
        let builds = self.builds.clone();
        Some(tokio::spawn(async move {
            run_scheduler(detector, builds, interval).await;
        }))
    }

    pub fn default_project_id(&self) -> &'static str {
        "default-project"
    }
}

pub fn make_scanner(toolchain: &Arc<Toolchain>) -> Arc<dyn forge_core::tooling::Scanner> {
    let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
    let bundled_prefix = toolchain.prefix().map(|p| p.to_path_buf());
    Arc::new(MergedScanner {
        primary: Arc::new(TrivyScanner::new(
            runner.clone(),
            TrivyConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        )),
        secondary: Arc::new(GrypeScanner::new(
            runner,
            GrypeConfig {
                bundled_prefix,
                ..Default::default()
            },
        )),
    })
}
