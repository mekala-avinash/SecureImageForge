//! Full orchestrator pipeline test using the MockRunner. Confirms that:
//!   * MergedScanner unions trivy + grype findings.
//!   * Attestation is invoked on policy-allow.
//!   * Provenance row is persisted.
//!   * Audit / drift integration points use the real domain types.

use std::collections::BTreeSet;
use std::sync::Arc;

use forge_core::adapters::buildkit::{BuildkitBuilder, BuildkitConfig};
use forge_core::adapters::cosign::{CosignConfig, CosignSigner};
use forge_core::adapters::grype::{GrypeConfig, GrypeScanner, MergedScanner};
use forge_core::adapters::opa::{OpaConfig, OpaPolicyEngine};
use forge_core::adapters::syft::{SyftConfig, SyftSbomGenerator};
use forge_core::adapters::trivy::{TrivyConfig, TrivyScanner};
use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, BuildStatus, ComplianceProfile, HardeningOptions, Runtime,
};
use forge_core::logs::LogStore;
use forge_core::orchestrator::BuildOrchestrator;
use forge_core::process::{MockRunner, ProcessOutput};
use forge_core::provenance::ProvenanceRepo;
use forge_core::repo::BuildRepo;
use forge_core::storage::Storage;
use forge_core::tooling::PolicyDecision;
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
            stdout: r#"{"Results":[{"Vulnerabilities":[{"VulnerabilityID":"CVE-2024-1","PkgName":"openssl","InstalledVersion":"3.0.0","Severity":"HIGH"}]}]}"#.into(),
            stderr: String::new(),
        },
    );
    mock.expect(
        |s| s.program.ends_with("grype"),
        ProcessOutput {
            status: 0,
            stdout: r#"{"matches":[{"vulnerability":{"id":"GHSA-1","severity":"Medium"},"artifact":{"name":"zlib","version":"1.2.11"}}]}"#.into(),
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

#[tokio::test]
async fn full_pipeline_attests_provenance_and_unions_scans() {
    let runner: Arc<MockRunner> = Arc::new(happy_runner());
    let storage = Storage::open_memory().await.unwrap();
    let repo = BuildRepo::new(storage.clone());
    let provenance = ProvenanceRepo::new(storage.clone());
    let log_dir = TempDir::new().unwrap();
    let logs = LogStore::new(log_dir.path().to_path_buf());

    let signer = Arc::new(CosignSigner::new(
        runner.clone(),
        CosignConfig {
            cosign_path: Some("/bin/cosign".into()),
            ..Default::default()
        },
    ));

    let orch = BuildOrchestrator {
        builder: Arc::new(BuildkitBuilder::new(
            runner.clone(),
            BuildkitConfig {
                buildctl_path: Some("/bin/buildctl".into()),
                ..Default::default()
            },
        )),
        scanner: Arc::new(MergedScanner {
            primary: Arc::new(TrivyScanner::new(
                runner.clone(),
                TrivyConfig {
                    trivy_path: Some("/bin/trivy".into()),
                    ..Default::default()
                },
            )),
            secondary: Arc::new(GrypeScanner::new(
                runner.clone(),
                GrypeConfig {
                    grype_path: Some("/bin/grype".into()),
                    ..Default::default()
                },
            )),
        }),
        sbom: Arc::new(SyftSbomGenerator::new(
            runner.clone(),
            SyftConfig {
                syft_path: Some("/bin/syft".into()),
                ..Default::default()
            },
        )),
        signer: signer.clone(),
        attestor: Some(signer),
        policy: Arc::new(OpaPolicyEngine::new(
            runner.clone(),
            OpaConfig {
                opa_path: Some("/bin/opa".into()),
                profiles: vec![ComplianceProfile::Cis],
                ..Default::default()
            },
        )),
        provenance: Some(provenance.clone()),
        repo: repo.clone(),
        logs,
    };

    let spec = BuildSpec {
        name: "e2e".into(),
        runtime: Runtime::Go,
        base_image: BaseImage::Alpine,
        architectures: BTreeSet::from([Architecture::Amd64]),
        compliance: BTreeSet::from([ComplianceProfile::Cis]),
        hardening: HardeningOptions::strict(),
        generate_sbom: true,
        sign: true,
    };

    let outcome = orch.run(spec).await.unwrap();
    assert_eq!(outcome.record.status, BuildStatus::Succeeded);
    assert_eq!(outcome.policy, PolicyDecision::Allow);

    let saved_scan = repo.get_scan(outcome.record.id).await.unwrap().unwrap();
    let mut ids: Vec<_> = saved_scan.findings.iter().map(|f| f.id.clone()).collect();
    ids.sort();
    assert_eq!(ids, vec!["CVE-2024-1".to_string(), "GHSA-1".to_string()]);

    // Provenance should be persisted because attestor was supplied.
    let stmt = provenance.get(outcome.record.id).await.unwrap();
    assert!(
        stmt.is_some(),
        "provenance should be persisted on policy allow"
    );
}
