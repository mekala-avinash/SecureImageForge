# Reusable CI

Two equivalent reusable workflows: GitHub Actions + GitLab CI. Both run the same paved-road sequence: pre-flight → build → scan → sign-attest → GitOps PR.

## Architecture rationale

- **Reusable, not copy-paste.** Service repos call a 6-line wrapper; the platform owns the implementation.
- **OIDC throughout.** No long-lived registry/cloud secrets in service repos.
- **Single source of truth for security gates.** A platform-side change rolls to all services on next build.
- **Vendor-agnostic.** GitHub and GitLab paths produce equivalent artifacts.

## Files

```
ci/
├── github-actions/
│   └── reusable-build.yml          # called via `uses: acme/platform/.github/workflows/reusable-build.yml@main`
├── gitlab-ci/
│   └── build.gitlab-ci.yml         # included via `include: project: acme/platform, file: …`
└── shared/
    └── (any future shared scripts)
```

## Service-side usage

### GitHub

```yaml
# .github/workflows/build.yml in a service repo
name: build
on:
  pull_request:
  push: { branches: [main] }
jobs:
  build:
    uses: acme/platform/.github/workflows/reusable-build.yml@main
    with:
      service-name: orders-api
      runtime: python
      gitops-env: staging      # or "dev" / "prod"
    secrets:
      gitops-token: ${{ secrets.GITOPS_TOKEN }}
```

### GitLab

```yaml
# .gitlab-ci.yml in a service repo
include:
  - project: acme/platform
    file: /ci/gitlab-ci/build.gitlab-ci.yml
    ref: main
variables:
  SERVICE_NAME: orders-api
  RUNTIME: python
```

## What the pipeline produces

1. Multi-arch image (`linux/amd64` + `linux/arm64`) pushed to registry, immutable digest.
2. Cosign keyless signature in Fulcio + entry in Rekor transparency log.
3. SBOM (SPDX + CycloneDX) attached as OCI artifacts.
4. SLSA L3 in-toto provenance attached.
5. Trivy SARIF uploaded to GitHub Security tab.
6. GitOps PR opened against `acme/gitops` (or GitLab equivalent) bumping the digest in the chosen environment overlay.

## Security gates that block a merge

| Gate | Tool | Threshold |
|---|---|---|
| Secret scan | gitleaks | any HIGH+ |
| SAST | Semgrep (OWASP Top 10) | any HIGH+ |
| SCA | osv-scanner | any vuln |
| Dockerfile lint | hadolint | warn only |
| Image scan | Trivy + Grype | CRITICAL/HIGH |
| Signed commits | gitsign | warn (branch protection enforces) |

## Operational guidance

- **Pinning vs floating.** Pin `@main` for early adopters; switch to `@v1` once 1.0 ships.
- **Skipping a gate** requires a documented waiver (see `/app/docs/phase-1/01-hardened-runtimes/waiver-process.md`).
- **Adding a new gate** flows automatically to every service on the next push.
- **Build cache** is shared per service via the registry's `:buildcache` tag.
