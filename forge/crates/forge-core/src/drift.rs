//! Drift detection. For every persisted build artifact we periodically
//! re-scan and record the delta against the original scan; the API surfaces
//! "new critical/high CVE" counts so security teams see vulnerabilities that
//! emerge after a build is signed and shipped.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::Serialize;
use sqlx::Row;
use uuid::Uuid;

use crate::audit::{AuditLog, Outcome};
use crate::domain::{ScanResult, Severity, Vulnerability};
use crate::repo::BuildRepo;
use crate::storage::Storage;
use crate::tooling::Scanner;
use crate::Result;

#[derive(Debug, Clone, Serialize)]
pub struct DriftSnapshot {
    pub id: i64,
    pub build_id: String,
    pub scanner: String,
    pub scanned_at: String,
    pub new_critical: i64,
    pub new_high: i64,
    pub findings_count: usize,
}

pub struct DriftDetector {
    pub repo: BuildRepo,
    pub storage: Storage,
    pub scanner: Arc<dyn Scanner>,
    pub audit: AuditLog,
}

impl DriftDetector {
    pub async fn rescan_one(&self, build_id: Uuid, image_ref: &str) -> Result<DriftSnapshot> {
        let baseline = self
            .repo
            .get_scan(build_id)
            .await?
            .map(|r| ids_by_severity(&r))
            .unwrap_or_default();
        let now = self.scanner.scan(image_ref).await?;
        let snapshot = compute_snapshot(build_id, &baseline, &now);
        self.persist(&snapshot, &now).await?;
        let _ = self
            .audit
            .record(
                "drift-scheduler",
                "drift.rescan",
                Some(&build_id.to_string()),
                Outcome::Success,
                Some(serde_json::json!({
                    "new_critical": snapshot.new_critical,
                    "new_high": snapshot.new_high,
                })),
            )
            .await;
        Ok(snapshot)
    }

    pub async fn list(&self, build_id: Uuid, limit: i64) -> Result<Vec<DriftSnapshot>> {
        let rows = sqlx::query(
            r#"SELECT id, build_id, scanner, scanned_at, findings, new_critical, new_high
               FROM drift_snapshots WHERE build_id = ? ORDER BY id DESC LIMIT ?"#,
        )
        .bind(build_id.to_string())
        .bind(limit)
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let findings_str: String = r.get("findings");
                let findings: Vec<Vulnerability> =
                    serde_json::from_str(&findings_str).unwrap_or_default();
                DriftSnapshot {
                    id: r.get::<i64, _>("id"),
                    build_id: r.get("build_id"),
                    scanner: r.get("scanner"),
                    scanned_at: r.get("scanned_at"),
                    new_critical: r.get("new_critical"),
                    new_high: r.get("new_high"),
                    findings_count: findings.len(),
                }
            })
            .collect())
    }

    async fn persist(&self, snap: &DriftSnapshot, scan: &ScanResult) -> Result<()> {
        let findings = serde_json::to_string(&scan.findings)?;
        sqlx::query(
            r#"INSERT INTO drift_snapshots (build_id, scanned_at, scanner, findings, new_critical, new_high)
               VALUES (?, ?, ?, ?, ?, ?)"#,
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

/// Schedule periodic rescans. Lives in core so the API daemon can run it
/// directly without re-implementing it.
pub async fn run_scheduler(detector: Arc<DriftDetector>, interval: Duration) {
    loop {
        match detector.repo.drift_targets(500).await {
            Ok(targets) => {
                for (id, image_ref) in targets {
                    if let Err(e) = detector.rescan_one(id, &image_ref).await {
                        tracing::warn!(error = %e, build_id = %id, "drift rescan failed");
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load drift targets");
            }
        }
        tokio::time::sleep(interval).await;
    }
}

pub fn compute_snapshot(
    build_id: Uuid,
    baseline: &SeveritySet,
    fresh: &ScanResult,
) -> DriftSnapshot {
    let mut new_critical = 0;
    let mut new_high = 0;
    for f in &fresh.findings {
        let matched = match f.severity {
            Severity::Critical => baseline.critical.contains(&f.id),
            Severity::High => baseline.high.contains(&f.id),
            _ => true, // we only count critical/high deltas
        };
        if matched {
            continue;
        }
        match f.severity {
            Severity::Critical => new_critical += 1,
            Severity::High => new_high += 1,
            _ => {}
        }
    }
    DriftSnapshot {
        id: 0,
        build_id: build_id.to_string(),
        scanner: fresh.scanner.clone(),
        scanned_at: Utc::now().to_rfc3339(),
        new_critical,
        new_high,
        findings_count: fresh.findings.len(),
    }
}

#[derive(Debug, Default)]
pub struct SeveritySet {
    pub critical: BTreeSet<String>,
    pub high: BTreeSet<String>,
}

fn ids_by_severity(r: &ScanResult) -> SeveritySet {
    let mut s = SeveritySet::default();
    for f in &r.findings {
        match f.severity {
            Severity::Critical => {
                s.critical.insert(f.id.clone());
            }
            Severity::High => {
                s.high.insert(f.id.clone());
            }
            _ => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn vuln(id: &str, sev: Severity) -> Vulnerability {
        Vulnerability {
            id: id.into(),
            package: "p".into(),
            installed_version: "1".into(),
            fixed_version: None,
            severity: sev,
            title: None,
        }
    }

    #[test]
    fn detects_only_new_critical_or_high() {
        let baseline = SeveritySet {
            critical: BTreeSet::from(["CVE-1".to_string()]),
            high: BTreeSet::from(["CVE-2".to_string()]),
        };
        let fresh = ScanResult {
            scanner: "trivy".into(),
            scanned_at: Utc::now(),
            findings: vec![
                vuln("CVE-1", Severity::Critical), // unchanged
                vuln("CVE-2", Severity::High),     // unchanged
                vuln("CVE-3", Severity::Critical), // new!
                vuln("CVE-4", Severity::High),     // new!
                vuln("CVE-5", Severity::Medium),   // ignored
            ],
        };
        let snap = compute_snapshot(Uuid::new_v4(), &baseline, &fresh);
        assert_eq!(snap.new_critical, 1);
        assert_eq!(snap.new_high, 1);
        assert_eq!(snap.findings_count, 5);
    }
}
