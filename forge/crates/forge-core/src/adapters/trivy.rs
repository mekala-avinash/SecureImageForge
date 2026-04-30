//! Trivy adapter (Apache-2.0). Runs `trivy image --format json` and normalizes
//! the output into our `ScanResult` domain type.

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
pub struct TrivyConfig {
    pub trivy_path: Option<PathBuf>,
    pub bundled_prefix: Option<PathBuf>,
    /// Severities to include. Empty = all.
    pub severities: Vec<Severity>,
}

pub struct TrivyScanner {
    runner: Arc<dyn ProcessRunner>,
    config: TrivyConfig,
}

impl TrivyScanner {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: TrivyConfig) -> Self {
        Self { runner, config }
    }

    fn trivy(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.trivy_path {
            return Ok(p.clone());
        }
        resolve_tool("trivy", self.config.bundled_prefix.as_deref())
    }
}

#[async_trait]
impl Scanner for TrivyScanner {
    async fn scan(&self, image_ref: &str) -> Result<ScanResult> {
        let trivy = self.trivy()?;
        let mut spec = ProcessSpec::new(trivy.to_string_lossy().to_string())
            .arg("image")
            .arg("--format")
            .arg("json")
            .arg("--quiet")
            .arg("--no-progress");
        if !self.config.severities.is_empty() {
            spec = spec
                .arg("--severity")
                .arg(severity_filter(&self.config.severities));
        }
        spec = spec.arg(image_ref);

        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "trivy".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        parse_trivy_json(&out.stdout)
    }
}

fn severity_filter(severities: &[Severity]) -> String {
    severities
        .iter()
        .map(|s| match s {
            Severity::Low => "LOW",
            Severity::Medium => "MEDIUM",
            Severity::High => "HIGH",
            Severity::Critical => "CRITICAL",
            Severity::Unknown => "UNKNOWN",
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Deserialize)]
struct TrivyReport {
    #[serde(default, alias = "Results")]
    results: Vec<TrivyResultBlock>,
}

#[derive(Deserialize)]
struct TrivyResultBlock {
    #[serde(default, alias = "Vulnerabilities")]
    vulnerabilities: Vec<TrivyVuln>,
}

#[derive(Deserialize)]
struct TrivyVuln {
    #[serde(alias = "VulnerabilityID")]
    id: String,
    #[serde(alias = "PkgName")]
    pkg_name: String,
    #[serde(alias = "InstalledVersion")]
    installed_version: String,
    #[serde(default, alias = "FixedVersion")]
    fixed_version: Option<String>,
    #[serde(alias = "Severity")]
    severity: String,
    #[serde(default, alias = "Title")]
    title: Option<String>,
}

fn parse_trivy_json(stdout: &str) -> Result<ScanResult> {
    let report: TrivyReport = serde_json::from_str(stdout)?;
    let findings = report
        .results
        .into_iter()
        .flat_map(|r| r.vulnerabilities.into_iter())
        .map(|v| Vulnerability {
            id: v.id,
            package: v.pkg_name,
            installed_version: v.installed_version,
            fixed_version: v.fixed_version,
            severity: parse_severity(&v.severity),
            title: v.title,
        })
        .collect();
    Ok(ScanResult {
        scanner: "trivy".into(),
        scanned_at: Utc::now(),
        findings,
    })
}

fn parse_severity(s: &str) -> Severity {
    match s.to_ascii_uppercase().as_str() {
        "LOW" => Severity::Low,
        "MEDIUM" => Severity::Medium,
        "HIGH" => Severity::High,
        "CRITICAL" => Severity::Critical,
        _ => Severity::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};

    const SAMPLE: &str = r#"{
      "Results": [{
        "Vulnerabilities": [
          {"VulnerabilityID":"CVE-2024-0001","PkgName":"openssl","InstalledVersion":"3.0.0","FixedVersion":"3.0.1","Severity":"HIGH","Title":"buffer overflow"},
          {"VulnerabilityID":"CVE-2024-0002","PkgName":"zlib","InstalledVersion":"1.2.11","Severity":"CRITICAL"}
        ]
      }]
    }"#;

    #[test]
    fn parses_trivy_json() {
        let r = parse_trivy_json(SAMPLE).unwrap();
        assert_eq!(r.findings.len(), 2);
        assert_eq!(r.findings[0].id, "CVE-2024-0001");
        assert_eq!(r.findings[0].severity, Severity::High);
        assert_eq!(r.findings[1].severity, Severity::Critical);
        assert!(r.findings[1].fixed_version.is_none());
    }

    #[tokio::test]
    async fn scan_returns_findings() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("trivy") && s.args.contains(&"image".to_string()),
            ProcessOutput {
                status: 0,
                stdout: SAMPLE.into(),
                stderr: String::new(),
            },
        );
        let scanner = TrivyScanner::new(
            Arc::new(mock),
            TrivyConfig {
                trivy_path: Some("/bin/trivy".into()),
                ..Default::default()
            },
        );
        let r = scanner.scan("forge/demo:latest").await.unwrap();
        assert_eq!(r.findings.len(), 2);
        assert_eq!(r.scanner, "trivy");
    }

    #[tokio::test]
    async fn nonzero_exit_propagates() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("trivy"),
            ProcessOutput {
                status: 2,
                stdout: String::new(),
                stderr: "auth failed".into(),
            },
        );
        let scanner = TrivyScanner::new(
            Arc::new(mock),
            TrivyConfig {
                trivy_path: Some("/bin/trivy".into()),
                ..Default::default()
            },
        );
        let err = scanner.scan("priv/img").await.unwrap_err();
        assert!(matches!(err, Error::ToolFailure { code: 2, .. }));
    }
}
