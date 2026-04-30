# SecureImage Forge — Architecture (Phase 0)

## Workspace layout

```
forge/
├── Cargo.toml              # workspace root
├── rust-toolchain.toml     # pinned 1.83.0 + cross-compile targets
├── deny.toml               # license + advisory policy (cargo-deny)
├── .cargo/config.toml      # cargo aliases
├── crates/
│   ├── forge-core/         # domain model, storage, tooling traits
│   ├── forge-cli/          # `forge` binary (clap)
│   ├── forge-api/          # axum HTTP API (serve mode)
│   └── forge-desktop/      # Dioxus desktop binary
├── xtask/                  # repo automation (license audit, dist, bundle)
└── docs/                   # design notes
```

## Crate responsibilities

| Crate | Owns |
|---|---|
| `forge-core` | `BuildSpec`, `BuildRecord`, `Vulnerability`, `Sbom`; SQLite storage; trait abstractions over BuildKit / Trivy / Syft / Cosign / OPA |
| `forge-cli` | `forge` binary — single static executable wrapping `forge-core` |
| `forge-api` | Optional `forge serve` HTTP daemon for CI integrations |
| `forge-desktop` | Dioxus-based native desktop wrapping `forge-core` directly (no HTTP indirection) |
| `xtask` | `cargo xtask license-audit`, `cargo xtask bundle-buildkit`, `cargo xtask dist` |

## Data flow

```
                ┌──────────────────────┐
 user ───►──┬──►│  forge-cli (binary)  │──┐
            │   └──────────────────────┘  │
            │   ┌──────────────────────┐  │   ┌──────────────────┐
            ├──►│ forge-desktop (UI)   │──┼──►│   forge-core     │
            │   └──────────────────────┘  │   │  (domain + DB)   │
            │   ┌──────────────────────┐  │   └────────┬─────────┘
            └──►│  forge-api (axum)    │──┘            │
                └──────────────────────┘               │
                                                       ▼
                              ┌────────────────────────────────────┐
                              │ External Apache-2.0 tools (CLI):   │
                              │  buildctl/buildkitd · trivy        │
                              │  syft · cosign · opa · in-toto     │
                              └────────────────────────────────────┘
```

`forge-desktop` calls `forge-core` directly in-process (no IPC). `forge-api` is optional and is what CI/automation consume.

## Storage

SQLite via `sqlx`. Single-user desktop default; team/server mode (Postgres) is a Phase 4 stretch goal.

Schema seeded by migrations in `crates/forge-core/migrations/`.

## License posture

`deny.toml` rejects GPL / AGPL / LGPL / SSPL / BUSL. Allowed: Apache-2.0, MIT, BSD-2/3, ISC, MPL-2.0, Zlib, Unicode, CC0.

All bundled external tools are Apache 2.0:

| Tool | Purpose | License |
|---|---|---|
| BuildKit (rootless `buildkitd` + `buildctl`) | OCI build engine | Apache 2.0 |
| Trivy | Vulnerability scanning | Apache 2.0 |
| Syft | SBOM (CycloneDX/SPDX) | Apache 2.0 |
| Grype | Optional second scanner | Apache 2.0 |
| Cosign / Sigstore | Signing + attestation | Apache 2.0 |
| OPA | Policy evaluation (Rego) | Apache 2.0 |
| in-toto | SLSA provenance | Apache 2.0 |

`xtask bundle-buildkit` (Phase 1) downloads, verifies checksum, and stages binaries for desktop bundling.

## Phase plan

| Phase | Scope | Status |
|---|---|---|
| 0 — Foundation | Workspace, crate skeletons, CI, license gate | ✅ landed |
| 1 — Core engine | BuildKit/Trivy/Syft/Cosign/OPA adapters; SQLite migrations live; orchestrator | ✅ landed |
| 2 — CLI + bundled toolchain | Config file, persisted logs, SARIF, registry push/target, `forge doctor`, completions, `xtask bundle-buildkit` (Apache-2.0 pinned versions), CLI integration tests | ✅ landed |
| 3 — Desktop | Dioxus shell with Dashboard / Builds / New build / Build detail / Doctor views; in-process orchestrator dispatch (no HTTP); auto-refresh polling; Klein-blue control-room theme | ✅ landed |
| 3.5 — Desktop polish | Cosign-signed update manifest + in-app `Settings → Check for updates`, tray menu (show / update / quit), `xtask dist` with cosign sign-blob, macOS notarize.sh + entitlements, Windows NSIS + Authenticode template, Linux `.deb` via fpm, tag-driven release workflow | ✅ landed |
| 4 — Enterprise | `forge serve` axum daemon + OpenAPI, RBAC roles + bearer-token principals, append-only audit log, in-toto SLSA L3 provenance, registry auth (basic / token / cred-helper), Grype scanner + `MergedScanner`, drift detection scheduler, `forge principals` admin CLI | ✅ landed |
| 4.5 — Enterprise polish | Orchestrator uses MergedScanner, cosign attests provenance, drift scheduler config surface, `/v1/builds/{id}/start` API dispatch, RBAC role matrix tests | ✅ landed |
| 5 — QA / Release | `xtask coverage` with 80% floor (cargo-llvm-cov), proptest invariants for Dockerfile generator, full orchestrator e2e test (MergedScanner + provenance), expanded RBAC + drift API tests, `cargo auditable` builds + cyclonedx SBOMs + cosign attest in release.yml, CodeQL + dependabot, coverage CI workflow | ✅ landed |
| 6 — Operations & integrations | Storage backend discriminator (sqlite + postgres URL parsing), Prometheus `/metrics` + axum middleware, OTLP scaffold, SSE log streaming, HMAC-signed webhook event sink, `forge-sdk` typed Rust client | ✅ landed |
| 6.5 — Postgres team mode | Postgres connection pool + migrations + read-mirror `PgBuildRepo` (gated on `pg` feature), full OTLP HTTP/protobuf exporter via `tracing-opentelemetry` (gated on `otlp` feature), persistent webhook retry queue with exponential backoff worker, SDK `RetryPolicy` middleware | ✅ landed |
| 7 — Postgres write parity | Per-table Any-driver adapters for scans/sboms/drift/provenance, write-path tests against a live Postgres, scheduler config surface for the webhook worker | next |
