//! Read-side helpers (list, summary, scan, sbom, log).

use anyhow::Result;
use uuid::Uuid;

use forge_core::domain::{Sbom, ScanResult};
use forge_core::logs::LogStore;
use forge_core::repo::{BuildRepo, BuildSummary};

use crate::state::runtime;

pub async fn list_async(repo: &std::sync::Arc<dyn BuildRepo>, limit: i64) -> Result<Vec<BuildSummary>> {
    Ok(repo.list(limit).await?)
}

pub async fn summary_async(repo: &std::sync::Arc<dyn BuildRepo>, id: Uuid) -> Result<Option<BuildSummary>> {
    Ok(repo.get_summary(id).await?)
}

pub async fn scan_async(repo: &std::sync::Arc<dyn BuildRepo>, id: Uuid) -> Result<Option<ScanResult>> {
    Ok(repo.get_scan(id).await?)
}

pub async fn sbom_async(repo: &std::sync::Arc<dyn BuildRepo>, id: Uuid) -> Result<Option<Sbom>> {
    Ok(repo.get_sbom(id).await?)
}

pub async fn log_async(logs: &std::sync::Arc<dyn LogStore>, id: Uuid) -> Result<Option<String>> {
    Ok(logs.read(id).await?)
}
