# Hardened Runtime Image Standards

This directory contains hardened, distroless / Wolfi-class Dockerfile templates and BuildKit strategies for all supported language runtimes and tooling images.

## Layout

```
runtime-images/
├── runtimes/                     # Application runtimes
│   ├── Dockerfile.nodejs.template
│   ├── Dockerfile.python.template
│   ├── Dockerfile.go.template
│   ├── Dockerfile.rust.template
│   ├── Dockerfile.java.template
│   ├── Dockerfile.ruby.template
│   ├── Dockerfile.php.template
│   └── Dockerfile.dotnet.template
├── tooling/                      # CI/Ops tooling images
│   └── Dockerfile.tooling-multi.template
└── templates/                    # Build helpers
    ├── buildkit-build.sh
    ├── cosign-sign.sh
    ├── sbom-attest.sh
    └── slsa-provenance.sh
```

## Universal Standards (all images)

| Standard | Value |
|---|---|
| Base | Wolfi (`cgr.dev/chainguard/wolfi-base`) or Wolfi-derived (`*-static`, `*-glibc`) |
| Default user | `nonroot` (UID/GID 65532) |
| Root filesystem | read-only at runtime |
| Capabilities | drop ALL; add explicitly when required |
| Shell | not present in runtime stage |
| Package manager | not present in runtime stage |
| Multi-arch | `linux/amd64`, `linux/arm64` (mandatory) |
| Healthcheck | HTTP `/healthz` or process-level for non-HTTP |
| SBOM | Syft → SPDX 2.3, attached via Cosign |
| Signing | Cosign keyless (Fulcio OIDC) |
| Provenance | SLSA L3 in-toto attestation |
| Tags | Immutable digests in prod manifests; semver tags for humans |
| Reproducibility | `SOURCE_DATE_EPOCH`, deterministic builds |

## CVE SLA

| Severity | Patch SLA |
|---|---|
| Critical | 24 hours |
| High | 7 days |
| Medium | 30 days |
| Low | 90 days |

Bases rebuild nightly; runtimes rebuild weekly; CVE scans run hourly via Trivy Operator. The **Dependency Management Agent** auto-creates PRs for any base-image CVE that exceeds SLA.

## Build Workflow

```
source ──▶ BuildKit (hermetic) ──▶ Syft SBOM ──▶ Trivy scan ──▶ Cosign sign ──▶ Rekor log ──▶ registry
                                                       │
                                                  fail on Critical
```

Each runtime template follows the pattern:

```
FROM ${BASE_BUILDER}@sha256:...   AS builder   # has compilers + package manager
... compile / install ...
FROM ${BASE_RUNTIME}@sha256:...   AS runtime   # distroless / Wolfi-static
USER 65532:65532
COPY --from=builder --chown=65532:65532 /app/dist /app
ENTRYPOINT ["/app/server"]
```

See individual template files for full implementations.
