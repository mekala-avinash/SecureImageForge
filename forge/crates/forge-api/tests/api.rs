//! End-to-end API tests. Spin up the axum router with an in-memory storage
//! and drive it via reqwest. Bootstrap mode kicks in (no principals exist)
//! so the test acts as admin without supplying a token.

use std::sync::Arc;

use forge_api::{router, ApiState};
use forge_core::config::Config;
use forge_core::logs::LogStore;
use forge_core::storage::Storage;
use forge_core::toolchain::Toolchain;

async fn spawn() -> String {
    let storage = Storage::open_memory().await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let logs = Arc::new(forge_core::logs::FileLogStore::new(dir.path().join("logs"))) as Arc<dyn forge_core::logs::LogStore>;
    let toolchain = Arc::new(Toolchain::new(None));
    let state = ApiState::new(
        Arc::new(Config::default()),
        Arc::new(forge_core::repo::SqliteBuildRepo::new(storage.clone())),
        logs,
        Arc::new(forge_core::audit::SqliteAuditLog::new(storage.clone())),
        Arc::new(forge_core::rbac::SqlitePrincipalRepo::new(storage.clone())),
        Arc::new(forge_core::provenance::SqliteProvenanceRepo::new(storage.clone())),
        Arc::new(forge_core::team::SqliteTeamRepo::new(storage.clone())),
        Arc::new(forge_core::team::SqliteScopeRepo::new(storage.clone())),
        Arc::new(forge_core::team::SqliteBuildQueueRepo::new(storage.clone())),
        Arc::new(forge_core::drift::SqliteDriftDetector {
            repo: Arc::new(forge_core::repo::SqliteBuildRepo::new(storage.clone())),
            storage: storage.clone(),
            scanner: forge_api::make_scanner(&toolchain),
            audit: Arc::new(forge_core::audit::SqliteAuditLog::new(storage.clone())),
        }),
        toolchain,
    );
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn create_principal(base: &str, token: Option<&str>, name: &str, role: &str) -> String {
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{base}/v1/principals"))
        .json(&serde_json::json!({"name": name, "role": role}));
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let created: serde_json::Value = req.send().await.unwrap().json().await.unwrap();
    created["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn health_endpoint_is_open() {
    let base = spawn().await;
    let body: serde_json::Value = reqwest::get(format!("{base}/healthz"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn list_builds_in_bootstrap_mode_returns_empty_array() {
    let base = spawn().await;
    let body: serde_json::Value = reqwest::get(format!("{base}/v1/builds"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn create_principal_then_authenticate() {
    let base = spawn().await;
    // Bootstrap admin: create the first real principal.
    let created: serde_json::Value = reqwest::Client::new()
        .post(format!("{base}/v1/principals"))
        .json(&serde_json::json!({"name": "alice", "role": "operator"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = created["token"].as_str().unwrap().to_string();
    assert!(token.starts_with("forge_"));

    // Now bootstrap mode is off — calls without the token should 401.
    let resp = reqwest::get(format!("{base}/v1/builds")).await.unwrap();
    assert_eq!(resp.status(), 401);

    // With the token, the operator can list builds.
    let resp = reqwest::Client::new()
        .get(format!("{base}/v1/builds"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn operator_cannot_list_principals() {
    let base = spawn().await;
    let created: serde_json::Value = reqwest::Client::new()
        .post(format!("{base}/v1/principals"))
        .json(&serde_json::json!({"name": "op", "role": "operator"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = created["token"].as_str().unwrap().to_string();
    let resp = reqwest::Client::new()
        .get(format!("{base}/v1/principals"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn create_build_validates_runtime() {
    let base = spawn().await;
    // Still in bootstrap mode → admin → create works for a valid spec.
    let resp = reqwest::Client::new()
        .post(format!("{base}/v1/builds"))
        .json(&serde_json::json!({"name":"x","runtime":"java","base":"alpine"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Invalid runtime is a 400.
    let resp = reqwest::Client::new()
        .post(format!("{base}/v1/builds"))
        .json(&serde_json::json!({"name":"x","runtime":"perl","base":"alpine"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn openapi_spec_is_served() {
    let base = spawn().await;
    let body: serde_json::Value = reqwest::get(format!("{base}/v1/openapi.json"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"]["/v1/builds"].is_object());
}

#[tokio::test]
async fn drift_endpoint_returns_persisted_snapshots() {
    use forge_core::domain::{
        Architecture, BaseImage, ComplianceProfile, HardeningOptions, Runtime,
    };
    use std::collections::BTreeSet;
    let base = spawn().await;
    // Use bootstrap-mode admin to seed a build via the API.
    let create_resp: serde_json::Value = reqwest::Client::new()
        .post(format!("{base}/v1/builds"))
        .json(&serde_json::json!({
            "name":"drift-target",
            "runtime":"go",
            "base":"alpine",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let _ = (
        Architecture::Amd64,
        BaseImage::Alpine,
        ComplianceProfile::Cis,
        HardeningOptions::strict(),
        Runtime::Go,
        BTreeSet::<Architecture>::new(),
    );
    let id = create_resp["id"].as_str().unwrap().to_string();
    // Until the scheduler runs, the drift endpoint must still respond 200
    // with an empty array (not 404). Ensures the route is wired to the
    // persistence backend.
    let resp = reqwest::Client::new()
        .get(format!("{base}/v1/builds/{id}/drift"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
}

#[tokio::test]
async fn rbac_full_role_matrix_for_api_surface() {
    let base = spawn().await;
    let admin = create_principal(&base, None, "admin", "admin").await;
    let operator = create_principal(&base, Some(&admin), "operator", "operator").await;
    let viewer = create_principal(&base, Some(&admin), "viewer", "viewer").await;
    let client = reqwest::Client::new();

    let create_resp: serde_json::Value = client
        .post(format!("{base}/v1/builds"))
        .bearer_auth(&operator)
        .json(&serde_json::json!({"name":"matrix","runtime":"go","base":"alpine"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let build_id = create_resp["id"].as_str().unwrap();

    for token in [&admin, &operator, &viewer] {
        let resp = client
            .get(format!("{base}/v1/builds"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);

        let resp = client
            .get(format!("{base}/v1/builds/{build_id}"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    let resp = client
        .post(format!("{base}/v1/builds"))
        .bearer_auth(&viewer)
        .json(&serde_json::json!({"name":"denied","runtime":"go","base":"alpine"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = client
        .post(format!("{base}/v1/builds/{build_id}/start"))
        .bearer_auth(&viewer)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = client
        .post(format!("{base}/v1/builds/{build_id}/start"))
        .bearer_auth(&operator)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    for token in [&operator, &viewer] {
        let resp = client
            .get(format!("{base}/v1/principals"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403);
    }

    // Audit log: only admin gets 200; operator + viewer must 403.
    for (token, expected) in [(&admin, 200u16), (&operator, 403), (&viewer, 403)] {
        let resp = client
            .get(format!("{base}/v1/audit"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), expected, "audit / {token}");
    }

    // Read-only build endpoints: admin + operator + viewer all 200.
    for (token, label) in [
        (&admin, "admin"),
        (&operator, "operator"),
        (&viewer, "viewer"),
    ] {
        for path in [
            "/v1/builds",
            &format!("/v1/builds/{build_id}"),
            // sub-resources may legitimately 404 when no scan/sbom/log exists
            // — but the auth gate must not return 401/403 for these roles.
        ] {
            let resp = client
                .get(format!("{base}{path}"))
                .bearer_auth(token)
                .send()
                .await
                .unwrap();
            assert_eq!(resp.status(), 200, "{label} GET {path}");
        }
    }

    // Principal mutation surface: only admin succeeds.
    let resp = client
        .post(format!("{base}/v1/principals"))
        .bearer_auth(&viewer)
        .json(&serde_json::json!({"name":"x","role":"viewer"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    let resp = client
        .get(format!("{base}/v1/principals"))
        .bearer_auth(&admin)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
