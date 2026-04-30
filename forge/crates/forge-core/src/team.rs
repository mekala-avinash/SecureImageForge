use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

use crate::rbac::Role;
use crate::storage::Storage;
use crate::{Error, Result};

#[derive(Clone)]
pub struct TeamRepo {
    storage: Storage,
}

impl TeamRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn create_org(&self, id: &str, name: &str) -> Result<()> {
        sqlx::query(r#"INSERT INTO organizations (id, name, created_at) VALUES (?, ?, ?)"#)
            .bind(id)
            .bind(name)
            .bind(Utc::now().to_rfc3339())
            .execute(self.storage.pool())
            .await?;
        Ok(())
    }

    pub async fn create_project(&self, id: &str, org_id: &str, name: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO projects (id, organization_id, name, created_at) VALUES (?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(Utc::now().to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn create_environment(&self, id: &str, project_id: &str, name: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO environments (id, project_id, name, created_at) VALUES (?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(project_id)
        .bind(name)
        .bind(Utc::now().to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ScopeRepo {
    storage: Storage,
}

impl ScopeRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn bind_group_role(&self, group_name: &str, role: Role) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO group_role_bindings (group_name, role, created_at)
               VALUES (?, ?, ?)
               ON CONFLICT(group_name) DO UPDATE SET role = excluded.role"#,
        )
        .bind(group_name)
        .bind(role.as_str())
        .bind(Utc::now().to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn list_group_bindings(&self) -> Result<Vec<GroupRoleBinding>> {
        let rows = sqlx::query(
            r#"SELECT group_name, role, created_at FROM group_role_bindings ORDER BY group_name"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let role: String = r.get("role");
                Ok(GroupRoleBinding {
                    group_name: r.get("group_name"),
                    role: Role::parse(&role)
                        .ok_or_else(|| Error::Internal(anyhow::anyhow!("unknown role: {role}")))?,
                    created_at: r.get("created_at"),
                })
            })
            .collect()
    }

    pub async fn create_scope_grant(
        &self,
        principal_id: &str,
        scope_type: &str,
        scope_id: &str,
        role: Role,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO principal_scopes (principal_id, scope_type, scope_id, role, created_at)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(principal_id, scope_type, scope_id)
               DO UPDATE SET role = excluded.role"#,
        )
        .bind(principal_id)
        .bind(scope_type)
        .bind(scope_id)
        .bind(role.as_str())
        .bind(Utc::now().to_rfc3339())
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn list_scope_grants(&self) -> Result<Vec<ScopeGrant>> {
        let rows = sqlx::query(
            r#"SELECT principal_id, scope_type, scope_id, role, created_at
               FROM principal_scopes ORDER BY principal_id, scope_type, scope_id"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter()
            .map(|r| {
                let role: String = r.get("role");
                Ok(ScopeGrant {
                    principal_id: r.get("principal_id"),
                    scope_type: r.get("scope_type"),
                    scope_id: r.get("scope_id"),
                    role: Role::parse(&role)
                        .ok_or_else(|| Error::Internal(anyhow::anyhow!("unknown role: {role}")))?,
                    created_at: r.get("created_at"),
                })
            })
            .collect()
    }

    pub async fn has_scope_role(
        &self,
        principal_id: &str,
        scope_type: &str,
        scope_id: &str,
        min_role: Role,
    ) -> Result<bool> {
        let row = sqlx::query(
            r#"SELECT role FROM principal_scopes
               WHERE principal_id = ? AND scope_type = ? AND scope_id = ?"#,
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

#[derive(Debug, Clone, Serialize)]
pub struct GroupRoleBinding {
    pub group_name: String,
    pub role: Role,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScopeGrant {
    pub principal_id: String,
    pub scope_type: String,
    pub scope_id: String,
    pub role: Role,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildJob {
    pub id: String,
    pub build_id: String,
    pub project_id: String,
    pub status: String,
    pub attempts: i64,
    pub max_retries: i64,
    pub leased_until: Option<String>,
    pub worker_id: Option<String>,
    pub next_attempt_at: String,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct BuildQueueRepo {
    storage: Storage,
}

impl BuildQueueRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn enqueue(
        &self,
        build_id: Uuid,
        project_id: &str,
        max_retries: u32,
    ) -> Result<BuildJob> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO build_jobs (
                    id, build_id, project_id, status, attempts, max_retries,
                    leased_until, worker_id, next_attempt_at, last_error,
                    created_at, updated_at
               ) VALUES (?, ?, ?, 'queued', 0, ?, NULL, NULL, ?, NULL, ?, ?)"#,
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

    pub async fn get_by_id(&self, id: &str) -> Result<Option<BuildJob>> {
        let row = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(self.storage.pool())
        .await?;
        row.map(row_to_job).transpose()
    }

    pub async fn list_project(&self, project_id: &str, limit: i64) -> Result<Vec<BuildJob>> {
        let rows = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs
               WHERE project_id = ?
               ORDER BY created_at DESC
               LIMIT ?"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        rows.into_iter().map(row_to_job).collect()
    }

    pub async fn cancel_by_build(
        &self,
        project_id: &str,
        build_id: Uuid,
    ) -> Result<Option<BuildJob>> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"UPDATE build_jobs
               SET status = 'canceled', updated_at = ?, worker_id = NULL, leased_until = NULL
               WHERE project_id = ? AND build_id = ? AND status IN ('queued','leased','running')"#,
        )
        .bind(&now)
        .bind(project_id)
        .bind(build_id.to_string())
        .execute(self.storage.pool())
        .await?;
        let row = sqlx::query(
            r#"SELECT id, build_id, project_id, status, attempts, max_retries, leased_until,
                      worker_id, next_attempt_at, last_error, created_at, updated_at
               FROM build_jobs
               WHERE project_id = ? AND build_id = ?
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(project_id)
        .bind(build_id.to_string())
        .fetch_optional(self.storage.pool())
        .await?;
        row.map(row_to_job).transpose()
    }

    pub async fn lease_next(
        &self,
        worker_id: &str,
        lease_seconds: u64,
    ) -> Result<Option<BuildJob>> {
        let row = sqlx::query(
            r#"SELECT id FROM build_jobs
               WHERE status = 'queued' AND next_attempt_at <= ?
               ORDER BY created_at ASC
               LIMIT 1"#,
        )
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(self.storage.pool())
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let id: String = row.get("id");
        let now = Utc::now();
        let leased_until = (now + Duration::seconds(lease_seconds as i64)).to_rfc3339();
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='leased', worker_id=?, leased_until=?, updated_at=?
               WHERE id = ? AND status = 'queued'"#,
        )
        .bind(worker_id)
        .bind(&leased_until)
        .bind(now.to_rfc3339())
        .bind(&id)
        .execute(self.storage.pool())
        .await?;
        self.get_by_id(&id).await
    }

    pub async fn mark_running(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='running', attempts=attempts+1, updated_at=?
               WHERE id = ?"#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn mark_success(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE build_jobs
               SET status='succeeded', leased_until=NULL, worker_id=NULL, updated_at=?
               WHERE id = ?"#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(())
    }

    pub async fn mark_failure_retry_or_deadletter(
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
        let next_attempt = (Utc::now() + Duration::seconds(backoff_seconds as i64)).to_rfc3339();
        sqlx::query(
            r#"UPDATE build_jobs
               SET status=?, last_error=?, next_attempt_at=?, leased_until=NULL, worker_id=NULL, updated_at=?
               WHERE id = ?"#,
        )
        .bind(status)
        .bind(error)
        .bind(next_attempt)
        .bind(Utc::now().to_rfc3339())
        .bind(job_id)
        .execute(self.storage.pool())
        .await?;
        Ok(status.into())
    }
}

fn row_to_job(r: sqlx::sqlite::SqliteRow) -> Result<BuildJob> {
    Ok(BuildJob {
        id: r.get("id"),
        build_id: r.get("build_id"),
        project_id: r.get("project_id"),
        status: r.get("status"),
        attempts: r.get("attempts"),
        max_retries: r.get("max_retries"),
        leased_until: r.get("leased_until"),
        worker_id: r.get("worker_id"),
        next_attempt_at: r.get("next_attempt_at"),
        last_error: r.get("last_error"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    })
}
