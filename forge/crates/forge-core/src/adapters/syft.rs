//! Syft adapter (Apache-2.0). Generates a CycloneDX SBOM by default.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::Sbom;
use crate::process::{resolve_tool, ProcessRunner, ProcessSpec};
use crate::tooling::SbomGenerator;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct SyftConfig {
    pub syft_path: Option<PathBuf>,
    pub bundled_prefix: Option<PathBuf>,
    /// "cyclonedx-json" (default) | "spdx-json".
    pub format: String,
}

impl Default for SyftConfig {
    fn default() -> Self {
        Self {
            syft_path: None,
            bundled_prefix: None,
            format: "cyclonedx-json".into(),
        }
    }
}

pub struct SyftSbomGenerator {
    runner: Arc<dyn ProcessRunner>,
    config: SyftConfig,
}

impl SyftSbomGenerator {
    pub fn new(runner: Arc<dyn ProcessRunner>, config: SyftConfig) -> Self {
        Self { runner, config }
    }

    fn syft(&self) -> Result<PathBuf> {
        if let Some(p) = &self.config.syft_path {
            return Ok(p.clone());
        }
        resolve_tool("syft", self.config.bundled_prefix.as_deref())
    }
}

#[async_trait]
impl SbomGenerator for SyftSbomGenerator {
    async fn generate(&self, image_ref: &str) -> Result<Sbom> {
        let syft = self.syft()?;
        let spec = ProcessSpec::new(syft.to_string_lossy().to_string())
            .arg(image_ref)
            .arg("-o")
            .arg(&self.config.format)
            .arg("--quiet");
        let out = self.runner.run(spec).await?;
        if out.status != 0 {
            return Err(Error::ToolFailure {
                tool: "syft".into(),
                code: out.status,
                stderr: out.stderr,
            });
        }
        let document: serde_json::Value = serde_json::from_str(&out.stdout)?;
        let format = if self.config.format.starts_with("cyclonedx") {
            "cyclonedx"
        } else {
            "spdx"
        };
        Ok(Sbom {
            format: format.into(),
            document,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};

    #[tokio::test]
    async fn generates_cyclonedx() {
        let mock = MockRunner::new();
        mock.expect(
            |s| s.program.ends_with("syft"),
            ProcessOutput {
                status: 0,
                stdout: r#"{"bomFormat":"CycloneDX","specVersion":"1.5","components":[]}"#.into(),
                stderr: String::new(),
            },
        );
        let g = SyftSbomGenerator::new(
            Arc::new(mock),
            SyftConfig {
                syft_path: Some("/bin/syft".into()),
                ..Default::default()
            },
        );
        let sbom = g.generate("img:tag").await.unwrap();
        assert_eq!(sbom.format, "cyclonedx");
        assert_eq!(sbom.document["bomFormat"], "CycloneDX");
    }
}
