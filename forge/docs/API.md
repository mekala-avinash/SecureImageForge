# forge-api

Local-mode HTTP API for SecureImage Forge. Backs the `forge serve` daemon and
the same set of build/scan/sbom/audit/principal operations that the CLI and
desktop app expose to operators.

## Quick start

```bash
# Start the daemon — listens on 127.0.0.1:7878 by default.
forge serve --addr 127.0.0.1:7878
# In another shell, hit the open endpoint:
curl http://127.0.0.1:7878/healthz
# {"status":"ok","version":"0.1.0"}
```

The first time the daemon starts there are no principals configured; the
service runs in **bootstrap mode** and treats unauthenticated requests as a
synthetic admin so you can issue a real token:

```bash
curl -s http://127.0.0.1:7878/v1/principals \
  -X POST \
  -H 'content-type: application/json' \
  -d '{"name":"alice","role":"admin"}'
# {"principal":{"id":"…","name":"alice","role":"admin",…},"token":"forge_…"}
```

After the first principal exists, every subsequent call requires a bearer
token:

```bash
TOKEN=forge_xxxxxxxxxxxxxxxxxxxxxxxx
curl -s -H "authorization: Bearer $TOKEN" http://127.0.0.1:7878/v1/builds
```

## Endpoint summary

| Method | Path | Min role | Notes |
|---|---|---|---|
| GET | `/healthz` | (open) | liveness probe |
| GET | `/v1/openapi.json` | (open) | OpenAPI 3.1 spec |
| GET | `/v1/builds` | viewer | recent 200 builds |
| POST | `/v1/builds` | operator | create a build |
| GET | `/v1/builds/{id}` | viewer | summary |
| POST | `/v1/builds/{id}/start` | operator | dispatch a pending build through the API daemon |
| GET | `/v1/builds/{id}/scan` | viewer | normalized vulnerabilities |
| GET | `/v1/builds/{id}/sbom` | viewer | CycloneDX SBOM |
| GET | `/v1/builds/{id}/log` | viewer | raw build log |
| GET | `/v1/builds/{id}/provenance` | viewer | in-toto SLSA Statement |
| GET | `/v1/builds/{id}/drift` | viewer | drift snapshots |
| POST | `/v1/builds/{id}/cancel` | operator | cancel a running build |
| GET | `/v1/projects/{project_id}/builds` | viewer | builds scoped to project |
| POST | `/v1/projects/{project_id}/builds` | operator | create a build in project |
| GET | `/v1/projects/{project_id}/builds/{id}/...` | viewer | (all build subresources) |
| POST | `/v1/projects/{project_id}/builds/{id}/cancel` | operator | cancel a running build in project |
| GET | `/v1/projects/{project_id}/jobs` | viewer | active/pending jobs in project |
| GET | `/v1/audit` | admin | last 200 audit events |
| GET | `/v1/principals` | admin | list principals |
| POST | `/v1/principals` | admin | create + issue token |
| DELETE | `/v1/principals/{id}` | admin | revoke |
| GET | `/v1/auth/config` | (open) | auth configuration (e.g. OIDC details) |
| GET | `/v1/rbac/bindings` | admin | list RBAC bindings |
| POST | `/v1/rbac/bindings` | admin | create RBAC binding |
| GET | `/v1/scopes` | admin | list scope grants |
| POST | `/v1/scopes` | admin | create scope grant |

## RBAC

Three roles, ordered admin > operator > viewer. `forge-core::rbac::Action`
maps each call to the minimum role; an `Authenticated` extractor in
`forge-api::auth` runs that check before the handler executes. See
[`crates/forge-core/src/rbac.rs`](../crates/forge-core/src/rbac.rs).

Tokens are stored as sha256 hashes; the plaintext is shown only once at
creation time. Manage them through the API or:

```bash
forge principals create --name alice --role operator
forge principals list
forge principals revoke <id>
```

## Audit log

Every privileged write (build creation, principal mutation, drift rescan)
appends a row to the `audit_events` table. `GET /v1/audit` exposes the last
200 entries to admins. Schema in
[`crates/forge-core/migrations/0003_rbac_and_drift.sql`](../crates/forge-core/migrations/0003_rbac_and_drift.sql).

## Drift detection

`forge_core::drift::DriftDetector::rescan_one` re-scans a previously-built
image with the configured scanner, diffs critical/high CVE IDs against the
original baseline, and persists a `drift_snapshots` row with the deltas. A
`run_scheduler` helper drives this on an interval when
`[drift].scheduler_enabled = true`; set `[drift].interval_seconds` or
`FORGE_DRIFT_INTERVAL_SECONDS` to tune the cadence.

## SLSA provenance

After a successful build, the orchestrator calls
`forge_core::provenance::build_statement` to produce an in-toto Statement
following the SLSA v1 predicate, then signs and stores it. `cosign attest`
pushes the same blob as an OCI attestation alongside the artifact.

## Registry auth

Three modes, in precedence:

1. Explicit `username` + `password` in `[registry.auth]`.
2. `credential_helper = "osxkeychain"` (or any docker-credential-* binary).
3. `FORGE_REGISTRY_TOKEN` env var.

The orchestrator surfaces these as env vars to `buildctl` (and `cosign`)
when pushing to a registry — see `forge-core::registry::auth_env`.

## Multi-scanner gate

The orchestrator wraps Trivy + Grype in `MergedScanner`: findings are merged
on `(id, package)` with the higher severity winning. A CVE that one scanner
misses but the other catches still gates the build.
