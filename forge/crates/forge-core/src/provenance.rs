//! SLSA L3 provenance via in-toto Statements. Generated for every successful
//! build and persisted to `provenance` table; `cosign attest` (via the
//! existing CosignSigner) signs the statement and pushes it as an OCI
//! attestation alongside the artifact.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::domain::{BuildRecord, BuildSpec};
use crate::storage::Storage;
use crate::Result;

const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
const PREDICATE_TYPE: &str = "https://slsa.dev/provenance/v1";
const BUILDER_ID: &str = "https://secureimage-forge.dev/buildkit-orchestrator/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<Subject>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: Predicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub build_definition: BuildDefinition,
    pub run_details: RunDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDefinition {
    pub build_type: String,
    pub external_parameters: serde_json::Value,
    pub internal_parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDetails {
    pub builder: serde_json::Value,
    pub metadata: RunMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetadata {
    pub invocation_id: String,
    pub started_on: Option<DateTime<Utc>>,
    pub finished_on: Option<DateTime<Utc>>,
}

/// Build a Statement from a finished build record + image digest. Pure
/// function so it's trivially testable.
pub fn build_statement(record: &BuildRecord, image_ref: &str, image_digest: &str) -> Statement {
    let digest = image_digest
        .strip_prefix("sha256:")
        .unwrap_or(image_digest)
        .to_string();
    Statement {
        statement_type: STATEMENT_TYPE.into(),
        subject: vec![Subject {
            name: image_ref.into(),
            digest: json!({"sha256": digest}),
        }],
        predicate_type: PREDICATE_TYPE.into(),
        predicate: Predicate {
            build_definition: BuildDefinition {
                build_type: "https://secureimage-forge.dev/build/v1".into(),
                external_parameters: spec_to_json(&record.spec),
                internal_parameters: json!({"orchestrator": "forge-core"}),
            },
            run_details: RunDetails {
                builder: json!({"id": BUILDER_ID, "version": env!("CARGO_PKG_VERSION")}),
                metadata: RunMetadata {
                    invocation_id: record.id.to_string(),
                    started_on: record.started_at,
                    finished_on: record.finished_at,
                },
            },
        },
    }
}

fn spec_to_json(spec: &BuildSpec) -> serde_json::Value {
    json!({
        "name": spec.name,
        "runtime": format!("{:?}", spec.runtime).to_lowercase(),
        "base_image": format!("{:?}", spec.base_image).to_lowercase(),
        "compliance": spec.compliance.iter().map(|c| format!("{c:?}").to_lowercase()).collect::<Vec<_>>(),
        "architectures": spec.architectures.iter().map(|a| format!("{a:?}").to_lowercase()).collect::<Vec<_>>(),
        "sign": spec.sign,
        "generate_sbom": spec.generate_sbom,
    })
}

#[derive(Clone)]
pub struct ProvenanceRepo {
    pub(crate) storage: Storage,
}

impl ProvenanceRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn save(
        &self,
        build_id: Uuid,
        statement: &Statement,
        bundle_path: Option<&str>,
    ) -> Result<()> {
        let predicate = serde_json::to_string(statement)?;
        sqlx::query(
            r#"INSERT INTO provenance (build_id, predicate, attested_at, bundle_path)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(build_id) DO UPDATE SET
                   predicate = excluded.predicate,
                   attested_at = excluded.attested_at,
                   bundle_path = excluded.bundle_path"#,
        )
        .bind(build_id.to_string())
        .bind(predicate)
        .bind(Utc::now().to_rfc3339())
        .bind(bundle_path)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn get(&self, build_id: Uuid) -> Result<Option<Statement>> {
        let row = sqlx::query(r#"SELECT predicate FROM provenance WHERE build_id = ?"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        match row {
            None => Ok(None),
            Some(r) => {
                let s: String = r.get("predicate");
                Ok(Some(serde_json::from_str(&s)?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Architecture, BaseImage, BuildSpec, ComplianceProfile, HardeningOptions, Runtime,
    };
    use std::collections::BTreeSet;

    fn record() -> BuildRecord {
        let spec = BuildSpec {
            name: "alpha".into(),
            runtime: Runtime::Java,
            base_image: BaseImage::Distroless,
            architectures: BTreeSet::from([Architecture::Amd64, Architecture::Arm64]),
            compliance: BTreeSet::from([ComplianceProfile::Cis]),
            hardening: HardeningOptions::strict(),
            generate_sbom: true,
            sign: true,
        };
        BuildRecord::new(spec)
    }

    #[test]
    fn statement_includes_subject_digest_and_builder() {
        let r = record();
        let s = build_statement(&r, "ghcr.io/x/y:latest", "sha256:abcdef");
        assert_eq!(s.statement_type, STATEMENT_TYPE);
        assert_eq!(s.predicate_type, PREDICATE_TYPE);
        assert_eq!(s.subject.len(), 1);
        assert_eq!(s.subject[0].digest["sha256"], "abcdef");
        let builder_id = s.predicate.run_details.builder["id"].as_str().unwrap();
        assert!(builder_id.starts_with("https://"));
    }

    #[tokio::test]
    async fn save_and_get_round_trip() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = ProvenanceRepo::new(storage);
        let r = record();

        // foreign key requires the builds row exist; reuse BuildRepo for that.
        let storage_for_builds = repo.storage.clone();
        let build_repo = crate::repo::BuildRepo::new(storage_for_builds);
        build_repo.insert(&r).await.unwrap();

        let stmt = build_statement(&r, "img:tag", "sha256:deadbeef");
        repo.save(r.id, &stmt, Some("/bundles/x.json"))
            .await
            .unwrap();
        let got = repo.get(r.id).await.unwrap().unwrap();
        assert_eq!(got.subject[0].name, "img:tag");
    }
}
