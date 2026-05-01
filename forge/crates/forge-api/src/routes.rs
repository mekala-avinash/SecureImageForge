use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use forge_core::audit::{AuditLog, Outcome};
use forge_core::config::AuthMode;
use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, BuildStatus, ComplianceProfile, HardeningOptions, Runtime,
};
use forge_core::rbac::{Action, CreatedPrincipal, Principal, Role};

use crate::auth::{require, Authenticated};
use crate::error::ApiError;
use crate::state::ApiState;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Prometheus scrape endpoint. Exposes the global recorder installed in
/// `serve()`; safe to expose without auth for the typical scraper deployment
/// where the daemon listens on localhost or a private metrics network.
pub async fn metrics() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        crate::metrics::render(),
    )
}

/// Stream the persisted build log as Server-Sent Events. The endpoint
/// re-reads from disk every second so a UI can subscribe early in a build
/// and watch lines appear; events stop once the file no longer grows for
/// `idle_max` ticks.
pub async fn stream_log(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Sse<impl futures::Stream<Item = Result<SseEvent, std::convert::Infallible>>>, ApiError>
{
    let project_id = s.default_project_id().to_string();
    stream_log_for_project(s, principal, &project_id, &id).await
}

pub async fn stream_log_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Sse<impl futures::Stream<Item = Result<SseEvent, std::convert::Infallible>>>, ApiError>
{
    stream_log_for_project(s, principal, &project_id, &id).await
}

async fn stream_log_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Sse<impl futures::Stream<Item = Result<SseEvent, std::convert::Infallible>>>, ApiError>
{
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let logs = s.logs.clone();

    use async_stream::stream;
    let stream = stream! {
        let mut last_len = 0usize;
        let mut idle_ticks = 0u32;
        loop {
            let content = logs.read(id).await.ok().flatten().unwrap_or_default();
            if content.len() > last_len {
                let chunk = content[last_len..].to_string();
                last_len = content.len();
                idle_ticks = 0;
                yield Ok(SseEvent::default().data(chunk));
            } else {
                idle_ticks += 1;
                if idle_ticks > 60 {
                    // 60 ticks * 1s = 1 minute of silence → assume the build
                    // is no longer producing output and close the stream.
                    yield Ok(SseEvent::default().event("eof").data(""));
                    break;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Deserialize)]
pub struct CreateBuildRequest {
    pub name: String,
    pub runtime: String,
    pub base: String,
    #[serde(default)]
    pub compliance: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
    #[serde(default)]
    pub no_sbom: bool,
    #[serde(default)]
    pub no_sign: bool,
}

pub async fn list_builds(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    list_builds_for_project(s, principal, &project_id).await
}

pub async fn create_build(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(req): Json<CreateBuildRequest>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    create_build_for_project(s, principal, &project_id, req).await
}

pub async fn list_builds_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    list_builds_for_project(s, principal, &project_id).await
}

pub async fn create_build_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(project_id): Path<String>,
    Json(req): Json<CreateBuildRequest>,
) -> Result<Json<Value>, ApiError> {
    create_build_for_project(s, principal, &project_id, req).await
}

async fn list_builds_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ListBuilds).await?;
    let rows = s
        .builds
        .list_project(project_id, 200)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

async fn create_build_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    req: CreateBuildRequest,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::StartBuild).await?;
    let spec = parse_spec(req)?;
    spec.validate().map_err(ApiError::from)?;

    // Persist the build immediately; /v1/builds/{id}/start dispatches it.
    let record = forge_core::domain::BuildRecord::new(spec);
    s.builds
        .insert_for_project(&record, project_id)
        .await
        .map_err(ApiError::from)?;
    record_audit(
        &s.audit,
        &principal,
        "build.create",
        Some(&record.id.to_string()),
        Outcome::Success,
    )
    .await;
    Ok(Json(
        json!({ "id": record.id.to_string(), "project_id": project_id }),
    ))
}

pub async fn start_build(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    start_build_for_project(s, principal, &project_id, &id).await
}

pub async fn start_build_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    start_build_for_project(s, principal, &project_id, &id).await
}

async fn start_build_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::StartBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    let record = s
        .builds
        .get_record_in_project(project_id, id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    if record.status != BuildStatus::Pending {
        return Err(ApiError::BadRequest("build is not pending".into()));
    }
    let job = s
        .queue
        .enqueue(id, project_id, s.config.workers.max_retries)
        .await
        .map_err(ApiError::from)?;

    record_audit_with_details(
        &s.audit,
        &principal.name,
        "build.start.requested",
        Some(&id.to_string()),
        Outcome::Success,
        Some(json!({"project_id": project_id, "job_id": job.id, "dispatch": "queued"})),
    )
    .await;
    Ok(Json(json!({
        "id": id.to_string(),
        "project_id": project_id,
        "status": "queued",
        "job_id": job.id
    })))
}

pub async fn get_build(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    get_build_for_project(s, principal, &project_id, &id).await
}

pub async fn get_build_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    get_build_for_project(s, principal, &project_id, &id).await
}

async fn get_build_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    let summary = s
        .builds
        .get_summary_in_project(project_id, id)
        .await
        .map_err(ApiError::from)?;
    summary.map(|v| Json(json!(v))).ok_or(ApiError::NotFound)
}

pub async fn get_scan(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    get_scan_for_project(s, principal, &project_id, &id).await
}

pub async fn get_scan_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    get_scan_for_project(s, principal, &project_id, &id).await
}

async fn get_scan_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let scan = s.builds.get_scan(id).await.map_err(ApiError::from)?;
    scan.map(|v| Json(json!(v))).ok_or(ApiError::NotFound)
}

pub async fn get_sbom(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    get_sbom_for_project(s, principal, &project_id, &id).await
}

pub async fn get_sbom_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    get_sbom_for_project(s, principal, &project_id, &id).await
}

async fn get_sbom_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let sbom = s.builds.get_sbom(id).await.map_err(ApiError::from)?;
    sbom.map(|b| Json(b.document)).ok_or(ApiError::NotFound)
}

pub async fn get_log(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<String, ApiError> {
    let project_id = s.default_project_id().to_string();
    get_log_for_project(s, principal, &project_id, &id).await
}

pub async fn get_log_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<String, ApiError> {
    get_log_for_project(s, principal, &project_id, &id).await
}

async fn get_log_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<String, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let log = s.logs.read(id).await.map_err(ApiError::from)?;
    log.ok_or(ApiError::NotFound)
}

pub async fn get_provenance(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    get_provenance_for_project(s, principal, &project_id, &id).await
}

pub async fn get_provenance_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    get_provenance_for_project(s, principal, &project_id, &id).await
}

async fn get_provenance_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let stmt = s.provenance.get(id).await.map_err(ApiError::from)?;
    stmt.map(|v| Json(json!(v))).ok_or(ApiError::NotFound)
}

pub async fn list_drift(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project_id = s.default_project_id().to_string();
    list_drift_for_project(s, principal, &project_id, &id).await
}

pub async fn list_drift_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    list_drift_for_project(s, principal, &project_id, &id).await
}

async fn list_drift_for_project(
    s: ApiState,
    principal: Principal,
    project_id: &str,
    id: &str,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, project_id, Action::ReadBuild).await?;
    let id = Uuid::parse_str(id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    ensure_build_in_project(&s, project_id, id).await?;
    let rows = s.drift.list(id, 100).await.map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

pub async fn cancel_build_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, &project_id, Action::StartBuild).await?;
    let id = Uuid::parse_str(&id).map_err(|_| ApiError::BadRequest("invalid build id".into()))?;
    let job = s
        .queue
        .cancel_by_build(&project_id, id)
        .await
        .map_err(ApiError::from)?;
    if let Some(job) = &job {
        s.builds
            .update_status(id, BuildStatus::Cancelled, None, Some(Utc::now()), None)
            .await
            .map_err(ApiError::from)?;
        record_audit_with_details(
            &s.audit,
            &principal.name,
            "build.cancel",
            Some(&id.to_string()),
            Outcome::Success,
            Some(json!({"project_id": project_id, "job_id": job.id})),
        )
        .await;
    }
    Ok(Json(json!({
        "build_id": id.to_string(),
        "project_id": project_id,
        "canceled": job.is_some(),
        "job": job
    })))
}

pub async fn list_jobs_in_project(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(project_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require_project_scope(&s, &principal, &project_id, Action::ReadBuild).await?;
    let rows = s
        .queue
        .list_project(&project_id, 200)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

pub async fn list_audit(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    // Audit reads require admin to avoid leaking who-did-what to viewers.
    require(&principal, Action::ManagePrincipals)?;
    let rows = s.audit.recent(200).await.map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

pub async fn list_principals(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let rows = s.principals.list().await.map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

#[derive(Deserialize)]
pub struct CreatePrincipalRequest {
    pub name: String,
    pub role: String,
}

pub async fn create_principal(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(req): Json<CreatePrincipalRequest>,
) -> Result<Json<CreatedPrincipal>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let role = Role::parse(&req.role)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown role: {}", req.role)))?;
    let created = s
        .principals
        .create(&req.name, role)
        .await
        .map_err(ApiError::from)?;
    record_audit(
        &s.audit,
        &principal,
        "principal.create",
        Some(&created.principal.id),
        Outcome::Success,
    )
    .await;
    Ok(Json(created))
}

pub async fn revoke_principal(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    s.principals.revoke(&id).await.map_err(ApiError::from)?;
    record_audit(
        &s.audit,
        &principal,
        "principal.revoke",
        Some(&id),
        Outcome::Success,
    )
    .await;
    Ok(Json(json!({"revoked": id})))
}

pub async fn openapi() -> Json<Value> {
    Json(crate::openapi::spec())
}

pub async fn get_auth_config(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    Ok(Json(json!({
        "mode": match s.config.auth.mode {
            AuthMode::Local => "local",
            AuthMode::Oidc => "oidc",
            AuthMode::Hybrid => "hybrid",
        },
        "oidc": {
            "enabled": s.config.auth.oidc.enabled,
            "issuer": s.config.auth.oidc.issuer,
            "audience": s.config.auth.oidc.audience,
            "jwks_refresh_seconds": s.config.auth.oidc.jwks_refresh_seconds,
            "allowed_clock_skew_seconds": s.config.auth.oidc.allowed_clock_skew_seconds
        }
    })))
}

#[derive(Deserialize, Serialize)]
pub struct RbacBindingRequest {
    pub group_name: String,
    pub role: String,
}

pub async fn list_rbac_bindings(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let rows = s
        .scopes
        .list_group_bindings()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

pub async fn create_rbac_binding(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(req): Json<RbacBindingRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let role = Role::parse(&req.role)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown role: {}", req.role)))?;
    s.scopes
        .bind_group_role(&req.group_name, role)
        .await
        .map_err(ApiError::from)?;
    record_audit_with_details(
        &s.audit,
        &principal.name,
        "rbac.binding.create",
        Some(&req.group_name),
        Outcome::Success,
        Some(json!({"role": req.role})),
    )
    .await;
    Ok(Json(
        json!({"group_name": req.group_name, "role": req.role}),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct ScopeGrantRequest {
    pub principal_id: String,
    pub scope_type: String,
    pub scope_id: String,
    pub role: String,
}

pub async fn list_scope_grants(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let rows = s.scopes.list_scope_grants().await.map_err(ApiError::from)?;
    Ok(Json(json!(rows)))
}

pub async fn create_scope_grant(
    State(s): State<ApiState>,
    Authenticated(principal): Authenticated,
    Json(req): Json<ScopeGrantRequest>,
) -> Result<Json<Value>, ApiError> {
    require(&principal, Action::ManagePrincipals)?;
    let role = Role::parse(&req.role)
        .ok_or_else(|| ApiError::BadRequest(format!("unknown role: {}", req.role)))?;
    s.scopes
        .create_scope_grant(&req.principal_id, &req.scope_type, &req.scope_id, role)
        .await
        .map_err(ApiError::from)?;
    record_audit_with_details(
        &s.audit,
        &principal.name,
        "scope.grant.create",
        Some(&req.principal_id),
        Outcome::Success,
        Some(json!({
            "scope_type": req.scope_type,
            "scope_id": req.scope_id,
            "role": req.role
        })),
    )
    .await;
    Ok(Json(json!(req)))
}

async fn record_audit(
    audit: &Arc<dyn AuditLog>,
    principal: &Principal,
    action: &str,
    target: Option<&str>,
    outcome: Outcome,
) {
    let _ = audit
        .record(&principal.name, action, target, outcome, None)
        .await;
}

pub async fn record_audit_with_details(
    audit: &Arc<dyn AuditLog>,
    actor: &str,
    action: &str,
    target: Option<&str>,
    outcome: Outcome,
    details: Option<serde_json::Value>,
) {
    let _ = audit.record(actor, action, target, outcome, details).await;
}

async fn require_project_scope(
    s: &ApiState,
    principal: &Principal,
    project_id: &str,
    action: Action,
) -> Result<(), ApiError> {
    if require(principal, action).is_ok() {
        return Ok(());
    }
    let min_role = match action {
        Action::ListBuilds | Action::ReadBuild => Role::Viewer,
        Action::StartBuild => Role::Operator,
        Action::ManagePrincipals | Action::WritePolicy => Role::Admin,
    };
    let allowed = s
        .scopes
        .has_scope_role(&principal.id, "project", project_id, min_role)
        .await
        .map_err(ApiError::from)?;
    if allowed {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn ensure_build_in_project(
    s: &ApiState,
    project_id: &str,
    build_id: Uuid,
) -> Result<(), ApiError> {
    let summary = s
        .builds
        .get_summary_in_project(project_id, build_id)
        .await
        .map_err(ApiError::from)?;
    if summary.is_some() {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

fn parse_spec(req: CreateBuildRequest) -> Result<BuildSpec, ApiError> {
    let runtime = match req.runtime.as_str() {
        "java" => Runtime::Java,
        "dotnet" => Runtime::Dotnet,
        "go" => Runtime::Go,
        "node" => Runtime::Node,
        "python" => Runtime::Python,
        other => return Err(ApiError::BadRequest(format!("unknown runtime: {other}"))),
    };
    let base = match req.base.as_str() {
        "alpine" => BaseImage::Alpine,
        "debian" => BaseImage::Debian,
        "distroless" => BaseImage::Distroless,
        other => return Err(ApiError::BadRequest(format!("unknown base: {other}"))),
    };
    let mut archs = BTreeSet::new();
    if req.architectures.is_empty() {
        archs.insert(Architecture::Amd64);
    } else {
        for a in &req.architectures {
            archs.insert(match a.as_str() {
                "amd64" => Architecture::Amd64,
                "arm64" => Architecture::Arm64,
                other => {
                    return Err(ApiError::BadRequest(format!("unknown arch: {other}")));
                }
            });
        }
    }
    let mut compliance = BTreeSet::new();
    for c in &req.compliance {
        compliance.insert(match c.as_str() {
            "hipaa" => ComplianceProfile::Hipaa,
            "soc2" => ComplianceProfile::Soc2,
            "pcidss" | "pci-dss" => ComplianceProfile::PciDss,
            "cis" => ComplianceProfile::Cis,
            "fedramp-moderate" => ComplianceProfile::FedrampModerate,
            other => return Err(ApiError::BadRequest(format!("unknown compliance: {other}"))),
        });
    }
    Ok(BuildSpec {
        name: req.name,
        runtime,
        base_image: base,
        architectures: archs,
        compliance,
        hardening: HardeningOptions::strict(),
        generate_sbom: !req.no_sbom,
        sign: !req.no_sign,
    })
}
