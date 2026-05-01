//! Build orchestrator. Drives the pipeline:
//!   1. validate spec
//!   2. render Dockerfile
//!   3. invoke ImageBuilder
//!   4. invoke Scanner
//!   5. invoke SbomGenerator (optional)
//!   6. invoke PolicyEngine (gate)
//!   7. invoke Signer (optional, only on policy allow)
//!
//! Persistence happens incrementally through the supplied `BuildRepo`.

use std::sync::Arc;

use chrono::Utc;
use tracing::{info, warn};

use crate::dockerfile;
use crate::domain::{Architecture, BuildArtifact, BuildRecord, BuildSpec, BuildStatus};
use crate::logs::LogStore;
use crate::provenance::{build_statement, ProvenanceRepo};
use crate::repo::BuildRepo;
use crate::tooling::{
    Attestor, ImageBuilder, PolicyDecision, PolicyEngine, SbomGenerator, Scanner, Signer, Verifier,
};
use crate::{Error, Result};

pub struct BuildOrchestrator {
    pub builder: Arc<dyn ImageBuilder>,
    pub scanner: Arc<dyn Scanner>,
    pub sbom: Arc<dyn SbomGenerator>,
    pub signer: Arc<dyn Signer>,
    pub attestor: Option<Arc<dyn Attestor>>,
    pub policy: Arc<dyn PolicyEngine>,
    pub verifier: Option<Arc<dyn Verifier>>,
    pub provenance: Option<Arc<dyn ProvenanceRepo>>,
    pub repo: Arc<dyn BuildRepo>,
    pub logs: Arc<dyn LogStore>,
}

#[derive(Debug, Clone)]
pub struct BuildOutcome {
    pub record: BuildRecord,
    pub policy: PolicyDecision,
}

impl BuildOrchestrator {
    pub async fn run(&self, spec: BuildSpec) -> Result<BuildOutcome> {
        spec.validate()?;
        let record = BuildRecord::new(spec.clone());
        self.repo.insert(&record).await?;
        self.run_record(record).await
    }

    pub async fn run_existing(&self, record: BuildRecord) -> Result<BuildOutcome> {
        record.spec.validate()?;
        self.run_record(record).await
    }

    async fn run_record(&self, record: BuildRecord) -> Result<BuildOutcome> {
        let spec = record.spec.clone();

        let started = Utc::now();
        self.repo
            .update_status(record.id, BuildStatus::Running, Some(started), None, None)
            .await?;

        let dockerfile = dockerfile::render(&spec);

        if let Some(verifier) = &self.verifier {
            let base_ref = dockerfile::base_reference(spec.runtime, spec.base_image);
            info!(build_id = %record.id, base = %base_ref, "verifying upstream base image");
            if let Err(e) = verifier.verify(base_ref).await {
                self.fail(&record, &e).await;
                return Err(e);
            }
        }

        info!(build_id = %record.id, "running buildkit");
        let built = match self.builder.build(&spec, &dockerfile).await {
            Ok(b) => b,
            Err(e) => {
                self.fail(&record, &e).await;
                return Err(e);
            }
        };

        let log_path = self.logs.write(record.id, &built.log).await?;
        let log_path_str = log_path.display().to_string();
        self.repo
            .update_status(
                record.id,
                BuildStatus::Running,
                None,
                None,
                Some(&log_path_str),
            )
            .await?;

        let artifact = BuildArtifact {
            digest: built.digest.clone(),
            registry_ref: Some(built.reference.clone()),
            bytes: 0,
            architecture: spec
                .architectures
                .iter()
                .next()
                .copied()
                .unwrap_or(Architecture::Amd64),
        };
        self.repo.save_artifact(record.id, &artifact).await?;

        info!(build_id = %record.id, image = %built.reference, "scanning image");
        let scan = self.scanner.scan(&built.reference).await?;
        self.repo.save_scan(record.id, &scan).await?;

        if spec.generate_sbom {
            info!(build_id = %record.id, "generating sbom");
            let sbom = self.sbom.generate(&built.reference).await?;
            self.repo.save_sbom(record.id, &sbom).await?;
        }

        let input = crate::adapters::opa::build_input(&spec, Some(&scan));
        let decision = self.policy.evaluate(input).await?;

        let final_status = match &decision {
            PolicyDecision::Allow => {
                if spec.sign {
                    info!(build_id = %record.id, "signing artifact");
                    self.signer.sign(&built.reference).await?;
                }
                BuildStatus::Succeeded
            }
            PolicyDecision::Deny { reasons } => {
                warn!(build_id = %record.id, reasons = ?reasons, "policy denied");
                BuildStatus::Failed
            }
        };

        let finished = Utc::now();
        let final_record_for_provenance = BuildRecord {
            status: final_status,
            started_at: Some(started),
            finished_at: Some(finished),
            artifacts: vec![artifact.clone()],
            scan: Some(scan.clone()),
            log_path: Some(log_path_str.clone()),
            ..record.clone()
        };

        if final_status == BuildStatus::Succeeded {
            if let Some(provenance) = &self.provenance {
                let statement = build_statement(
                    &final_record_for_provenance,
                    &built.reference,
                    &built.digest,
                );
                let predicate_json = serde_json::to_string(&statement)?;
                if let Some(attestor) = &self.attestor {
                    info!(build_id = %record.id, "attesting provenance");
                    attestor.attest(&built.reference, &predicate_json).await?;
                }
                provenance.save(record.id, &statement, None).await?;
            }
        }

        self.repo
            .update_status(record.id, final_status, None, Some(finished), None)
            .await?;

        let final_record = BuildRecord {
            status: final_status,
            started_at: Some(started),
            finished_at: Some(finished),
            artifacts: vec![artifact],
            scan: Some(scan),
            log_path: Some(log_path_str),
            ..record
        };

        Ok(BuildOutcome {
            record: final_record,
            policy: decision,
        })
    }

    async fn fail(&self, record: &BuildRecord, err: &Error) {
        let finished = Utc::now();
        let _ = self
            .repo
            .update_status(record.id, BuildStatus::Failed, None, Some(finished), None)
            .await;
        warn!(build_id = %record.id, error = %err, "build failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::buildkit::{BuildkitBuilder, BuildkitConfig};
    use crate::adapters::cosign::{CosignConfig, CosignSigner};
    use crate::adapters::opa::{OpaConfig, OpaPolicyEngine};
    use crate::adapters::syft::{SyftConfig, SyftSbomGenerator};
    use crate::adapters::trivy::{TrivyConfig, TrivyScanner};
    use crate::domain::{Architecture, BaseImage, ComplianceProfile, HardeningOptions, Runtime};
    use crate::logs::LogStore;
    use crate::process::{MockRunner, ProcessOutput};
    use crate::storage::Storage;
    use std::collections::BTreeSet;
    use tempfile::TempDir;

    fn happy_runner() -> MockRunner {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("buildctl"),
            ProcessOutput {
                status: 0,
                stdout: "exporting sha256:aaaabbbbccccddddeeeeffff0000111122223333444455556666777788889999 done\n".into(),
                stderr: String::new(),
            },
        );
        mock.expect(
            |s| s.program.ends_with("trivy"),
            ProcessOutput {
                status: 0,
                stdout: r#"{"Results":[{"Vulnerabilities":[]}]}"#.into(),
                stderr: String::new(),
            },
        );
        mock.expect(
            |s| s.program.ends_with("syft"),
            ProcessOutput {
                status: 0,
                stdout: r#"{"bomFormat":"CycloneDX","components":[]}"#.into(),
                stderr: String::new(),
            },
        );
        mock.expect(
            |s| s.program.ends_with("opa"),
            ProcessOutput {
                status: 0,
                stdout: r#"{"result":[{"expressions":[{"value":[]}]}]}"#.into(),
                stderr: String::new(),
            },
        );
        mock.expect(
            |s| s.program.ends_with("cosign"),
            ProcessOutput {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        mock
    }

    fn spec() -> BuildSpec {
        BuildSpec {
            name: "happy".into(),
            runtime: Runtime::Go,
            base_image: BaseImage::Alpine,
            architectures: BTreeSet::from([Architecture::Amd64]),
            compliance: BTreeSet::from([ComplianceProfile::Cis]),
            hardening: HardeningOptions::strict(),
            generate_sbom: true,
            sign: true,
        }
    }

    #[tokio::test]
    async fn full_pipeline_succeeds_when_policy_allows() {
        let runner: Arc<MockRunner> = Arc::new(happy_runner());
        let storage = Storage::open_memory().await.unwrap();
        let repo = Arc::new(crate::repo::SqliteBuildRepo::new(storage)) as Arc<dyn BuildRepo>;
        let log_dir = TempDir::new().unwrap();
        let logs = Arc::new(crate::logs::FileLogStore::new(log_dir.path().to_path_buf())) as Arc<dyn LogStore>;

        let orch = BuildOrchestrator {
            builder: Arc::new(BuildkitBuilder::new(
                runner.clone(),
                BuildkitConfig {
                    addr: "mock".into(),
                    buildctl_path: Some("/bin/buildctl".into()),
                    ..Default::default()
                },
            )),
            scanner: Arc::new(TrivyScanner::new(
                runner.clone(),
                TrivyConfig {
                    trivy_path: Some("/bin/trivy".into()),
                    ..Default::default()
                },
            )),
            sbom: Arc::new(SyftSbomGenerator::new(
                runner.clone(),
                SyftConfig {
                    syft_path: Some("/bin/syft".into()),
                    ..Default::default()
                },
            )),
            signer: Arc::new(CosignSigner::new(
                runner.clone(),
                CosignConfig {
                    cosign_path: Some("/bin/cosign".into()),
                    ..Default::default()
                },
            )),
            attestor: None,
            policy: Arc::new(OpaPolicyEngine::new(
                runner.clone(),
                OpaConfig {
                    opa_path: Some("/bin/opa".into()),
                    profiles: vec![ComplianceProfile::Cis],
                    ..Default::default()
                },
            )),
            verifier: None,
            provenance: None,
            repo,
            logs,
        };

        let outcome = orch.run(spec()).await.unwrap();
        assert_eq!(outcome.record.status, BuildStatus::Succeeded);
        assert_eq!(outcome.policy, PolicyDecision::Allow);
    }
}
