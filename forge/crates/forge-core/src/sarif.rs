//! Convert a `ScanResult` to a SARIF v2.1.0 document so CI pipelines (GitHub
//! Code Scanning, Azure DevOps, etc.) can ingest forge findings natively.

use serde_json::{json, Value};

use crate::domain::{ScanResult, Severity};

pub fn to_sarif(scan: &ScanResult, image_ref: &str) -> Value {
    let rules = scan
        .findings
        .iter()
        .map(|v| {
            json!({
                "id": v.id,
                "name": v.id,
                "shortDescription": { "text": v.title.clone().unwrap_or_else(|| v.id.clone()) },
                "help": { "text": format!("Package: {} {}", v.package, v.installed_version) },
                "properties": {
                    "security-severity": severity_score(v.severity),
                    "tags": ["security", "vulnerability"],
                }
            })
        })
        .collect::<Vec<_>>();

    let results = scan
        .findings
        .iter()
        .map(|v| {
            json!({
                "ruleId": v.id,
                "level": severity_level(v.severity),
                "message": { "text": format!(
                    "{} ({} {} -> {})",
                    v.id,
                    v.package,
                    v.installed_version,
                    v.fixed_version.clone().unwrap_or_else(|| "no-fix".into())
                ) },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": image_ref }
                    }
                }],
            })
        })
        .collect::<Vec<_>>();

    json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": scan.scanner.clone(),
                    "informationUri": "https://github.com/aquasecurity/trivy",
                    "rules": rules,
                }
            },
            "results": results,
        }],
    })
}

fn severity_level(s: Severity) -> &'static str {
    match s {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Unknown => "note",
    }
}

fn severity_score(s: Severity) -> &'static str {
    match s {
        Severity::Critical => "9.5",
        Severity::High => "7.5",
        Severity::Medium => "5.0",
        Severity::Low => "2.5",
        Severity::Unknown => "0.0",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Vulnerability;
    use chrono::Utc;

    #[test]
    fn empty_scan_produces_valid_skeleton() {
        let scan = ScanResult {
            scanner: "trivy".into(),
            scanned_at: Utc::now(),
            findings: vec![],
        };
        let s = to_sarif(&scan, "img:tag");
        assert_eq!(s["version"], "2.1.0");
        assert_eq!(s["runs"][0]["tool"]["driver"]["name"], "trivy");
        assert_eq!(s["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn mapping_of_severity_to_level() {
        let scan = ScanResult {
            scanner: "trivy".into(),
            scanned_at: Utc::now(),
            findings: vec![
                Vulnerability {
                    id: "CVE-1".into(),
                    package: "p".into(),
                    installed_version: "1".into(),
                    fixed_version: Some("2".into()),
                    severity: Severity::Critical,
                    title: None,
                },
                Vulnerability {
                    id: "CVE-2".into(),
                    package: "q".into(),
                    installed_version: "1".into(),
                    fixed_version: None,
                    severity: Severity::Low,
                    title: None,
                },
            ],
        };
        let s = to_sarif(&scan, "img");
        let results = s["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "note");
    }
}
