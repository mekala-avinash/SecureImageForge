//! Repository for build records. Keeps mutation localized so the rest of the
//! codebase can stay immutable.

use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::domain::{Architecture, BuildArtifact, BuildRecord, BuildStatus, Sbom, ScanResult};
use crate::storage::Storage;
use crate::{Error, Result};

#[async_trait::async_trait]
pub trait BuildRepo: Send + Sync {
    async fn insert(&self, record: &BuildRecord) -> Result<()>;
    async fn insert_for_project(&self, record: &BuildRecord, project_id: &str) -> Result<()>;
    async fn update_status(
        &self,
        id: Uuid,
        status: BuildStatus,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
        log_path: Option<&str>,
    ) -> Result<()>;
    async fn save_scan(&self, build_id: Uuid, scan: &ScanResult) -> Result<()>;
    async fn save_sbom(&self, build_id: Uuid, sbom: &Sbom) -> Result<()>;
    async fn save_artifact(&self, build_id: Uuid, artifact: &BuildArtifact) -> Result<()>;
    async fn list(&self, limit: i64) -> Result<Vec<BuildSummary>>;
    async fn list_project(&self, project_id: &str, limit: i64) -> Result<Vec<BuildSummary>>;
    async fn get_summary(&self, build_id: Uuid) -> Result<Option<BuildSummary>>;
    async fn get_summary_in_project(&self, project_id: &str, build_id: Uuid) -> Result<Option<BuildSummary>>;
    async fn get_record(&self, build_id: Uuid) -> Result<Option<BuildRecord>>;
    async fn get_record_in_project(&self, project_id: &str, build_id: Uuid) -> Result<Option<BuildRecord>>;
    async fn list_artifacts(&self, build_id: Uuid) -> Result<Vec<BuildArtifact>>;
    async fn drift_targets(&self, limit: i64) -> Result<Vec<(Uuid, String)>>;
    async fn get_log_path(&self, build_id: Uuid) -> Result<Option<String>>;
    async fn get_project_id(&self, build_id: Uuid) -> Result<Option<String>>;
    async fn get_sbom(&self, build_id: Uuid) -> Result<Option<Sbom>>;
    async fn get_scan(&self, build_id: Uuid) -> Result<Option<ScanResult>>;
}

#[derive(Clone)]
pub struct SqliteBuildRepo {
    storage: Storage,
}

impl SqliteBuildRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

#[async_trait::async_trait]
impl BuildRepo for SqliteBuildRepo {
    async fn insert(&self, record: &BuildRecord) -> Result<()> {
        self.insert_for_project(record, "default-project").await
    }

    async fn insert_for_project(&self, record: &BuildRecord, project_id: &str) -> Result<()> {
        let spec_json = serde_json::to_string(&record.spec)?;
        sqlx::query(
            r#"INSERT INTO builds (id, name, runtime, base_image, status, spec_json,
                                   created_at, started_at, finished_at, log_path, project_id)
               VALUES (?,?,?,?,?,?,?,?,?,?,?)"#,
        )
        .bind(record.id.to_string())
        .bind(&record.spec.name)
        .bind(format!("{:?}", record.spec.runtime).to_lowercase())
        .bind(format!("{:?}", record.spec.base_image).to_lowercase())
        .bind(format!("{:?}", record.status).to_lowercase())
        .bind(spec_json)
        .bind(record.created_at.to_rfc3339())
        .bind(record.started_at.map(|d| d.to_rfc3339()))
        .bind(record.finished_at.map(|d| d.to_rfc3339()))
        .bind(record.log_path.clone())
        .bind(project_id)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: BuildStatus,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
        log_path: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE builds
               SET status = ?,
                   started_at = COALESCE(?, started_at),
                   finished_at = COALESCE(?, finished_at),
                   log_path = COALESCE(?, log_path)
               WHERE id = ?"#,
        )
        .bind(format!("{status:?}").to_lowercase())
        .bind(started_at.map(|d| d.to_rfc3339()))
        .bind(finished_at.map(|d| d.to_rfc3339()))
        .bind(log_path)
        .bind(id.to_string())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn save_scan(&self, build_id: Uuid, scan: &ScanResult) -> Result<()> {
        let findings = serde_json::to_string(&scan.findings)?;
        sqlx::query(
            r#"INSERT INTO scans (build_id, scanner, scanned_at, findings)
               VALUES (?,?,?,?)
               ON CONFLICT(build_id) DO UPDATE SET
                   scanner = excluded.scanner,
                   scanned_at = excluded.scanned_at,
                   findings = excluded.findings"#,
        )
        .bind(build_id.to_string())
        .bind(&scan.scanner)
        .bind(scan.scanned_at.to_rfc3339())
        .bind(findings)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn save_sbom(&self, build_id: Uuid, sbom: &Sbom) -> Result<()> {
        let document = serde_json::to_string(&sbom.document)?;
        sqlx::query(
            r#"INSERT INTO sboms (build_id, format, document)
               VALUES (?,?,?)
               ON CONFLICT(build_id) DO UPDATE SET
                   format = excluded.format,
                   document = excluded.document"#,
        )
        .bind(build_id.to_string())
        .bind(&sbom.format)
        .bind(document)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn save_artifact(&self, build_id: Uuid, artifact: &BuildArtifact) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO artifacts (build_id, digest, registry_ref, bytes, architecture)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(build_id, digest, architecture) DO UPDATE SET
                   registry_ref = excluded.registry_ref,
                   bytes = excluded.bytes"#,
        )
        .bind(build_id.to_string())
        .bind(&artifact.digest)
        .bind(&artifact.registry_ref)
        .bind(artifact.bytes as i64)
        .bind(format!("{:?}", artifact.architecture).to_lowercase())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn list(&self, limit: i64) -> Result<Vec<BuildSummary>> {
        self.list_project("default-project", limit).await
    }

    async fn list_project(&self, project_id: &str, limit: i64) -> Result<Vec<BuildSummary>> {
        let rows = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds
               WHERE project_id = ?
               ORDER BY created_at DESC LIMIT ?"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| BuildSummary {
                id: r.get::<String, _>("id"),
                name: r.get::<String, _>("name"),
                runtime: r.get::<String, _>("runtime"),
                base_image: r.get::<String, _>("base_image"),
                status: r.get::<String, _>("status"),
                created_at: r.get::<String, _>("created_at"),
                finished_at: r.get::<Option<String>, _>("finished_at"),
            })
            .collect())
    }

    async fn get_summary(&self, build_id: Uuid) -> Result<Option<BuildSummary>> {
        self.get_summary_in_project("default-project", build_id)
            .await
    }

    async fn get_summary_in_project(
        &self,
        project_id: &str,
        build_id: Uuid,
    ) -> Result<Option<BuildSummary>> {
        let row = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds WHERE id = ? AND project_id = ?"#,
        )
        .bind(build_id.to_string())
        .bind(project_id)
        .fetch_optional(self.storage.pool())
        .await?;
        Ok(row.map(|r| BuildSummary {
            id: r.get::<String, _>("id"),
            name: r.get::<String, _>("name"),
            runtime: r.get::<String, _>("runtime"),
            base_image: r.get::<String, _>("base_image"),
            status: r.get::<String, _>("status"),
            created_at: r.get::<String, _>("created_at"),
            finished_at: r.get::<Option<String>, _>("finished_at"),
        }))
    }

    async fn get_record(&self, build_id: Uuid) -> Result<Option<BuildRecord>> {
        self.get_record_in_project("default-project", build_id)
            .await
    }

    async fn get_record_in_project(
        &self,
        project_id: &str,
        build_id: Uuid,
    ) -> Result<Option<BuildRecord>> {
        let row = sqlx::query(
            r#"SELECT id, status, spec_json, created_at, started_at, finished_at, log_path
               FROM builds WHERE id = ? AND project_id = ?"#,
        )
        .bind(build_id.to_string())
        .bind(project_id)
        .fetch_optional(self.storage.pool())
        .await?;
        let Some(r) = row else {
            return Ok(None);
        };
        let spec_json: String = r.get("spec_json");
        let spec = serde_json::from_str(&spec_json)?;
        let id: String = r.get("id");
        let status: String = r.get("status");
        let created_at: String = r.get("created_at");
        let started_at: Option<String> = r.get("started_at");
        let finished_at: Option<String> = r.get("finished_at");
        let artifacts = self.list_artifacts(build_id).await?;
        Ok(Some(BuildRecord {
            id: Uuid::parse_str(&id).map_err(|e| Error::Internal(anyhow::anyhow!(e)))?,
            spec,
            status: parse_status(&status)?,
            created_at: parse_rfc3339(&created_at)?,
            started_at: parse_optional_rfc3339(started_at)?,
            finished_at: parse_optional_rfc3339(finished_at)?,
            artifacts,
            scan: self.get_scan(build_id).await?,
            sbom: self.get_sbom(build_id).await?,
            log_path: r.get("log_path"),
        }))
    }

    async fn list_artifacts(&self, build_id: Uuid) -> Result<Vec<BuildArtifact>> {
        let rows = sqlx::query(
            r#"SELECT digest, registry_ref, bytes, architecture FROM artifacts WHERE build_id = ?"#,
        )
        .bind(build_id.to_string())
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let architecture: String = r.get("architecture");
                Ok(BuildArtifact {
                    digest: r.get("digest"),
                    registry_ref: r.get("registry_ref"),
                    bytes: r.get::<i64, _>("bytes").max(0) as u64,
                    architecture: parse_architecture(&architecture)?,
                })
            })
            .collect()
    }

    async fn drift_targets(&self, limit: i64) -> Result<Vec<(Uuid, String)>> {
        let rows = sqlx::query(
            r#"SELECT b.id, COALESCE(a.registry_ref, a.digest) AS image_ref
               FROM builds b
               JOIN artifacts a ON a.build_id = b.id
               WHERE b.status = 'succeeded'
               ORDER BY b.finished_at DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let id: String = r.get("id");
                let image_ref: String = r.get("image_ref");
                Ok((
                    Uuid::parse_str(&id).map_err(|e| Error::Internal(anyhow::anyhow!(e)))?,
                    image_ref,
                ))
            })
            .collect()
    }

    async fn get_log_path(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT log_path FROM builds WHERE id = ?"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.and_then(|r| r.get::<Option<String>, _>("log_path")))
    }

    async fn get_project_id(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT project_id FROM builds WHERE id = ?"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.map(|r| r.get("project_id")))
    }

    async fn get_sbom(&self, build_id: Uuid) -> Result<Option<Sbom>> {
        let row = sqlx::query(r#"SELECT format, document FROM sboms WHERE build_id = ?"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        match row {
            None => Ok(None),
            Some(r) => {
                let format: String = r.get("format");
                let doc_str: String = r.get("document");
                let document: serde_json::Value = serde_json::from_str(&doc_str)?;
                Ok(Some(Sbom { format, document }))
            }
        }
    }

    async fn get_scan(&self, build_id: Uuid) -> Result<Option<ScanResult>> {
        let row =
            sqlx::query(r#"SELECT scanner, scanned_at, findings FROM scans WHERE build_id = ?"#)
                .bind(build_id.to_string())
                .fetch_optional(self.storage.pool())
                .await?;
        match row {
            None => Ok(None),
            Some(r) => {
                let scanner: String = r.get("scanner");
                let scanned_at: String = r.get("scanned_at");
                let findings: String = r.get("findings");
                let findings = serde_json::from_str(&findings)?;
                let scanned_at = DateTime::parse_from_rfc3339(&scanned_at)
                    .map_err(|e| Error::Internal(anyhow::anyhow!(e)))?
                    .with_timezone(&Utc);
                Ok(Some(ScanResult {
                    scanner,
                    scanned_at,
                    findings,
                }))
            }
        }
    }
}

fn parse_rfc3339(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|e| Error::Internal(anyhow::anyhow!(e)))
        .map(|d| d.with_timezone(&Utc))
}

fn parse_optional_rfc3339(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.as_deref().map(parse_rfc3339).transpose()
}

fn parse_status(value: &str) -> Result<BuildStatus> {
    match value {
        "pending" => Ok(BuildStatus::Pending),
        "running" => Ok(BuildStatus::Running),
        "succeeded" => Ok(BuildStatus::Succeeded),
        "failed" => Ok(BuildStatus::Failed),
        "cancelled" => Ok(BuildStatus::Cancelled),
        other => Err(Error::Internal(anyhow::anyhow!(
            "unknown build status: {other}"
        ))),
    }
}

fn parse_architecture(value: &str) -> Result<Architecture> {
    match value {
        "amd64" => Ok(Architecture::Amd64),
        "arm64" => Ok(Architecture::Arm64),
        other => Err(Error::Internal(anyhow::anyhow!(
            "unknown architecture: {other}"
        ))),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BuildSummary {
    pub id: String,
    pub name: String,
    pub runtime: String,
    pub base_image: String,
    pub status: String,
    pub created_at: String,
    pub finished_at: Option<String>,
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
            runtime: Runtime::Go,
            base_image: BaseImage::Alpine,
            architectures: BTreeSet::from([Architecture::Amd64]),
            compliance: BTreeSet::from([ComplianceProfile::Cis]),
            hardening: HardeningOptions::strict(),
            generate_sbom: true,
            sign: true,
        };
        BuildRecord::new(spec)
    }

    #[tokio::test]
    async fn insert_and_list_round_trip() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = SqliteBuildRepo::new(storage);
        let r = record();
        repo.insert(&r).await.unwrap();
        let list = repo.list(10).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "alpha");
        assert_eq!(list[0].status, "pending");
    }

    #[tokio::test]
    async fn save_scan_then_get() {
        use crate::domain::{Severity, Vulnerability};
        let storage = Storage::open_memory().await.unwrap();
        let repo = SqliteBuildRepo::new(storage);
        let r = record();
        repo.insert(&r).await.unwrap();
        let scan = ScanResult {
            scanner: "trivy".into(),
            scanned_at: Utc::now(),
            findings: vec![Vulnerability {
                id: "CVE-1".into(),
                package: "openssl".into(),
                installed_version: "1".into(),
                fixed_version: None,
                severity: Severity::High,
                title: None,
            }],
        };
        repo.save_scan(r.id, &scan).await.unwrap();
        let got = repo.get_scan(r.id).await.unwrap().unwrap();
        assert_eq!(got.findings.len(), 1);
        assert_eq!(got.scanner, "trivy");
    }

    #[tokio::test]
    async fn get_summary_returns_inserted_row() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = SqliteBuildRepo::new(storage);
        let r = record();
        repo.insert(&r).await.unwrap();
        let s = repo.get_summary(r.id).await.unwrap().unwrap();
        assert_eq!(s.name, "alpha");
        assert_eq!(s.runtime, "go");
        let absent = repo.get_summary(uuid::Uuid::new_v4()).await.unwrap();
        assert!(absent.is_none());
    }

    #[tokio::test]
    async fn save_and_get_sbom() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = SqliteBuildRepo::new(storage);
        let r = record();
        repo.insert(&r).await.unwrap();
        let sbom = Sbom {
            format: "cyclonedx".into(),
            document: serde_json::json!({"bomFormat":"CycloneDX","components":[{"name":"x"}]}),
        };
        repo.save_sbom(r.id, &sbom).await.unwrap();
        let got = repo.get_sbom(r.id).await.unwrap().unwrap();
        assert_eq!(got.format, "cyclonedx");
        assert_eq!(got.document["components"][0]["name"], "x");
    }

    #[tokio::test]
    async fn update_status_persists_log_path() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = SqliteBuildRepo::new(storage);
        let r = record();
        repo.insert(&r).await.unwrap();
        repo.update_status(
            r.id,
            BuildStatus::Running,
            None,
            None,
            Some("/tmp/some.log"),
        )
        .await
        .unwrap();
        let got = repo.get_log_path(r.id).await.unwrap();
        assert_eq!(got.as_deref(), Some("/tmp/some.log"));
    }
}
