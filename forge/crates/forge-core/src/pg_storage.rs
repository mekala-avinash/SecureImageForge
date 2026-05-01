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

use crate::audit::{AuditEntry, AuditLog, Outcome};
use crate::domain::{
    Architecture, BuildArtifact, BuildRecord, BuildStatus, Sbom, ScanResult, Severity,
    Vulnerability,
};
use crate::drift::{DriftDetector, DriftSnapshot};
use crate::logs::LogStore;
use std::path::PathBuf;
use std::sync::Arc;
use crate::provenance::{ProvenanceRepo, Statement};
use crate::rbac::{CreatedPrincipal, Principal, PrincipalRepo, Role};
use crate::repo::{BuildRepo, BuildSummary};
use crate::team::{
    BuildJob, BuildQueueRepo, GroupRoleBinding, ScopeGrant, ScopeRepo, TeamRepo,
};
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
}

#[async_trait::async_trait]
impl BuildRepo for PgBuildRepo {
    async fn insert(&self, record: &BuildRecord) -> Result<()> {
        self.insert_for_project(record, "default-project").await
    }

    async fn insert_for_project(&self, record: &BuildRecord, project_id: &str) -> Result<()> {
        let spec_json = serde_json::to_string(&record.spec)?;
        sqlx::query(
            r#"INSERT INTO builds (id, name, runtime, base_image, status, spec_json,
                                   created_at, started_at, finished_at, log_path, project_id)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
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

    async fn list(&self, limit: i64) -> Result<Vec<BuildSummary>> {
        self.list_project("default-project", limit).await
    }

    async fn list_project(&self, project_id: &str, limit: i64) -> Result<Vec<BuildSummary>> {
        let rows = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds WHERE project_id = $1 ORDER BY created_at DESC LIMIT $2"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows.into_iter().map(row_to_summary).collect())
    }

    async fn get_summary(&self, id: Uuid) -> Result<Option<BuildSummary>> {
        self.get_summary_in_project("default-project", id).await
    }

    async fn get_summary_in_project(&self, project_id: &str, id: Uuid) -> Result<Option<BuildSummary>> {
        let row = sqlx::query(
            r#"SELECT id, name, runtime, base_image, status, created_at, finished_at
               FROM builds WHERE id = $1 AND project_id = $2"#,
        )
        .bind(id.to_string())
        .bind(project_id)
        .fetch_optional(self.storage.pool())
        .await?;
        Ok(row.map(row_to_summary))
    }

    async fn get_record(&self, id: Uuid) -> Result<Option<BuildRecord>> {
        self.get_record_in_project("default-project", id).await
    }

    async fn get_record_in_project(&self, project_id: &str, id: Uuid) -> Result<Option<BuildRecord>> {
        let row = sqlx::query(
            r#"SELECT id, status, spec_json, created_at, started_at, finished_at, log_path
               FROM builds WHERE id = $1 AND project_id = $2"#,
        )
        .bind(id.to_string())
        .bind(project_id)
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

    async fn save_artifact(&self, build_id: Uuid, artifact: &BuildArtifact) -> Result<()> {
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

    async fn list_artifacts(&self, build_id: Uuid) -> Result<Vec<BuildArtifact>> {
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

    async fn save_scan(&self, build_id: Uuid, scan: &ScanResult) -> Result<()> {
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

    async fn get_scan(&self, build_id: Uuid) -> Result<Option<ScanResult>> {
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

    async fn save_sbom(&self, build_id: Uuid, sbom: &Sbom) -> Result<()> {
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

    async fn get_sbom(&self, build_id: Uuid) -> Result<Option<Sbom>> {
        let row = sqlx::query(r#"SELECT format, document FROM sboms WHERE build_id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.map(|r| Sbom {
            format: r.get("format"),
            document: r.get("document"),
        }))
    }

    async fn get_log_path(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT log_path FROM builds WHERE id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.and_then(|r| r.try_get::<Option<String>, _>("log_path").ok().flatten()))
    }

    async fn get_project_id(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT project_id FROM builds WHERE id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.map(|r| r.get("project_id")))
    }

    async fn drift_targets(&self, limit: i64) -> Result<Vec<(Uuid, String)>> {
        let rows = sqlx::query(
            r#"SELECT b.id, COALESCE(a.registry_ref, a.digest) AS image_ref
               FROM builds b
               JOIN artifacts a ON a.build_id = b.id
               WHERE b.status = 'succeeded'
               ORDER BY b.finished_at DESC LIMIT $1"#,
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
}

// ---------------------------------------------------------------------------
// logs
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PgLogStore {
    storage: PgStorage,
}

impl PgLogStore {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl LogStore for PgLogStore {
    async fn write(&self, build_id: Uuid, content: &str) -> Result<PathBuf> {
        sqlx::query(
            r#"INSERT INTO build_logs (build_id, content)
               VALUES ($1, $2)
               ON CONFLICT (build_id) DO UPDATE SET content = EXCLUDED.content"#,
        )
        .bind(build_id.to_string())
        .bind(content)
        .execute(self.storage.pool())
        .await?;
        Ok(PathBuf::from(format!("pg://{build_id}")))
    }

    async fn read(&self, build_id: Uuid) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT content FROM build_logs WHERE build_id = $1"#)
            .bind(build_id.to_string())
            .fetch_optional(self.storage.pool())
            .await?;
        Ok(row.map(|r| r.get("content")))
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
}

#[async_trait::async_trait]
impl AuditLog for PgAuditLog {
    async fn record(
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

    async fn recent(&self, limit: i64) -> Result<Vec<AuditEntry>> {
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
}

#[async_trait::async_trait]
impl PrincipalRepo for PgPrincipalRepo {
    async fn create(&self, name: &str, role: Role) -> Result<CreatedPrincipal> {
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

    async fn list(&self) -> Result<Vec<Principal>> {
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

    async fn revoke(&self, id: &str) -> Result<()> {
        sqlx::query(r#"UPDATE principals SET revoked_at = $1 WHERE id = $2"#)
            .bind(Utc::now())
            .bind(id)
            .execute(self.storage.pool())
            .await?;
        Ok(())
    }

    async fn authenticate(&self, token: &str) -> Result<Option<Principal>> {
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
}

#[async_trait::async_trait]
impl ProvenanceRepo for PgProvenanceRepo {
    async fn save(
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

    async fn get(&self, build_id: Uuid) -> Result<Option<Statement>> {
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

pub struct PgTeamRepo {
    storage: PgStorage,
}

impl PgTeamRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl TeamRepo for PgTeamRepo {
    async fn create_org(&self, id: &str, name: &str) -> Result<()> {
        sqlx::query(r#"INSERT INTO organizations (id, name, created_at) VALUES ($1, $2, $3)"#)
            .bind(id)
            .bind(name)
            .bind(Utc::now())
            .execute(self.storage.pool())
            .await?;
        Ok(())
    }

    async fn create_project(&self, id: &str, org_id: &str, name: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO projects (id, organization_id, name, created_at) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(Utc::now())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn create_environment(&self, id: &str, project_id: &str, name: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO environments (id, project_id, name, created_at) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(project_id)
        .bind(name)
        .bind(Utc::now())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }
}

pub struct PgScopeRepo {
    storage: PgStorage,
}

impl PgScopeRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl ScopeRepo for PgScopeRepo {
    async fn bind_group_role(&self, group_name: &str, role: Role) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO group_role_bindings (group_name, role, created_at)
               VALUES ($1, $2, $3)
               ON CONFLICT(group_name) DO UPDATE SET role = EXCLUDED.role"#,
        )
        .bind(group_name)
        .bind(role.as_str())
        .bind(Utc::now())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn list_group_bindings(&self) -> Result<Vec<GroupRoleBinding>> {
        let rows = sqlx::query(
            r#"SELECT group_name, role, created_at FROM group_role_bindings ORDER BY group_name"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let role: String = r.get("role");
                let created_at: chrono::DateTime<Utc> = r.get("created_at");
                Ok(GroupRoleBinding {
                    group_name: r.get("group_name"),
                    role: Role::parse(&role)
                        .ok_or_else(|| Error::Internal(anyhow::anyhow!("unknown role: {role}")))?,
                    created_at: created_at.to_rfc3339(),
                })
            })
            .collect()
    }

    async fn create_scope_grant(
        &self,
        principal_id: &str,
        scope_type: &str,
        scope_id: &str,
        role: Role,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO principal_scopes (principal_id, scope_type, scope_id, role, created_at)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT(principal_id, scope_type, scope_id)
               DO UPDATE SET role = EXCLUDED.role"#,
        )
        .bind(principal_id)
        .bind(scope_type)
        .bind(scope_id)
        .bind(role.as_str())
        .bind(Utc::now())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn list_scope_grants(&self) -> Result<Vec<ScopeGrant>> {
        let rows = sqlx::query(
            r#"SELECT principal_id, scope_type, scope_id, role, created_at
               FROM principal_scopes ORDER BY principal_id, scope_type, scope_id"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let role: String = r.get("role");
                let created_at: chrono::DateTime<Utc> = r.get("created_at");
                Ok(ScopeGrant {
                    principal_id: r.get("principal_id"),
                    scope_type: r.get("scope_type"),
                    scope_id: r.get("scope_id"),
                    role: Role::parse(&role)
                        .ok_or_else(|| Error::Internal(anyhow::anyhow!("unknown role: {role}")))?,
                    created_at: created_at.to_rfc3339(),
                })
            })
            .collect()
    }

    async fn has_scope_role(
        &self,
        principal_id: &str,
        scope_type: &str,
        scope_id: &str,
        min_role: Role,
    ) -> Result<bool> {
        let row = sqlx::query(
            r#"SELECT role FROM principal_scopes
               WHERE principal_id = $1 AND scope_type = $2 AND scope_id = $3"#,
        )
        .bind(principal_id)
        .bind(scope_type)
        .bind(scope_id)
        .fetch_optional(self.storage.pool())
        .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let role: String = row.get("role");
        let role = Role::parse(&role)
            .ok_or_else(|| Error::Internal(anyhow::anyhow!("unknown role: {role}")))?;
        Ok(role.rank() >= min_role.rank())
    }
}

pub struct PgBuildQueueRepo {
    storage: PgStorage,
}

impl PgBuildQueueRepo {
    pub fn new(storage: PgStorage) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl BuildQueueRepo for PgBuildQueueRepo {
    async fn enqueue(
        &self,
        build_id: Uuid,
        project_id: &str,
        max_retries: u32,
    ) -> Result<BuildJob> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO build_jobs (
                    id, build_id, project_id, status, attempts, max_retries,
                    leased_until, worker_id, next_attempt_at, last_error,
                    created_at, updated_at
               ) VALUES ($1, $2, $3, 'queued', 0, $4, NULL, NULL, $5, NULL, $6, $7)"#,
        )
        .bind(&id)
        .bind(build_id.to_string())
        .bind(project_id)
        .bind(max_retries as i64)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(self.storage.pool())
        .await?;
        self.get_by_id(&id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("job {id}")))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<BuildJob>> {
        let row = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(self.storage.pool())
        .await?;
        row.map(pg_row_to_job).transpose()
    }

    async fn list_project(&self, project_id: &str, limit: i64) -> Result<Vec<BuildJob>> {
        let rows = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs
               WHERE project_id = $1
               ORDER BY created_at DESC
               LIMIT $2"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter().map(pg_row_to_job).collect()
    }

    async fn cancel_by_build(
        &self,
        project_id: &str,
        build_id: Uuid,
    ) -> Result<Option<BuildJob>> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE build_jobs
               SET status = 'canceled', updated_at = $1, worker_id = NULL, leased_until = NULL
               WHERE project_id = $2 AND build_id = $3 AND status IN ('queued','leased','running')"#,
        )
        .bind(now)
        .bind(project_id)
        .bind(build_id.to_string())
        .execute(self.storage.pool())
        .await?;
        let row = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs
               WHERE project_id = $1 AND build_id = $2
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(project_id)
        .bind(build_id.to_string())
        .fetch_optional(self.storage.pool())
        .await?;
        row.map(pg_row_to_job).transpose()
    }

    async fn lease_next(
        &self,
        worker_id: &str,
        lease_seconds: u64,
    ) -> Result<Option<BuildJob>> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"SELECT id FROM build_jobs
               WHERE status = 'queued' AND next_attempt_at <= $1
               ORDER BY created_at ASC
               LIMIT 1"#,
        )
        .bind(now)
        .fetch_optional(self.storage.pool())
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let id: String = row.get("id");
        let leased_until = now + chrono::Duration::seconds(lease_seconds as i64);
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='leased', worker_id=$1, leased_until=$2, updated_at=$3
               WHERE id = $4 AND status = 'queued'"#,
        )
        .bind(worker_id)
        .bind(leased_until)
        .bind(now)
        .bind(&id)
        .execute(self.storage.pool())
        .await?;
        self.get_by_id(&id).await
    }

    async fn mark_running(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='running', attempts=attempts+1, updated_at=$1
               WHERE id = $2"#,
        )
        .bind(Utc::now())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn mark_success(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='succeeded', leased_until=NULL, worker_id=NULL, updated_at=$1
               WHERE id = $2"#,
        )
        .bind(Utc::now())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    async fn mark_failure_retry_or_deadletter(
        &self,
        job_id: &str,
        error: &str,
        backoff_seconds: u64,
    ) -> Result<String> {
        let job = self
            .get_by_id(job_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("job {job_id}")))?;
        let status = if job.attempts >= job.max_retries {
            "deadletter"
        } else {
            "queued"
        };
        let next_attempt = Utc::now() + chrono::Duration::seconds(backoff_seconds as i64);
        sqlx::query(
            r#"UPDATE build_jobs
               SET status=$1, last_error=$2, next_attempt_at=$3, leased_until=NULL, worker_id=NULL, updated_at=$4
               WHERE id = $5"#,
        )
        .bind(status)
        .bind(error)
        .bind(next_attempt)
        .bind(Utc::now())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(status.into())
    }
}

fn pg_row_to_job(r: sqlx::postgres::PgRow) -> Result<BuildJob> {
    let build_id: String = r.get("build_id");
    let created_at: chrono::DateTime<Utc> = r.get("created_at");
    let updated_at: chrono::DateTime<Utc> = r.get("updated_at");
    let next_attempt_at: chrono::DateTime<Utc> = r.get("next_attempt_at");
    let leased_until: Option<chrono::DateTime<Utc>> = r.get("leased_until");
    
    Ok(BuildJob {
        id: r.get("id"),
        build_id,
        project_id: r.get("project_id"),
        status: r.get("status"),
        attempts: r.get::<i32, _>("attempts") as i64,
        max_retries: r.get::<i32, _>("max_retries") as i64,
        leased_until: leased_until.map(|d| d.to_rfc3339()),
        worker_id: r.get("worker_id"),
        next_attempt_at: next_attempt_at.to_rfc3339(),
        last_error: r.get("last_error"),
        created_at: created_at.to_rfc3339(),
        updated_at: updated_at.to_rfc3339(),
    })
}

pub struct PgDriftDetector {
    pub repo: Arc<dyn BuildRepo>,
    pub storage: PgStorage,
    pub scanner: Arc<dyn crate::tooling::Scanner>,
    pub audit: Arc<dyn crate::audit::AuditLog>,
}

impl PgDriftDetector {
    async fn persist(&self, snap: &DriftSnapshot, scan: &crate::domain::ScanResult) -> Result<()> {
        let findings = serde_json::to_string(&scan.findings)?;
        sqlx::query(
            r#"INSERT INTO drift_snapshots (build_id, scanned_at, scanner, findings, new_critical, new_high)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(&snap.build_id)
        .bind(&snap.scanned_at)
        .bind(&snap.scanner)
        .bind(findings)
        .bind(snap.new_critical)
        .bind(snap.new_high)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DriftDetector for PgDriftDetector {
    async fn rescan_one(&self, build_id: Uuid, image_ref: &str) -> Result<DriftSnapshot> {
        let baseline = self
            .repo
            .get_scan(build_id)
            .await?
            .map(|r| crate::drift::ids_by_severity(&r))
            .unwrap_or_default();
        let now = self.scanner.scan(image_ref).await?;
        let snapshot = crate::drift::compute_snapshot(build_id, &baseline, &now);
        self.persist(&snapshot, &now).await?;
        let _ = self
            .audit
            .record(
                "drift-scheduler",
                "drift.rescan",
                Some(&build_id.to_string()),
                crate::audit::Outcome::Success,
                Some(serde_json::json!({
                    "new_critical": snapshot.new_critical,
                    "new_high": snapshot.new_high,
                })),
            )
            .await;
        Ok(snapshot)
    }

    async fn list(&self, build_id: Uuid, limit: i64) -> Result<Vec<DriftSnapshot>> {
        let rows = sqlx::query(
            r#"SELECT id, build_id, scanner, scanned_at, findings, new_critical, new_high
               FROM drift_snapshots WHERE build_id = $1 ORDER BY id DESC LIMIT $2"#,
        )
        .bind(build_id.to_string())
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let findings_str: String = r.get("findings");
                let findings: Vec<crate::domain::Vulnerability> =
                    serde_json::from_str(&findings_str).unwrap_or_default();
                DriftSnapshot {
                    id: r.get::<i64, _>("id"),
                    build_id: r.get("build_id"),
                    scanner: r.get("scanner"),
                    scanned_at: r.get("scanned_at"),
                    new_critical: r.get::<i64, _>("new_critical"),
                    new_high: r.get::<i64, _>("new_high"),
                    findings_count: findings.len(),
                }
            })
            .collect())
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
