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

| Phase | Scope |
|---|---|
| **0 — Foundation** *(this PR)* | Workspace, crate skeletons, CI, license gate |
| 1 — Core engine | BuildKit/Trivy/Syft/Cosign/OPA adapters; SQLite migrations live |
| 2 — CLI | All subcommands operational with json/sarif output, completions |
| 3 — Desktop | Full Dioxus UI parity with old React dashboard; auto-updater; signed installers |
| 4 — Enterprise | `forge serve`, multi-arch, registries, SLSA L3, RBAC, audit log, drift |
| 5 — QA / Release | Integration tests, E2E, ≥80% coverage, reproducible release pipeline |
