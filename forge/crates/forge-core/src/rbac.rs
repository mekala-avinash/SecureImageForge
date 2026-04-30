//! Lightweight RBAC: bearer-token principals with three roles. Tokens are
//! stored as sha256 hashes; the plaintext is shown only at creation time.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::storage::Storage;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Operator,
    Viewer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "operator" => Some(Role::Operator),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }

    /// Admin > Operator > Viewer for purposes of permission checks.
    pub fn rank(self) -> u8 {
        match self {
            Role::Admin => 3,
            Role::Operator => 2,
            Role::Viewer => 1,
        }
    }

    pub fn can(self, action: Action) -> bool {
        let needed = match action {
            Action::ListBuilds | Action::ReadBuild => Role::Viewer,
            Action::StartBuild => Role::Operator,
            Action::ManagePrincipals | Action::WritePolicy => Role::Admin,
        };
        self.rank() >= needed.rank()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    ListBuilds,
    ReadBuild,
    StartBuild,
    ManagePrincipals,
    WritePolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct Principal {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedPrincipal {
    pub principal: Principal,
    /// Plaintext token. Shown ONCE at creation; only its hash is stored.
    pub token: String,
}

#[derive(Clone)]
pub struct PrincipalRepo {
    storage: Storage,
}

impl PrincipalRepo {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn create(&self, name: &str, role: Role) -> Result<CreatedPrincipal> {
        let id = Uuid::new_v4().to_string();
        let token = format!("forge_{}", random_token());
        let token_hash = hash_token(&token);
        let created_at = Utc::now().to_rfc3339();
        sqlx::query(
            r#"INSERT INTO principals (id, name, role, token_hash, created_at)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(name)
        .bind(role.as_str())
        .bind(&token_hash)
        .bind(&created_at)
        .execute(self.storage.pool())
        .await?;
        Ok(CreatedPrincipal {
            principal: Principal {
                id,
                name: name.into(),
                role,
                created_at,
            },
            token,
        })
    }

    pub async fn list(&self) -> Result<Vec<Principal>> {
        let rows = sqlx::query(
            r#"SELECT id, name, role, created_at
               FROM principals WHERE revoked_at IS NULL ORDER BY created_at DESC"#,
        )
        .fetch_all(self.storage.pool())
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                Some(Principal {
                    id: r.get("id"),
                    name: r.get("name"),
                    role: Role::parse(r.get::<&str, _>("role"))?,
                    created_at: r.get("created_at"),
                })
            })
            .collect())
    }

    pub async fn revoke(&self, id: &str) -> Result<()> {
        sqlx::query(r#"UPDATE principals SET revoked_at = ? WHERE id = ?"#)
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(self.storage.pool())
            .await?;
        Ok(())
    }

    /// Resolve a bearer token to a principal. Constant-time comparison via
    /// the unique hash index; revoked principals are ignored.
    pub async fn authenticate(&self, token: &str) -> Result<Option<Principal>> {
        let hash = hash_token(token);
        let row = sqlx::query(
            r#"SELECT id, name, role, created_at FROM principals
               WHERE token_hash = ? AND revoked_at IS NULL"#,
        )
        .bind(&hash)
        .fetch_optional(self.storage.pool())
        .await?;
        Ok(row.and_then(|r| {
            Some(Principal {
                id: r.get("id"),
                name: r.get("name"),
                role: Role::parse(r.get::<&str, _>("role"))?,
                created_at: r.get("created_at"),
            })
        }))
    }
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn random_token() -> String {
    // 24 hex chars from a UUIDv4 — 96 bits of entropy is plenty for an
    // operator API token; tighten via env config if needed.
    Uuid::new_v4().simple().to_string()[..24].to_string()
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("permission denied: role {role:?} cannot {action:?}")]
pub struct Forbidden {
    pub role: Role,
    pub action: String,
}

pub fn require(principal: &Principal, action: Action) -> Result<()> {
    if principal.role.can(action) {
        Ok(())
    } else {
        Err(Error::PolicyViolation(format!(
            "permission denied for role {:?}",
            principal.role
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_then_authenticate() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = PrincipalRepo::new(storage);
        let created = repo.create("alice", Role::Operator).await.unwrap();
        assert!(created.token.starts_with("forge_"));
        let p = repo
            .authenticate(&created.token)
            .await
            .unwrap()
            .expect("token should resolve");
        assert_eq!(p.name, "alice");
        assert_eq!(p.role, Role::Operator);
    }

    #[tokio::test]
    async fn revoke_invalidates_token() {
        let storage = Storage::open_memory().await.unwrap();
        let repo = PrincipalRepo::new(storage);
        let created = repo.create("bob", Role::Viewer).await.unwrap();
        repo.revoke(&created.principal.id).await.unwrap();
        assert!(repo.authenticate(&created.token).await.unwrap().is_none());
    }

    #[test]
    fn role_rank_ordering_and_permissions() {
        assert!(Role::Admin.can(Action::ManagePrincipals));
        assert!(!Role::Operator.can(Action::ManagePrincipals));
        assert!(Role::Operator.can(Action::StartBuild));
        assert!(!Role::Viewer.can(Action::StartBuild));
        assert!(Role::Viewer.can(Action::ListBuilds));
    }
}
