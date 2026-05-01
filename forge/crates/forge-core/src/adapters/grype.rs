//! Grype adapter (Apache-2.0). A second scanner whose findings are merged
//! with Trivy's via `MergedScanner` so a CVE missed by one tool but caught
//! by the other still gates the build.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::domain::{ScanResult, Severity, Vulnerability};
use crate::process::{resolve_tool, ProcessRunner, ProcessSpec};
use crate::tooling::Scanner;
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct GrypeConfig {
    pub grype_path: Option<PathBuf>,
    pub bundled_prefix: Option<PathBuf>,
    pub offline_mode: bool,
}

pub struct GrypeScanner {
    runner: Arc<dyn ProcessRunner>,
    config: GrypeConfig,
}

impl GrypeScanner {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: GrypeConfig) -> Self {
        Self { runner, config }
    }

    fn grype(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.grype_path {
            return Ok(p.clone());
        }
        resolve_tool("grype", self.config.bundled_prefix.as_deref())
    }
}

#[async_trait]
impl Scanner for GrypeScanner {
    async fn scan(&self, image_ref: &str) -> Result<ScanResult> {
        let grype = self.grype()?;
        
        if !self.config.offline_mode {
            let update_spec = ProcessSpec::new(grype.to_string_lossy().to_string())
                .arg("db")
                .arg("update");
            let _ = self.runner.run(update_spec).await; // Ignore failure, try best-effort update
        }

        let mut spec = ProcessSpec::new(grype.to_string_lossy().to_string())
            .arg(image_ref)
            .arg("-o")
            .arg("json")
            .arg("--quiet");
            
        if self.config.offline_mode {
            spec = spec.env("GRYPE_DB_AUTO_UPDATE", "false");
        }
            
        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "grype".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        parse_grype_json(&out.stdout)
    }
}

#[derive(Deserialize)]
struct GrypeReport {
    matches: Vec<GrypeMatch>,
}

#[derive(Deserialize)]
struct GrypeMatch {
    vulnerability: GrypeVuln,
    artifact: GrypeArtifact,
}

#[derive(Deserialize)]
struct GrypeVuln {
    id: String,
    severity: String,
    fix: Option<GrypeFix>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct GrypeFix {
    versions: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct GrypeArtifact {
    name: String,
    version: String,
}

fn parse_grype_json(stdout: &str) -> Result<ScanResult> {
    let report: GrypeReport = serde_json::from_str(stdout)?;
    let findings = report
        .matches
        .into_iter()
        .map(|m| Vulnerability {
            id: m.vulnerability.id,
            package: m.artifact.name,
            installed_version: m.artifact.version,
            fixed_version: m
                .vulnerability
                .fix
                .and_then(|f| f.versions)
                .and_then(|v| v.into_iter().next()),
            severity: parse_severity(&m.vulnerability.severity),
            title: m.vulnerability.description,
        })
        .collect();
    Ok(ScanResult {
        scanner: "grype".into(),
        scanned_at: Utc::now(),
        findings,
    })
}

fn parse_severity(s: &str) -> Severity {
    match s.to_ascii_uppercase().as_str() {
        "LOW" | "NEGLIGIBLE" => Severity::Low,
        "MEDIUM" => Severity::Medium,
        "HIGH" => Severity::High,
        "CRITICAL" => Severity::Critical,
        _ => Severity::Unknown,
    }
}

/// Combines two Scanners (typically Trivy + Grype) and unions their findings,
/// de-duplicated by `(id, package)`. Higher severity wins on conflict.
pub struct MergedScanner {
    pub primary: Arc<dyn Scanner>,
    pub secondary: Arc<dyn Scanner>,
}

#[async_trait]
impl Scanner for MergedScanner {
    async fn scan(&self, image_ref: &str) -> Result<ScanResult> {
        let (a, b) = tokio::join!(self.primary.scan(image_ref), self.secondary.scan(image_ref));
        let mut findings = match a {
            Ok(r) => r.findings,
            Err(e) => {
                tracing::warn!(error = %e, "primary scanner failed");
                Vec::new()
            }
        };
        let extra = match b {
            Ok(r) => r.findings,
            Err(e) => {
                tracing::warn!(error = %e, "secondary scanner failed");
                Vec::new()
            }
        };
        for f in extra {
            let key = (f.id.clone(), f.package.clone());
            if let Some(existing) = findings
                .iter_mut()
                .find(|x| (x.id.clone(), x.package.clone()) == key)
            {
                if f.severity > existing.severity {
                    existing.severity = f.severity;
                }
                if existing.fixed_version.is_none() {
                    existing.fixed_version = f.fixed_version.clone();
                }
            } else {
                findings.push(f);
            }
        }
        Ok(ScanResult {
            scanner: "merged".into(),
            scanned_at: Utc::now(),
            findings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};
    use async_trait::async_trait;

    const SAMPLE: &str = r#"{
      "matches": [
        {
          "vulnerability": {"id":"GHSA-1","severity":"High","description":"x","fix":{"versions":["1.2.0"]}},
          "artifact": {"name":"openssl","version":"1.1.0"}
        }
      ]
    }"#;

    #[test]
    fn parses_grype_json() {
        let r = parse_grype_json(SAMPLE).unwrap();
        assert_eq!(r.findings.len(), 1);
        assert_eq!(r.findings[0].severity, Severity::High);
        assert_eq!(r.findings[0].fixed_version.as_deref(), Some("1.2.0"));
    }

    #[tokio::test]
    async fn merge_dedupes_and_picks_higher_severity() {
        struct StaticScanner(ScanResult);
        #[async_trait]
        impl Scanner for StaticScanner {
            async fn scan(&self, _image_ref: &str) -> Result<ScanResult> {
                Ok(self.0.clone())
            }
        }
        let primary = ScanResult {
            scanner: "trivy".into(),
            scanned_at: Utc::now(),
            findings: vec![Vulnerability {
                id: "CVE-1".into(),
                package: "p".into(),
                installed_version: "1".into(),
                fixed_version: None,
                severity: Severity::High,
                title: None,
            }],
        };
        let secondary = ScanResult {
            scanner: "grype".into(),
            scanned_at: Utc::now(),
            findings: vec![
                Vulnerability {
                    id: "CVE-1".into(),
                    package: "p".into(),
                    installed_version: "1".into(),
                    fixed_version: Some("1.0.1".into()),
                    severity: Severity::Critical, // promotes severity
                    title: None,
                },
                Vulnerability {
                    id: "GHSA-2".into(),
                    package: "q".into(),
                    installed_version: "1".into(),
                    fixed_version: None,
                    severity: Severity::Medium,
                    title: None,
                },
            ],
        };
        let merged = MergedScanner {
            primary: Arc::new(StaticScanner(primary)),
            secondary: Arc::new(StaticScanner(secondary)),
        };
        let r = merged.scan("img").await.unwrap();
        assert_eq!(r.findings.len(), 2);
        let cve1 = r.findings.iter().find(|f| f.id == "CVE-1").unwrap();
        assert_eq!(cve1.severity, Severity::Critical);
        assert_eq!(cve1.fixed_version.as_deref(), Some("1.0.1"));
    }

    #[tokio::test]
    async fn nonzero_exit_propagates() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("grype"),
            ProcessOutput {
                status: 2,
                stdout: String::new(),
                stderr: "boom".into(),
            },
        );
        let scanner = GrypeScanner::new(
            Arc::new(mock),
            GrypeConfig {
                grype_path: Some("/bin/grype".into()),
                ..Default::default()
            },
        );
        let err = scanner.scan("img").await.unwrap_err();
        assert!(matches!(err, Error::ToolFailure { code: 2, .. }));
    }
}
