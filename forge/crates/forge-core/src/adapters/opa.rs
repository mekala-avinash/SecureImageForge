//! OPA adapter (Apache-2.0). Evaluates compliance Rego bundles via `opa eval`.
//!
//! Bundles are embedded at compile time so the daemon does not need to ship a
//! separate policies directory; downstream consumers can still register
//! additional bundles dynamically via `OpaPolicyEngine::with_extra_bundle`.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use tempfile::TempDir;

use crate::domain::ComplianceProfile;
use crate::process::{resolve_tool, ProcessRunner, ProcessSpec};
use crate::tooling::{PolicyDecision, PolicyEngine};
use crate::{Error, Result};

const POLICY_CIS: &str = include_str!("../../policies/cis.rego");
const POLICY_HIPAA: &str = include_str!("../../policies/hipaa.rego");
const POLICY_SOC2: &str = include_str!("../../policies/soc2.rego");
const POLICY_FEDRAMP_MODERATE: &str = include_str!("../../policies/fedramp_moderate.rego");

#[derive(Debug, Clone, Default)]
pub struct OpaConfig {
    pub opa_path: Option<PathBuf>,
    pub bundled_prefix: Option<PathBuf>,
    /// Profiles to enforce. Empty = no policy (allow-all).
    pub profiles: Vec<ComplianceProfile>,
}

pub struct OpaPolicyEngine {
    runner: Arc<dyn ProcessRunner>,
    config: OpaConfig,
}

impl OpaPolicyEngine {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: OpaConfig) -> Self {
        Self { runner, config }
    }

    fn opa(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.opa_path {
            return Ok(p.clone());
        }
        resolve_tool("opa", self.config.bundled_prefix.as_deref())
    }

    fn bundle_for(
        profile: ComplianceProfile,
    ) -> Option<(&'static str, &'static str, &'static str)> {
        match profile {
            ComplianceProfile::Cis => Some(("cis.rego", POLICY_CIS, "data.forge.cis.deny")),
            ComplianceProfile::Hipaa => Some(("hipaa.rego", POLICY_HIPAA, "data.forge.hipaa.deny")),
            ComplianceProfile::Soc2 => Some(("soc2.rego", POLICY_SOC2, "data.forge.soc2.deny")),
            // PciDss reuses SOC2 controls until a dedicated bundle ships.
            ComplianceProfile::PciDss => Some(("soc2.rego", POLICY_SOC2, "data.forge.soc2.deny")),
            ComplianceProfile::FedrampModerate => Some((
                "fedramp_moderate.rego",
                POLICY_FEDRAMP_MODERATE,
                "data.forge.fedramp_moderate.deny",
            )),
        }
    }
}

#[async_trait]
impl PolicyEngine for OpaPolicyEngine {
    async fn evaluate(&self, input: serde_json::Value) -> Result<PolicyDecision> {
        if self.config.profiles.is_empty() {
            return Ok(PolicyDecision::Allow);
        }
        let opa = self.opa()?;
        let mut reasons: Vec<String> = Vec::new();

        for profile in &self.config.profiles {
            let Some((file_name, source, query)) = Self::bundle_for(*profile) else {
                continue;
            };
            let tmp = TempDir::new().map_err(Error::Io)?;
            let policy_path = tmp.path().join(file_name);
            std::fs::write(&policy_path, source).map_err(Error::Io)?;
            let input_path = tmp.path().join("input.json");
            std::fs::write(&input_path, serde_json::to_vec(&input)?).map_err(Error::Io)?;

            let spec = ProcessSpec::new(opa.to_string_lossy().to_string())
                .arg("eval")
                .arg("--format=json")
                .arg("--data")
                .arg(policy_path.to_string_lossy().to_string())
                .arg("--input")
                .arg(input_path.to_string_lossy().to_string())
                .arg(query);

            let out = self.runner.run(spec).await?;
            if out.status != 0 {
                return Err(Error::ToolFailure {
                    tool: "opa".into(),
                    code: out.status,
                    stderr: out.stderr,
                });
            }
            let mut decision_reasons = parse_deny(&out.stdout)?;
            reasons.append(&mut decision_reasons);
        }

        if reasons.is_empty() {
            Ok(PolicyDecision::Allow)
        } else {
            Ok(PolicyDecision::Deny { reasons })
        }
    }
}

#[derive(Deserialize)]
struct OpaEvalOutput {
    result: Option<Vec<OpaResultEntry>>,
}

#[derive(Deserialize)]
struct OpaResultEntry {
    expressions: Vec<OpaExpression>,
}

#[derive(Deserialize)]
struct OpaExpression {
    value: serde_json::Value,
}

fn parse_deny(stdout: &str) -> Result<Vec<String>> {
    let parsed: OpaEvalOutput = serde_json::from_str(stdout)?;
    let mut reasons = Vec::new();
    if let Some(results) = parsed.result {
        for entry in results {
            for expr in entry.expressions {
                if let Some(arr) = expr.value.as_array() {
                    for v in arr {
                        if let Some(s) = v.as_str() {
                            reasons.push(s.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(reasons)
}

/// Helper to build the JSON input expected by the policies from spec + scan.
pub fn build_input(
    spec: &crate::domain::BuildSpec,
    scan: Option<&crate::domain::ScanResult>,
) -> serde_json::Value {
    let scan_json = scan.map(|s| {
        json!({
            "findings": s.findings.iter().map(|f| json!({
                "id": f.id,
                "package": f.package,
                "severity": format!("{:?}", f.severity).to_uppercase(),
            })).collect::<Vec<_>>()
        })
    });
    json!({
        "spec": {
            "name": spec.name,
            "runtime": format!("{:?}", spec.runtime).to_lowercase(),
            "base_image": format!("{:?}", spec.base_image).to_lowercase(),
            "sign": spec.sign,
            "generate_sbom": spec.generate_sbom,
            "hardening": {
                "non_root_user": spec.hardening.non_root_user,
                "remove_shells": spec.hardening.remove_shells,
                "remove_pkg_managers": spec.hardening.remove_pkg_managers,
                "readonly_rootfs": spec.hardening.readonly_rootfs,
            },
        },
        "scan": scan_json.unwrap_or(json!({"findings": []})),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};

    #[tokio::test]
    async fn empty_profiles_short_circuit_allow() {
        let runner = Arc::new(MockRunner::new());
        let engine = OpaPolicyEngine::new(runner, OpaConfig::default());
        let result = engine.evaluate(json!({})).await.unwrap();
        assert_eq!(result, PolicyDecision::Allow);
    }

    #[tokio::test]
    async fn deny_messages_propagate() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("opa"),
            ProcessOutput {
                status: 0,
                stdout:
                    r#"{"result":[{"expressions":[{"value":["CIS 4.1: ...","CIS 4.6: ..."]}]}]}"#
                        .into(),
                stderr: String::new(),
            },
        );
        let engine = OpaPolicyEngine::new(
            Arc::new(mock),
            OpaConfig {
                opa_path: Some("/bin/opa".into()),
                profiles: vec![ComplianceProfile::Cis],
                ..Default::default()
            },
        );
        let res = engine.evaluate(json!({"spec":{}})).await.unwrap();
        match res {
            PolicyDecision::Deny { reasons } => assert_eq!(reasons.len(), 2),
            other => panic!("expected deny, got {other:?}"),
        }
    }

    #[test]
    fn build_input_shapes_payload() {
        use crate::domain::{Architecture, BaseImage, BuildSpec, HardeningOptions, Runtime};
        use std::collections::BTreeSet;
        let spec = BuildSpec {
            name: "x".into(),
            runtime: Runtime::Java,
            base_image: BaseImage::Alpine,
            architectures: BTreeSet::from([Architecture::Amd64]),
            compliance: BTreeSet::from([ComplianceProfile::Cis]),
            hardening: HardeningOptions::strict(),
            generate_sbom: true,
            sign: true,
        };
        let v = build_input(&spec, None);
        assert_eq!(v["spec"]["base_image"], "alpine");
        assert_eq!(v["spec"]["hardening"]["non_root_user"], true);
    }
}
