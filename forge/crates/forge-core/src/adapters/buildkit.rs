//! BuildKit adapter (Apache-2.0). Drives `buildctl` against a running
//! `buildkitd` (rootless or system-managed). Streams logs to the orchestrator.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

use crate::domain::BuildSpec;
use crate::process::{resolve_tool, ProcessRunner, ProcessSpec};
use crate::tooling::{BuiltImage, ImageBuilder};
use crate::{Error, Result};

/// Configuration for the buildctl adapter.
#[derive(Debug, Clone)]
pub struct BuildkitConfig {
    /// Address of the buildkit daemon, e.g. `unix:///run/user/1000/buildkit/buildkitd.sock`
    /// or `tcp://127.0.0.1:1234`.
    pub addr: String,
    /// Optional explicit path to buildctl (else resolved from PATH / bundled prefix).
    pub buildctl_path: Option<PathBuf>,
    /// Optional bundled tools prefix.
    pub bundled_prefix: Option<PathBuf>,
    /// Local OCI tag to assign on success (e.g. `forge/<spec.name>:latest`).
    pub local_tag: Option<String>,
    /// Optional registry reference to push to. Overrides `local_tag` for the
    /// final image name; when set with `push = true` the artifact is pushed.
    pub registry_target: Option<String>,
    /// Push the resulting image to the configured registry.
    pub push: bool,
}

impl Default for BuildkitConfig {
    fn default() -> Self {
        Self {
            addr: "unix:///run/buildkit/buildkitd.sock".to_string(),
            buildctl_path: None,
            bundled_prefix: None,
            local_tag: None,
            registry_target: None,
            push: false,
        }
    }
}

pub struct BuildkitBuilder {
    runner: Arc<dyn ProcessRunner>,
    config: BuildkitConfig,
}

impl BuildkitBuilder {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: BuildkitConfig) -> Self {
        Self { runner, config }
    }

    fn buildctl(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.buildctl_path {
            return Ok(p.clone());
        }
        resolve_tool("buildctl", self.config.bundled_prefix.as_deref())
    }
}

#[async_trait]
impl ImageBuilder for BuildkitBuilder {
    async fn build(&self, spec: &BuildSpec, dockerfile: &str) -> Result<BuiltImage> {
        spec.validate()?;
        let tmp = TempDir::new().map_err(Error::Io)?;
        let context = tmp.path().to_path_buf();
        let dockerfile_path = context.join("Dockerfile");
        let mut f = tokio::fs::File::create(&dockerfile_path)
            .await
            .map_err(Error::Io)?;
        f.write_all(dockerfile.as_bytes())
            .await
            .map_err(Error::Io)?;
        f.flush().await.map_err(Error::Io)?;

        let buildctl = self.buildctl()?;
        let image_name = self
            .config
            .registry_target
            .clone()
            .or_else(|| self.config.local_tag.clone())
            .unwrap_or_else(|| format!("forge/{}:latest", sanitize(&spec.name)));
        let push_flag = if self.config.push { "true" } else { "false" };

        // Multi-arch platform string (e.g. linux/amd64,linux/arm64).
        let platforms = spec
            .architectures
            .iter()
            .map(|a| format!("linux/{}", arch_to_str(*a)))
            .collect::<Vec<_>>()
            .join(",");

        let process = ProcessSpec::new(buildctl.to_string_lossy().to_string())
            .arg("--addr")
            .arg(&self.config.addr)
            .arg("build")
            .arg("--frontend=dockerfile.v0")
            .arg(format!("--local=context={}", context.display()))
            .arg(format!("--local=dockerfile={}", context.display()))
            .arg(format!("--opt=platform={platforms}"))
            .arg(format!(
                "--output=type=image,name={image_name},push={push_flag}"
            ))
            .arg("--progress=plain");

        let mut child = self.runner.stream(process).await?;
        let mut log = String::new();
        while let Some(line) = child.lines.next().await {
            let line = line?;
            log.push_str(&line);
            log.push('\n');
        }
        let exit = child
            .wait
            .await
            .map_err(|e| Error::Internal(anyhow::anyhow!(e)))??;
        if exit != 0 {
            return Err(Error::ToolFailure {
                tool: "buildctl".into(),
                code: exit,
                stderr: log,
            });
        }

        Ok(BuiltImage {
            digest: extract_digest(&log).unwrap_or_else(|| "sha256:unknown".into()),
            reference: image_name,
            log,
        })
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn arch_to_str(a: crate::domain::Architecture) -> &'static str {
    match a {
        crate::domain::Architecture::Amd64 => "amd64",
        crate::domain::Architecture::Arm64 => "arm64",
    }
}

/// Best-effort parse of the `sha256:<hex>` digest from buildctl's plain output.
fn extract_digest(log: &str) -> Option<String> {
    log.split_whitespace()
        .find(|tok| tok.starts_with("sha256:") && tok.len() == 71)
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Architecture, BaseImage, ComplianceProfile, HardeningOptions, Runtime};
    use crate::process::{MockRunner, ProcessOutput};
    use std::collections::BTreeSet;

    fn build_spec() -> BuildSpec {
        let mut archs = BTreeSet::new();
        archs.insert(Architecture::Amd64);
        BuildSpec {
            name: "demo".into(),
            runtime: Runtime::Go,
            base_image: BaseImage::Alpine,
            architectures: archs,
            compliance: BTreeSet::from([ComplianceProfile::Cis]),
            hardening: HardeningOptions::strict(),
            generate_sbom: false,
            sign: false,
        }
    }

    #[tokio::test]
    async fn happy_path_returns_built_image() {
        let mock = MockRunner::new();
        mock.expect(
            |spec| spec.program.ends_with("buildctl") && spec.args.contains(&"build".to_string()),
            ProcessOutput {
                status: 0,
                stdout: "exporting sha256:aaaabbbbccccddddeeeeffff0000111122223333444455556666777788889999 done\n".into(),
                stderr: String::new(),
            },
        );
        let cfg = BuildkitConfig {
            buildctl_path: Some("/usr/local/bin/buildctl".into()),
            ..BuildkitConfig::default()
        };
        let builder = BuildkitBuilder::new(Arc::new(mock), cfg);
        let result = builder
            .build(&build_spec(), "FROM scratch\n")
            .await
            .unwrap();
        assert!(result.reference.starts_with("forge/demo:"));
    }

    #[tokio::test]
    async fn nonzero_exit_propagates_failure() {
        let mock = MockRunner::new();
        mock.expect(
            |spec| spec.program.ends_with("buildctl"),
            ProcessOutput {
                status: 1,
                stdout: String::new(),
                stderr: "boom".into(),
            },
        );
        let cfg = BuildkitConfig {
            buildctl_path: Some("/usr/local/bin/buildctl".into()),
            ..BuildkitConfig::default()
        };
        let builder = BuildkitBuilder::new(Arc::new(mock), cfg);
        let err = builder
            .build(&build_spec(), "FROM scratch\n")
            .await
            .unwrap_err();
        match err {
            Error::ToolFailure { tool, code, .. } => {
                assert_eq!(tool, "buildctl");
                assert_eq!(code, 1);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn extracts_digest_from_log() {
        let line = "exporting sha256:aaaabbbbccccddddeeeeffff0000111122223333444455556666777788889999 done";
        let d = extract_digest(line).unwrap();
        assert!(d.starts_with("sha256:"));
        assert_eq!(d.len(), 71);
    }
}
