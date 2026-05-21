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
    /// Resolved registry authentication credentials.
    pub registry_auth: Option<crate::registry::ResolvedAuth>,
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
            registry_auth: None,
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
        
        if self.config.addr != "mock" {
            preflight_check(&self.config.addr).await?;
        }

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

        // If we have registry credentials, write them to a temporary config.json
        // inside DOCKER_CONFIG to pass to buildctl.
        let mut process_env = std::collections::HashMap::new();
        if let Some(auth) = &self.config.registry_auth {
            let docker_config_dir = tmp.path().join(".docker");
            tokio::fs::create_dir_all(&docker_config_dir)
                .await
                .map_err(Error::Io)?;
            let config_json_path = docker_config_dir.join("config.json");
            let encoded_auth = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                format!("{}:{}", auth.username, auth.password).as_bytes(),
            );
            let config_json = serde_json::json!({
                "auths": {
                    &auth.registry: {
                        "auth": encoded_auth
                    }
                }
            });
            tokio::fs::write(&config_json_path, config_json.to_string())
                .await
                .map_err(Error::Io)?;
            process_env.insert("DOCKER_CONFIG".to_string(), docker_config_dir.to_string_lossy().to_string());
        }

        let buildctl = self.buildctl()?;
        let image_name = self
            .config
            .registry_target
            .clone()
            .or_else(|| self.config.local_tag.clone())
            .unwrap_or_else(|| format!("forge/{}:latest", sanitize(&spec.name)));
        // Multi-arch platform string (e.g. linux/amd64,linux/arm64).
        let platforms = spec
            .architectures
            .iter()
            .map(|a| format!("linux/{}", arch_to_str(*a)))
            .collect::<Vec<_>>()
            .join(",");

        let output_arg = if self.config.push {
            format!("--output=type=image,name={},push=true", image_name)
        } else {
            let home = dirs::home_dir().ok_or_else(|| Error::Internal(anyhow::anyhow!("Could not find home directory")))?;
            let tmp_dir = home.join(".secureimageforge").join("tmp");
            std::fs::create_dir_all(&tmp_dir).map_err(Error::Io)?;
            let tar_name = format!("{}.tar", sanitize(&image_name));
            let tar_path = tmp_dir.join(tar_name);
            format!("--output=type=docker,dest={},name={}", tar_path.display(), image_name)
        };

        let mut process = ProcessSpec::new(buildctl.to_string_lossy().to_string())
            .arg("--addr")
            .arg(&self.config.addr)
            .arg("build")
            .arg("--frontend=dockerfile.v0")
            .arg(format!("--local=context={}", context.display()))
            .arg(format!("--local=dockerfile={}", context.display()))
            .arg(format!("--opt=platform={platforms}"))
            .arg(output_arg)
            .arg("--progress=plain");

        for (k, v) in process_env {
            process = process.env(&k, &v);
        }

        let mut child = self.runner.stream(process).await?;
        let mut log = String::new();
        while let Some(line) = child.lines.next().await {
            let line = line?;
            log.push_str(&line);
            log.push('\n');
        }
        let exit = (&mut child.wait)
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

async fn preflight_check(addr: &str) -> Result<()> {
    if let Some(path) = addr.strip_prefix("unix://") {
        if !std::path::Path::new(path).exists() {
            return Err(Error::ToolFailure {
                tool: "buildkitd".into(),
                code: 1,
                stderr: format!("buildkitd socket not found at {}. Ensure buildkitd is running (e.g. `buildkitd --rootless &`)", path),
            });
        }
    } else if let Some(addr_str) = addr.strip_prefix("tcp://") {
        if tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::TcpStream::connect(addr_str),
        )
        .await
        .is_err()
        {
            return Err(Error::ToolFailure {
                tool: "buildkitd".into(),
                code: 1,
                stderr: format!("Could not connect to buildkitd at {}. Ensure buildkitd is running.", addr_str),
            });
        }
    }
    Ok(())
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
        let builder = BuildkitBuilder::new(
            Arc::new(mock),
            BuildkitConfig {
                addr: "mock".into(),
                ..Default::default()
            },
        );
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
            addr: "mock".into(),
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
