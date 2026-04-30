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
    pub storage: Storage,
    pub builds: BuildRepo,
    pub logs: LogStore,
    pub audit: AuditLog,
    pub principals: PrincipalRepo,
    pub provenance: ProvenanceRepo,
    pub toolchain: Arc<Toolchain>,
    pub drift: Arc<DriftDetector>,
    pub team: TeamRepo,
    pub scopes: ScopeRepo,
    pub queue: BuildQueueRepo,
}

impl ApiState {
    pub fn new(
        config: Arc<Config>,
        storage: Storage,
        logs: LogStore,
        toolchain: Arc<Toolchain>,
    ) -> Self {
        let builds = BuildRepo::new(storage.clone());
        let audit = AuditLog::new(storage.clone());
        let drift_scanner = make_scanner(&toolchain);
        Self {
            builds: builds.clone(),
            audit: audit.clone(),
            principals: PrincipalRepo::new(storage.clone()),
            provenance: ProvenanceRepo::new(storage.clone()),
            team: TeamRepo::new(storage.clone()),
            scopes: ScopeRepo::new(storage.clone()),
            queue: BuildQueueRepo::new(storage.clone()),
            drift: Arc::new(DriftDetector {
                repo: builds,
                storage: storage.clone(),
                scanner: drift_scanner,
                audit,
            }),
            logs,
            toolchain,
            config,
            storage,
        }
    }

    pub fn orchestrator(&self) -> BuildOrchestrator {
        let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
        let bundled_prefix = self.toolchain.prefix().map(|p| p.to_path_buf());
        let signer = Arc::new(CosignSigner::new(
            runner.clone(),
            CosignConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        ));
        BuildOrchestrator {
            builder: Arc::new(BuildkitBuilder::new(
                runner.clone(),
                BuildkitConfig {
                    addr: self.config.buildkit.addr.clone(),
                    bundled_prefix: bundled_prefix.clone(),
                    registry_target: self.config.registry.default_target.clone(),
                    push: self.config.registry.default_push,
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
            attestor: Some(signer),
            policy: Arc::new(OpaPolicyEngine::new(
                runner,
                OpaConfig {
                    bundled_prefix,
                    ..Default::default()
                },
            )),
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
        Some(tokio::spawn(async move {
            run_scheduler(detector, interval).await;
        }))
    }

    pub fn default_project_id(&self) -> &'static str {
        "default-project"
    }
}

fn make_scanner(toolchain: &Arc<Toolchain>) -> Arc<dyn forge_core::tooling::Scanner> {
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
