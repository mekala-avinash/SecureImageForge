//! Read-side helpers (list, summary, scan, sbom, log).

use anyhow::Result;
use uuid::Uuid;

use forge_core::domain::{Sbom, ScanResult};
use forge_core::logs::LogStore;
use forge_core::repo::{BuildRepo, BuildSummary};

use crate::state::runtime;

pub fn list(repo: &BuildRepo, limit: i64) -> Result<Vec<BuildSummary>> {
    let repo = repo.clone();
    runtime().block_on(async move { Ok(repo.list(limit).await?) })
}

pub fn summary(repo: &BuildRepo, id: Uuid) -> Result<Option<BuildSummary>> {
    let repo = repo.clone();
    runtime().block_on(async move { Ok(repo.get_summary(id).await?) })
}

pub fn scan(repo: &BuildRepo, id: Uuid) -> Result<Option<ScanResult>> {
    let repo = repo.clone();
    runtime().block_on(async move { Ok(repo.get_scan(id).await?) })
}

pub fn sbom(repo: &BuildRepo, id: Uuid) -> Result<Option<Sbom>> {
    let repo = repo.clone();
    runtime().block_on(async move { Ok(repo.get_sbom(id).await?) })
}

pub fn log(logs: &LogStore, id: Uuid) -> Result<Option<String>> {
    Ok(logs.read(id)?)
}
