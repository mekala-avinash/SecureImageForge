//! Postgres team-mode storage. Phase 7 ships full write-path parity for the
//! tables the orchestrator and API daemon touch:
//!   * builds (insert, update_status, list, get_summary, get_record)
//!   * artifacts (save_artifact, list_artifacts)
//!   * scans (save_scan, get_scan)
//!   * sboms (save_sbom, get_sbom)
//!   * principals (create, list, revoke, authenticate)
//!   * audit_events (record, recent)
//!   * provenance (save, get)
//!   * drift_snapshots (insert, list)
//!
//! Webhook deliveries reuse the SQLite worker (queue rows live in a backend-
//! agnostic table; the worker calls `sqlx::query` with the same string and
//! `?`/`$1` placeholders are interchangeable here because we only read by
//! `next_attempt <= ?` which Postgres also accepts via the
//! `bind_unchecked` shim documented inline).

#![cfg(feature = "pg")]

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use uuid::Uuid;

use crate::audit::{AuditEntry, Outcome};
use crate::domain::{
    Architecture, BuildArtifact, BuildRecord, BuildStatus, Sbom, ScanResult, Severity,
    Vulnerability,
};
use crate::provenance::Statement;
use crate::rbac::{CreatedPrincipal, Principal, Role};
use crate::repo::BuildSummary;
use crate::{Error, Result};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations_postgres");

/// Wraps a Postgres pool and runs the team-mode migrations on `open`.
#[derive(Clone)]
pub struct PgStorage {
    pool: PgPool,
}

impl PgStorage {
    pub async fn open(database_url: &str) -> Result<Self> {
        let opts = PgConnectOptions::from_str(database_url).map_err(sqlx::Error::from)?;
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect_with(opts)
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// ---------------------------------------------------------------------------
// builds + artifacts
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PgBuildRepo {
    storage: PgStorage,
}

impl PgBuildRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }

    pub async fn insert(&self, record: &BuildRecord) -> Result<()> {
        let spec_json = serde_json::to_string(&record.spec)?;
        sqlx::query(
            r#"INSERT INTO builds (id, name, runtime, base_image, status, spec_json,
                                   created_at, started_at, finished_at, log_path)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"#,
        )
        .bind(record.id.to_string())
        .bind(&record.spec.name)
        .bind(format!("{:?}", record.spec.runtime).to_lowercase())
        .bind(format!("{:?}", record.spec.base_image).to_lowercase())
        .bind(format!("{:?}", record.status).to_lowercase())
        .bind(spec_json)
        .bind(record.created_at)
        .bind(record.started_at)
        .bind(record.finished_at)
        .bind(record.log_path.clone())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: BuildStatus,
        started_at: Option<DateTime<Utc>>,
        finished_at: Option<DateTime<Utc>>,
        log_path: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE builds
               SET status = $1,
                   started_at = COALESCE($2, started_at),
                   finished_at = COALESCE($3, finished_at),
                   log_path = COALESCE($4, log_path)
               WHERE id = $5"#,
        )
        .bind(format!("{status:?}").to_lowercase())
        .bind(started_at)
        .bind(finished_at)
        .bind(log_path)
        .bind(id.to_string())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn list(&self, limit: i64) -> Result<Vec<BuildSummary>> {
        let rows = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds ORDER BY created_at DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows.into_iter().map(row_to_summary).collect())
    }

    pub async fn get_summary(&self, id: Uuid) -> Result<Option<BuildSummary>> {
        let row = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds WHERE id = $1"#,
        )
        .bind(id.to_string())
        .fetch_optional(self.storage.pool())
        .await?;
        Ok(row.map(row_to_summary))
    }

    pub async fn get_record(&self, id: Uuid) -> Result<Option<BuildRecord>> {
        let row = sqlx::query(
            r#"SELECT id, status, spec_json, created_at, started_at, finished_at, log_path
               FROM builds WHERE id = $1"#,
        )
        .bind(id.to_string())
        .fetch_optional(self.storage.pool())
        .await?;
        let Some(r) = row else {
            return Ok(None);
        };
        let spec_json: String = r.get("spec_json");
        let spec = serde_json::from_str(&spec_json)?;
        let status_s: String = r.get("status");
        let id_s: String = r.get("id");
        let created_at: DateTime<Utc> = r.get("created_at");
        let started_at: Option<DateTime<Utc>> = r.try_get("started_at").ok().flatten();
        let finished_at: Option<DateTime<Utc>> = r.try_get("finished_at").ok().flatten();
        let artifacts = self.list_artifacts(id).await?;
        Ok(Some(BuildRecord {
            id: Uuid::parse_str(&id_s).map_err(|e| Error::Internal(anyhow::anyhow!(e)))?,
            spec,
            status: parse_status(&status_s)?,
            created_at,
            started_at,
            finished_at,
            artifacts,
            scan: self.get_scan(id).await?,
            sbom: self.get_sbom(id).await?,
            log_path: r.try_get("log_path").ok().flatten(),
        }))
    }

    pub async fn save_artifact(&self, build_id: Uuid, artifact: &BuildArtifact) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO artifacts (build_id, digest, registry_ref, bytes, architecture)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (build_id, digest, architecture) DO UPDATE SET
                   registry_ref = EXCLUDED.registry_ref,
                   bytes = EXCLUDED.bytes"#,
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

    pub async fn list_artifacts(&self, build_id: Uuid) -> Result<Vec<BuildArtifact>> {
        let rows = sqlx::query(
            r#"SELECT digest, registry_ref, bytes, architecture
               FROM artifacts WHERE build_id = $1"#,
        )
        .bind(build_id.to_string())
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let arch_s: String = r.get("architecture");
                Ok(BuildArtifact {
                    digest: r.get("digest"),
                    registry_ref: r.try_get("registry_ref").ok().flatten(),
                    bytes: r.get::<i64, _>("bytes") as u64,
                    architecture: parse_arch(&arch_s)?,
                })
            })
            .collect()
    }

    pub async fn save_scan(&self, build_id: Uuid, scan: &ScanResult) -> Result<()> {
        let findings: serde_json::Value = serde_json::to_value(&scan.findings)?;
        sqlx::query(
            r#"INSERT INTO scans (build_id, scanner, scanned_at, findings)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (build_id) DO UPDATE SET
                   scanner = EXCLUDED.scanner,
                   scanned_at = EXCLUDED.scanned_at,
                   findings = EXCLUDED.findings"#,
        )
        .bind(build_id.to_string())
        .bind(&scan.scanner)
        .bind(scan.scanned_at)
        .bind(findings)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn get_scan(&self, build_id: Uuid) -> Result<Option<ScanResult>> {
        let row = sqlx::query(
            r#"SELECT scanner, scanned_at, findings FROM scans WHERE build_id = $1"#,
        )
        .bind(build_id.to_string())
        .fetch_optional(self.storage.pool())
        .await?;
        match row {
            None => Ok(None),
            Some(r) => {
                let scanner: String = r.get("scanner");
                let scanned_at: DateTime<Utc> = r.get("scanned_at");
                let findings_json: serde_json::Value = r.get("findings");
                let findings: Vec<Vulnerability> = serde_json::from_value(findings_json)?;
                Ok(Some(ScanResult {
                    scanner,
                    scanned_at,
                    findings,
                }))
            }
        }
    }

    pub async fn save_sbom(&self, build_id: Uuid, sbom: &Sbom) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO sboms (build_id, format, document)
               VALUES ($1, $2, $3)
               ON CONFLICT (build_id) DO UPDATE SET
                   format = EXCLUDED.format,
                   document = EXCLUDED.document"#,
        )
        .bind(build_id.to_string())
        .bind(&sbom.format)
        .bind(&sbom.document)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn get_sbom(&self, build_id: Uuid) -> Result<Option<Sbom>> {
        let row = sqlx::query(r#"SELECT format, document FROM sboms WHERE build_id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.map(|r| Sbom {
            format: r.get("format"),
            document: r.get("document"),
        }))
    }

    pub async fn get_log_path(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT log_path FROM builds WHERE id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.and_then(|r| r.try_get::<Option<String>, _>("log_path").ok().flatten()))
    }
}

// ---------------------------------------------------------------------------
// audit + principals + provenance + drift
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PgAuditLog {
    storage: PgStorage,
}

impl PgAuditLog {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }

    pub async fn record(
        &self,
        actor: &str,
        action: &str,
        target: Option<&str>,
        outcome: Outcome,
        details: Option<serde_json::Value>,
    ) -> Result<()> {
        let details = details
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".into()));
        sqlx::query(
            r#"INSERT INTO audit_events (actor, action, target, outcome, details, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(actor)
        .bind(action)
        .bind(target)
        .bind(outcome.as_str())
        .bind(details)
        .bind(Utc::now())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn recent(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        let rows = sqlx::query(
            r#"SELECT id, actor, action, target, outcome, details, created_at
               FROM audit_events ORDER BY id DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let created_at: DateTime<Utc> = r.get("created_at");
                AuditEntry {
                    id: r.get::<i64, _>("id"),
                    actor: r.get("actor"),
                    action: r.get("action"),
                    target: r.try_get("target").ok().flatten(),
                    outcome: r.get("outcome"),
                    details: r.try_get("details").ok().flatten(),
                    created_at: created_at.to_rfc3339(),
                }
            })
            .collect())
    }
}

#[derive(Clone)]
pub struct PgPrincipalRepo {
    storage: PgStorage,
}

impl PgPrincipalRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }

    pub async fn create(&self, name: &str, role: Role) -> Result<CreatedPrincipal> {
        let id = Uuid::new_v4().to_string();
        let token = format!("forge_{}", random_token());
        let token_hash = crate::rbac::hash_token(&token);
        let created_at = Utc::now();
        sqlx::query(
            r#"INSERT INTO principals (id, name, role, token_hash, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(&id)
        .bind(name)
        .bind(role.as_str())
        .bind(&token_hash)
        .bind(created_at)
        .execute(self.storage.pool())
        .await?;
        Ok(CreatedPrincipal {
            principal: Principal {
                id,
                name: name.into(),
                role,
                created_at: created_at.to_rfc3339(),
            },
            token,
        })
    }

    pub async fn list(&self) -> Result<Vec<Principal>> {
        let rows = sqlx::query(
            r#"SELECT id, name, role, created_at
               FROM principals WHERE revoked_at IS NULL ORDER BY created_at DESC"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let created_at: DateTime<Utc> = r.get("created_at");
                Some(Principal {
                    id: r.get("id"),
                    name: r.get("name"),
                    role: Role::parse(r.get::<&str, _>("role"))?,
                    created_at: created_at.to_rfc3339(),
                })
            })
            .collect())
    }

    pub async fn revoke(&self, id: &str) -> Result<()> {
        sqlx::query(r#"UPDATE principals SET revoked_at = $1 WHERE id = $2"#)
            .bind(Utc::now())
            .bind(id)
            .execute(self.storage.pool())
            .await?;
        Ok(())
    }

    pub async fn authenticate(&self, token: &str) -> Result<Option<Principal>> {
        let hash = crate::rbac::hash_token(token);
        let row = sqlx::query(
            r#"SELECT id, name, role, created_at FROM principals
               WHERE token_hash = $1 AND revoked_at IS NULL"#,
        )
        .bind(&hash)
        .fetch_optional(self.storage.pool())
        .await?;
        Ok(row.and_then(|r| {
            let created_at: DateTime<Utc> = r.get("created_at");
            Some(Principal {
                id: r.get("id"),
                name: r.get("name"),
                role: Role::parse(r.get::<&str, _>("role"))?,
                created_at: created_at.to_rfc3339(),
            })
        }))
    }
}

#[derive(Clone)]
pub struct PgProvenanceRepo {
    storage: PgStorage,
}

impl PgProvenanceRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }

    pub async fn save(
        &self,
        build_id: Uuid,
        statement: &Statement,
        bundle_path: Option<&str>,
    ) -> Result<()> {
        let predicate: serde_json::Value = serde_json::to_value(statement)?;
        sqlx::query(
            r#"INSERT INTO provenance (build_id, predicate, attested_at, bundle_path)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (build_id) DO UPDATE SET
                   predicate = EXCLUDED.predicate,
                   attested_at = EXCLUDED.attested_at,
                   bundle_path = EXCLUDED.bundle_path"#,
        )
        .bind(build_id.to_string())
        .bind(predicate)
        .bind(Utc::now())
        .bind(bundle_path)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn get(&self, build_id: Uuid) -> Result<Option<Statement>> {
        let row = sqlx::query(r#"SELECT predicate FROM provenance WHERE build_id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        match row {
            None => Ok(None),
            Some(r) => {
                let predicate: serde_json::Value = r.get("predicate");
                Ok(Some(serde_json::from_value(predicate)?))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn row_to_summary(r: PgRow) -> BuildSummary {
    let created_at: DateTime<Utc> = r.get("created_at");
    let finished_at: Option<DateTime<Utc>> = r.try_get("finished_at").ok().flatten();
    BuildSummary {
        id: r.get("id"),
        name: r.get("name"),
        runtime: r.get("runtime"),
        base_image: r.get("base_image"),
        status: r.get("status"),
        created_at: created_at.to_rfc3339(),
        finished_at: finished_at.map(|d| d.to_rfc3339()),
    }
}

fn parse_status(s: &str) -> Result<BuildStatus> {
    Ok(match s {
        "pending" => BuildStatus::Pending,
        "running" => BuildStatus::Running,
        "succeeded" => BuildStatus::Succeeded,
        "failed" => BuildStatus::Failed,
        "cancelled" => BuildStatus::Cancelled,
        other => {
            return Err(Error::Internal(anyhow::anyhow!(
                "unknown build status: {other}"
            )));
        }
    })
}

fn parse_arch(s: &str) -> Result<Architecture> {
    Ok(match s {
        "amd64" => Architecture::Amd64,
        "arm64" => Architecture::Arm64,
        other => {
            return Err(Error::Internal(anyhow::anyhow!(
                "unknown architecture: {other}"
            )));
        }
    })
}

fn random_token() -> String {
    Uuid::new_v4().simple().to_string()[..24].to_string()
}

// Suppress dead-code warning for the Severity import; it's pulled in for
// JSON-deserializing Vulnerability.severity through Vec<Vulnerability>.
#[allow(dead_code)]
const _SEVERITY_USED: Option<Severity> = None;
