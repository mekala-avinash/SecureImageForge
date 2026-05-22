# Hardened Runtime Images

Production-grade Dockerfiles. Each is multi-arch (`amd64` + `arm64`), runs as UID 65532, has a read-only filesystem in the runtime stage, and produces images ≤ 100 MB.

## Layout

```
images/
├── runtimes/
│   ├── python/Dockerfile      # FastAPI/Flask-ready, ≤ 75 MB
│   ├── go/Dockerfile          # static binary on chainguard/static, ≤ 15 MB
│   ├── nodejs/Dockerfile      # Express/Fastify-ready, ≤ 80 MB
│   └── java/                  # (use docs/runtime-images/runtimes/Dockerfile.java.template)
├── buildkit/
│   └── build.sh               # reproducible, signed, attested multi-arch build
├── base/                      # platform-internal Wolfi base hardening (placeholder)
└── tools/                     # CI/Ops tools image (kubectl/helm/cosign/syft/grype/trivy/jq/yq)
                               # (use docs/runtime-images/tooling/Dockerfile.tooling-multi.template)
```

## Build & sign (locally)

```bash
export IMAGE_REPO=registry.acme.io/orders/api
export IMAGE_TAG=v1.2.3
export COSIGN_EXPERIMENTAL=1
./images/buildkit/build.sh
```

Output:
- Image pushed to `${IMAGE_REPO}:${IMAGE_TAG}` + immutable digest.
- Cosign signature in Fulcio + Rekor.
- SBOM (SPDX + CycloneDX) attached as OCI artifacts.
- SLSA L3 provenance attached.

## Security guarantees

| Property | How |
|---|---|
| No shell in runtime | Wolfi/distroless base |
| No package manager in runtime | Separate builder stage |
| Non-root | `USER 65532:65532` |
| Read-only FS at runtime | `readOnlyRootFilesystem: true` in Helm library |
| Reproducible | `SOURCE_DATE_EPOCH` + `rewrite-timestamp=true` |
| Multi-arch | `buildx --platform linux/amd64,linux/arm64` |
| CVE SLA | Nightly rebuild of bases; weekly rebuild of runtimes |
| Signed | Cosign keyless (Fulcio OIDC → Rekor) |
| Attested | SBOM + SLSA L3 provenance via Cosign attest |

## CI integration

The reusable GitHub Actions workflow at `ci/github-actions/reusable-build.yml` invokes `build.sh` automatically. Service repos add a 6-line `.github/workflows/build.yml` and inherit everything.

## Operational guidance

- **Image promotion** = digest reference in a GitOps PR; no `:latest`, no mutable tags in prod overlays.
- **CVE response** is owned by the Dependency Management Agent (auto-PR within SLA: Critical 24h, High 7d).
- **Air-gapped** use: `cosign save ${IMAGE_REPO}@${DIGEST}` produces a bundle for transfer via approved diode.
