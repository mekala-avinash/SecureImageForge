//! Hand-written OpenAPI 3.1 spec for the v1 API. Keeping it static (vs
//! derived) avoids pulling utoipa-style proc-macros for a small surface; the
//! cost of maintaining it manually is one entry per endpoint.

use serde_json::{json, Value};

pub fn spec() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "SecureImage Forge API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Local-mode HTTP API for SecureImage Forge.",
            "license": { "name": "Apache-2.0" }
        },
        "components": {
            "securitySchemes": {
                "bearer": { "type": "http", "scheme": "bearer" }
            }
        },
        "security": [ { "bearer": [] } ],
        "paths": {
            "/healthz": {
                "get": {
                    "summary": "Health check",
                    "security": [],
                    "responses": { "200": { "description": "ok" } }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "Prometheus scrape endpoint",
                    "security": [],
                    "responses": { "200": { "description": "text/plain metrics exposition" } }
                }
            },
            "/v1/builds": {
                "get": { "summary": "List builds", "responses": { "200": { "description": "ok" } } },
                "post": { "summary": "Create a build (operator)", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/builds/{id}":            { "get": { "summary": "Get build summary",   "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/start":      { "post": { "summary": "Dispatch pending build (operator)", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds": {
                "get": { "summary": "List builds in project", "responses": { "200": { "description": "ok" } } },
                "post": { "summary": "Create build in project", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/projects/{project_id}/builds/{id}": { "get": { "summary": "Get build summary in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/start": { "post": { "summary": "Enqueue build in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/scan": { "get": { "summary": "Get scan in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/sbom": { "get": { "summary": "Get sbom in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/log": { "get": { "summary": "Get log in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/log/stream": { "get": { "summary": "Stream log in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/provenance": { "get": { "summary": "Get provenance in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/drift": { "get": { "summary": "Get drift snapshots in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/builds/{id}/cancel": { "post": { "summary": "Cancel queued/running build in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/projects/{project_id}/jobs": { "get": { "summary": "List queue jobs in project", "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/scan":       { "get": { "summary": "Get vulnerability scan", "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/sbom":       { "get": { "summary": "Get CycloneDX SBOM",     "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/log":        { "get": { "summary": "Get build log",          "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/log/stream": { "get": { "summary": "Stream build log via SSE", "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/provenance": { "get": { "summary": "Get in-toto provenance", "responses": { "200": { "description": "ok" } } } },
            "/v1/builds/{id}/drift":      { "get": { "summary": "Get drift snapshots",    "responses": { "200": { "description": "ok" } } } },
            "/v1/audit":                  { "get": { "summary": "Audit events (admin)",   "responses": { "200": { "description": "ok" } } } },
            "/v1/principals": {
                "get":  { "summary": "List principals (admin)", "responses": { "200": { "description": "ok" } } },
                "post": { "summary": "Create principal (admin)", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/principals/{id}": {
                "delete": { "summary": "Revoke principal (admin)", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/auth/config": {
                "get": { "summary": "Read auth mode/OIDC configuration (admin)", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/rbac/bindings": {
                "get": { "summary": "List group-role bindings (admin)", "responses": { "200": { "description": "ok" } } },
                "post": { "summary": "Create/update group-role binding (admin)", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/scopes": {
                "get": { "summary": "List principal scope grants (admin)", "responses": { "200": { "description": "ok" } } },
                "post": { "summary": "Create/update principal scope grant (admin)", "responses": { "200": { "description": "ok" } } }
            }
        }
    })
}
