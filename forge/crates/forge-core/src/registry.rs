//! Registry credential resolution. The orchestrator surfaces these as env
//! vars to `buildctl` (which inherits them to push) and to `cosign`.
//!
//! Three modes, in order of precedence:
//!   1. Explicit `username` + `password` in the config.
//!   2. A registry credential helper (`docker-credential-<name>`).
//!   3. A bearer token in `FORGE_REGISTRY_TOKEN`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::process::{ProcessRunner, ProcessSpec};
use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryAuth {
    pub registry: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub credential_helper: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAuth {
    pub username: String,
    pub password: String,
    pub registry: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CredentialHelperResult {
    #[serde(rename = "Username")]
    username: String,
    #[serde(rename = "Secret")]
    secret: String,
}

pub async fn resolve(
    runner: &dyn ProcessRunner,
    auth: &RegistryAuth,
) -> Result<Option<ResolvedAuth>> {
    let registry = match &auth.registry {
        Some(r) => r.clone(),
        None => return Ok(None),
    };
    if let (Some(u), Some(p)) = (&auth.username, &auth.password) {
        return Ok(Some(ResolvedAuth {
            username: u.clone(),
            password: p.clone(),
            registry,
        }));
    }
    if let Some(helper) = &auth.credential_helper {
        let bin = format!("docker-credential-{helper}");
        let spec = ProcessSpec::new(bin).arg("get");
        let mut spec = spec;
        // helper expects the registry on stdin; we model that with an env
        // shim because our ProcessRunner trait is request/response.
        spec = spec.env("FORGE_REGISTRY_INPUT", &registry);
        let out = runner.run(spec).await?;
        if out.status == 0 {
            let parsed: CredentialHelperResult = serde_json::from_str(&out.stdout)?;
            return Ok(Some(ResolvedAuth {
                username: parsed.username,
                password: parsed.secret,
                registry,
            }));
        }
    }
    if let Ok(token) = std::env::var("FORGE_REGISTRY_TOKEN") {
        return Ok(Some(ResolvedAuth {
            username: "<token>".into(),
            password: token,
            registry,
        }));
    }
    Ok(None)
}

/// Render env vars buildctl + cosign understand. Used by the orchestrator
/// when launching subprocess adapters.
pub fn auth_env(auth: &ResolvedAuth) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("DOCKER_AUTH_USERNAME".into(), auth.username.clone());
    env.insert("DOCKER_AUTH_PASSWORD".into(), auth.password.clone());
    env.insert("DOCKER_AUTH_REGISTRY".into(), auth.registry.clone());
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{MockRunner, ProcessOutput};

    #[tokio::test]
    async fn explicit_credentials_win() {
        let runner = MockRunner::new();
        let auth = RegistryAuth {
            registry: Some("ghcr.io".into()),
            username: Some("alice".into()),
            password: Some("secret".into()),
            credential_helper: None,
        };
        let r = resolve(&runner, &auth).await.unwrap().unwrap();
        assert_eq!(r.username, "alice");
        assert_eq!(r.password, "secret");
    }

    #[tokio::test]
    async fn no_registry_returns_none() {
        let runner = MockRunner::new();
        let auth = RegistryAuth::default();
        assert!(resolve(&runner, &auth).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn helper_output_parsed() {
        let runner = MockRunner::new();
        runner.expect(
            |s| s.program == "docker-credential-osxkeychain",
            ProcessOutput {
                status: 0,
                stdout: r#"{"Username":"u","Secret":"p"}"#.into(),
                stderr: String::new(),
            },
        );
        let auth = RegistryAuth {
            registry: Some("ghcr.io".into()),
            credential_helper: Some("osxkeychain".into()),
            ..Default::default()
        };
        let r = resolve(&runner, &auth).await.unwrap().unwrap();
        assert_eq!(r.username, "u");
        assert_eq!(r.password, "p");
    }
}
