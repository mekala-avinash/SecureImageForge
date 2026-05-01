//! Cosign adapter (Apache-2.0). Supports keyless OIDC and key-based signing.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use crate::process::{resolve_tool, ProcessRunner, ProcessSpec};
use crate::tooling::{Attestor, Signer, Verifier};
use crate::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct CosignConfig {
    pub cosign_path: Option<PathBuf>,
    pub bundled_prefix: Option<PathBuf>,
    /// Path to a cosign.key. If None, falls back to keyless OIDC.
    pub key_path: Option<PathBuf>,
    /// OIDC issuer URL when using keyless mode.
    pub oidc_issuer: Option<String>,
}

pub struct CosignSigner {
    runner: Arc<dyn ProcessRunner>,
    config: CosignConfig,
}

impl CosignSigner {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: CosignConfig) -> Self {
        Self { runner, config }
    }

    fn cosign(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.cosign_path {
            return Ok(p.clone());
        }
        resolve_tool("cosign", self.config.bundled_prefix.as_deref())
    }

    fn auth_args(&self, mut spec: ProcessSpec) -> ProcessSpec {
        if let Some(key) = &self.config.key_path {
            spec = spec.arg("--key").arg(key.to_string_lossy().to_string());
        } else {
            spec = spec.env("COSIGN_EXPERIMENTAL", "1");
            if let Some(iss) = &self.config.oidc_issuer {
                spec = spec.arg("--oidc-issuer").arg(iss);
            }
        }
        spec
    }
}

#[async_trait]
impl Signer for CosignSigner {
    async fn sign(&self, image_ref: &str) -> Result<()> {
        let cosign = self.cosign()?;
        let mut spec = ProcessSpec::new(cosign.to_string_lossy().to_string())
            .arg("sign")
            .arg("--yes");
        spec = self.auth_args(spec);
        spec = spec.arg(image_ref);

        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "cosign".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        Ok(())
    }
}

#[async_trait]
impl Attestor for CosignSigner {
    async fn attest(&self, image_ref: &str, predicate_json: &str) -> Result<()> {
        let cosign = self.cosign()?;
        let dir = tempfile::tempdir()?;
        let predicate_path = dir.path().join("slsa-statement.json");
        std::fs::write(&predicate_path, predicate_json)?;
        let spec = self
            .auth_args(
                ProcessSpec::new(cosign.to_string_lossy().to_string())
                    .arg("attest")
                    .arg("--yes")
                    .arg("--predicate")
                    .arg(predicate_path.to_string_lossy().to_string())
                    .arg("--type")
                    .arg("slsaprovenance"),
            )
            .arg(image_ref);

        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "cosign".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        Ok(())
    }
}

#[async_trait]
impl Verifier for CosignSigner {
    async fn verify(&self, image_ref: &str) -> Result<()> {
        let cosign = self.cosign()?;
        let mut spec = ProcessSpec::new(cosign.to_string_lossy().to_string())
            .arg("verify");
        
        if let Some(key) = &self.config.key_path {
            spec = spec.arg("--key").arg(key.to_string_lossy().to_string());
        } else {
            spec = self.auth_args(spec);
        }
        spec = spec.arg(image_ref);

        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "cosign".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};

    #[tokio::test]
    async fn key_based_sign_succeeds() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.args.iter().any(|a| a == "--key"),
            ProcessOutput {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        let signer = CosignSigner::new(
            Arc::new(mock),
            CosignConfig {
                cosign_path: Some("/bin/cosign".into()),
                key_path: Some("/keys/cosign.key".into()),
                ..Default::default()
            },
        );
        signer.sign("img").await.unwrap();
    }

    #[tokio::test]
    async fn keyless_uses_experimental_env() {
        let mock = MockRunner::new();
        mock.expect(
            |s| {
                s.env
                    .get("COSIGN_EXPERIMENTAL")
                    .map(|v| v == "1")
                    .unwrap_or(false)
            },
            ProcessOutput {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        let signer = CosignSigner::new(
            Arc::new(mock),
            CosignConfig {
                cosign_path: Some("/bin/cosign".into()),
                ..Default::default()
            },
        );
        signer.sign("img").await.unwrap();
    }

    #[tokio::test]
    async fn attest_invokes_cosign_attest_with_predicate() {
        let mock = MockRunner::new();
        mock.expect(
            |s| {
                s.args.first().map(|a| a == "attest").unwrap_or(false)
                    && s.args.iter().any(|a| a == "--predicate")
            },
            ProcessOutput {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        let signer = CosignSigner::new(
            Arc::new(mock),
            CosignConfig {
                cosign_path: Some("/bin/cosign".into()),
                ..Default::default()
            },
        );
        signer.attest("img", r#"{"_type":"x"}"#).await.unwrap();
    }
}
