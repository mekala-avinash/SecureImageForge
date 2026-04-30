//! Bearer-token authentication middleware. Resolves `Authorization: Bearer <t>`
//! to a `Principal` via the `PrincipalRepo`. When no principals are configured
//! (fresh install), the daemon runs in *bootstrap mode* and a synthetic admin
//! principal is returned so the operator can issue the first real token.

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use chrono::Utc;
use forge_core::config::AuthMode;
use forge_core::rbac::{self, Action, Principal, Role};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::error::ApiError;
use crate::state::ApiState;

pub struct Authenticated(pub Principal);

#[async_trait]
impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    ApiState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let api_state = ApiState::from_ref(state);
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        let local_principal = match token {
            Some(t) => api_state
                .principals
                .authenticate(t)
                .await
                .map_err(ApiError::from)?,
            None => None,
        };

        let bootstrap = api_state
            .principals
            .list()
            .await
            .unwrap_or_default()
            .is_empty();

        match api_state.config.auth.mode {
            AuthMode::Local => {
                if bootstrap {
                    return Ok(Authenticated(bootstrap_admin()));
                }
                Ok(Authenticated(
                    local_principal.ok_or(ApiError::Unauthorized)?,
                ))
            }
            AuthMode::Oidc => {
                if !api_state.config.auth.oidc.enabled {
                    return Err(ApiError::Forbidden);
                }
                let token = token.ok_or(ApiError::Unauthorized)?;
                Ok(Authenticated(
                    resolve_oidc_principal(token, &api_state).await?,
                ))
            }
            AuthMode::Hybrid => {
                if let Some(principal) = local_principal {
                    return Ok(Authenticated(principal));
                }
                if bootstrap {
                    return Ok(Authenticated(bootstrap_admin()));
                }
                if api_state.config.auth.oidc.enabled {
                    let token = token.ok_or(ApiError::Unauthorized)?;
                    return Ok(Authenticated(
                        resolve_oidc_principal(token, &api_state).await?,
                    ));
                }
                Err(ApiError::Unauthorized)
            }
        }
    }
}

pub fn require(principal: &Principal, action: Action) -> Result<(), ApiError> {
    rbac::require(principal, action).map_err(|_| ApiError::Forbidden)
}

fn bootstrap_admin() -> Principal {
    Principal {
        id: "bootstrap".into(),
        name: "bootstrap".into(),
        role: Role::Admin,
        created_at: Utc::now().to_rfc3339(),
    }
}

#[derive(Debug, Deserialize)]
struct OidcClaims {
    sub: String,
    exp: Option<i64>,
    iss: Option<String>,
    aud: Option<serde_json::Value>,
    groups: Option<Vec<String>>,
}

async fn resolve_oidc_principal(token: &str, api_state: &ApiState) -> Result<Principal, ApiError> {
    let claims = verify_and_decode_oidc_claims(token, api_state).await?;
    if let Some(exp) = claims.exp {
        let allowed_skew = api_state.config.auth.oidc.allowed_clock_skew_seconds as i64;
        if Utc::now().timestamp() > exp + allowed_skew {
            return Err(ApiError::Unauthorized);
        }
    }
    if let Some(expected_iss) = &api_state.config.auth.oidc.issuer {
        if claims.iss.as_deref() != Some(expected_iss.as_str()) {
            return Err(ApiError::Unauthorized);
        }
    }
    if let Some(expected_aud) = &api_state.config.auth.oidc.audience {
        let aud_match = match claims.aud {
            Some(serde_json::Value::String(s)) => s == *expected_aud,
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .any(|s| s == expected_aud),
            _ => false,
        };
        if !aud_match {
            return Err(ApiError::Unauthorized);
        }
    }

    let mut role = Role::Viewer;
    if let Some(groups) = claims.groups {
        let bindings = api_state
            .scopes
            .list_group_bindings()
            .await
            .map_err(ApiError::from)?;
        for g in groups {
            if let Some(binding) = bindings.iter().find(|b| b.group_name == g) {
                if binding.role.rank() > role.rank() {
                    role = binding.role;
                }
            }
        }
    }

    Ok(Principal {
        id: claims.sub.clone(),
        name: claims.sub,
        role,
        created_at: Utc::now().to_rfc3339(),
    })
}

#[derive(Debug, Deserialize)]
struct OidcDiscovery {
    jwks_uri: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    kid: String,
    kty: String,
    n: Option<String>,
    e: Option<String>,
    alg: Option<String>,
}

#[derive(Clone)]
struct CachedJwks {
    fetched_at: Instant,
    jwks: Jwks,
}

static JWKS_CACHE: OnceLock<Mutex<HashMap<String, CachedJwks>>> = OnceLock::new();

fn jwks_cache() -> &'static Mutex<HashMap<String, CachedJwks>> {
    JWKS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn verify_and_decode_oidc_claims(
    token: &str,
    api_state: &ApiState,
) -> Result<OidcClaims, ApiError> {
    let issuer = api_state
        .config
        .auth
        .oidc
        .issuer
        .as_deref()
        .ok_or(ApiError::Forbidden)?;

    let header = decode_header(token).map_err(|_| ApiError::Unauthorized)?;
    let kid = header.kid.ok_or(ApiError::Unauthorized)?;
    let alg = header.alg;
    if alg != Algorithm::RS256 {
        return Err(ApiError::Unauthorized);
    }

    let jwks = load_jwks(
        issuer,
        api_state.config.auth.oidc.jwks_refresh_seconds.max(30),
    )
    .await?;
    let key = jwks
        .keys
        .iter()
        .find(|k| k.kid == kid && k.kty == "RSA")
        .ok_or(ApiError::Unauthorized)?;

    if let Some(key_alg) = &key.alg {
        if key_alg != "RS256" {
            return Err(ApiError::Unauthorized);
        }
    }

    let n = key.n.as_deref().ok_or(ApiError::Unauthorized)?;
    let e = key.e.as_deref().ok_or(ApiError::Unauthorized)?;
    let decoding_key =
        DecodingKey::from_rsa_components(n, e).map_err(|_| ApiError::Unauthorized)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.leeway = api_state.config.auth.oidc.allowed_clock_skew_seconds;
    if let Some(aud) = api_state.config.auth.oidc.audience.as_deref() {
        validation.set_audience(&[aud]);
    }
    validation.set_issuer(&[issuer]);

    let data =
        decode::<Value>(token, &decoding_key, &validation).map_err(|_| ApiError::Unauthorized)?;
    serde_json::from_value::<OidcClaims>(data.claims).map_err(|_| ApiError::Unauthorized)
}

async fn load_jwks(issuer: &str, refresh_seconds: u64) -> Result<Jwks, ApiError> {
    let cached = {
        let guard = jwks_cache()
            .lock()
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("jwks cache poisoned")))?;
        guard.get(issuer).cloned()
    };

    if let Some(cached) = cached {
        if cached.fetched_at.elapsed() < Duration::from_secs(refresh_seconds) {
            return Ok(cached.jwks);
        }
    }

    let issuer = issuer.trim_end_matches('/');
    let discovery_url = format!("{issuer}/.well-known/openid-configuration");
    let discovery: OidcDiscovery = reqwest::Client::new()
        .get(discovery_url)
        .send()
        .await
        .map_err(|_| ApiError::Unauthorized)?
        .error_for_status()
        .map_err(|_| ApiError::Unauthorized)?
        .json()
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    let jwks: Jwks = reqwest::Client::new()
        .get(discovery.jwks_uri)
        .send()
        .await
        .map_err(|_| ApiError::Unauthorized)?
        .error_for_status()
        .map_err(|_| ApiError::Unauthorized)?
        .json()
        .await
        .map_err(|_| ApiError::Unauthorized)?;

    {
        let mut guard = jwks_cache()
            .lock()
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("jwks cache poisoned")))?;
        guard.insert(
            issuer.to_string(),
            CachedJwks {
                fetched_at: Instant::now(),
                jwks: jwks.clone(),
            },
        );
    }
    Ok(jwks)
}
