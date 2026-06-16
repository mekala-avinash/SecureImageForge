# Platform Quickstart

> Get from "I want a new service" to "production traffic" in **15 minutes**, on the paved road.

## 0. Prereqs

Local toolchain (one-time):

```bash
brew install kubectl helm kind kustomize cosign syft trivy hadolint kubeconform
pipx install pavedroad-cli                                   # or pip install -e developer-experience/pavedroad-cli
gh auth login                                                # for repo creation
```

## 1. Scaffold a service

```bash
pavedroad new service \
  --name   orders-api \
  --language python \
  --team   payments
```

Supported `--language`: `python` · `go` · `nodejs` · `java`.

What you get:
```
orders-api/
├── Dockerfile                       # hardened, multi-stage, distroless runtime
├── Makefile                         # standard targets: bootstrap, run, test, image-build, sbom
├── README.md
├── catalog-info.yaml                # Backstage component
├── helm/
│   ├── Chart.yaml                   # depends on acme-platform-lib v1
│   ├── values.yaml                  # SLOs, OTel, image repo
│   └── templates/all.yaml           # includes every paved-road primitive
├── src/                             # service code with /healthz, /ready, /metrics
├── tests/                           # smoke tests for endpoints
├── .github/workflows/build.yml      # uses platform/reusable-build.yml
├── .gitlab-ci.yml                   # uses platform/build.gitlab-ci.yml
└── azure-pipelines.yml              # extends platform/ci/azure-devops/build.yml
```

## 2. Run locally

```bash
cd orders-api
make bootstrap
make run               # http://localhost:8080
curl localhost:8080/healthz
curl localhost:8080/metrics | head
make test
```

## 3. Push & let CI take over

```bash
git init && git add . && git commit -sm "feat: bootstrap orders-api on paved road"
gh repo create acme/orders-api --private --source . --push
```

CI now:

| Stage          | Tools                                  | Output                       |
|----------------|----------------------------------------|------------------------------|
| Pre-flight     | gitleaks · semgrep · osv-scanner · hadolint | block on findings        |
| Build          | docker buildx + provenance + SBOM      | multi-arch image @ digest    |
| Scan           | trivy + grype                          | block on HIGH/CRITICAL CVE   |
| Sign + attest  | cosign keyless + Rekor + SPDX SBOM     | signed image + attestation   |
| SLSA L3        | slsa-github-generator                  | SLSA provenance              |
| Helm validate  | helm template + kubeconform + kyverno  | block on schema/policy fail  |
| GitOps         | yq + peter-evans/create-pull-request   | PR to acme/gitops            |

## 4. Promote through environments

ArgoCD's ApplicationSet (`gitops/applicationsets/services.yaml`) detects the new
service automatically and creates three Applications: `orders-api-dev`,
`orders-api-staging`, `orders-api-prod`.

Promote manually or via the CLI:

```bash
pavedroad sync --service orders-api --env dev
pavedroad status --service orders-api --env dev

# Watch until Synced + Healthy
pavedroad watch --service orders-api --env dev

# Promote to staging (opens GitOps PR)
ACME_LIVE=1 pavedroad promote --service orders-api --to staging --digest sha256:...
```

Production switches the workload from `Deployment` → `Argo Rollout` with the
canary `AnalysisTemplate` defined in
`apps/_template/overlays/prod/rollout.yaml` (10 % → analysis → 25 % → 50 % → 100 %,
gated by error-rate + p99 latency Prometheus queries).

## 5. Local end-to-end validation

Run the full smoke test in a throwaway kind cluster — useful before opening a
PR to the platform repo:

```bash
/app/scripts/validate-cli.sh        # CLI bootstrap (no cluster needed)
/app/scripts/validate-argocd.sh     # GitOps YAML + kustomize
/app/scripts/validate-helm.sh       # render every chart
/app/scripts/validate-e2e.sh        # kind + argocd + scaffold + apply
```

## 6. Operations cheat-sheet

| I want to…                          | Command                                                   |
|-------------------------------------|-----------------------------------------------------------|
| See app status                      | `pavedroad status --service S --env E`                    |
| Force a sync                        | `pavedroad sync --service S --env E`                      |
| Roll back the latest deploy         | `ACME_LIVE=1 pavedroad rollback --service S --env prod`   |
| Spin up a sandbox namespace         | `ACME_LIVE=1 pavedroad ns bootstrap --name my-sandbox`    |
| Audit a legacy repo                 | `pavedroad doctor /path/to/repo`                          |
| Rotate a secret                     | `vault kv put kv/<svc>/db password=...` (auto-remounts)   |

## 7. Where to find things

| Need                 | Path                                                         |
|----------------------|--------------------------------------------------------------|
| Scaffolder templates | `/app/templates/backstage-scaffolder/{python-fastapi,go-gin,nodejs-express,java-springboot}/` |
| Hardened Dockerfiles | `/app/images/runtimes/{python,go,nodejs,java}/`              |
| Helm library         | `/app/platform/helm-library/`                                |
| Reusable CI          | `/app/ci/{github-actions,gitlab-ci,azure-devops}/`           |
| Kyverno policies     | `/app/security/kyverno/`                                     |
| GitOps repo layout   | `/app/gitops-bootstrap/gitops/` (apps, applicationsets, bootstrap, platform) |
| Reference service    | `/app/reference-service/`                                    |
| Dependency strategy  | `/app/docs/dependency-update-strategy.md`                    |
| Secrets pattern      | `/app/gitops-bootstrap/gitops/bootstrap/secrets-pattern.md`  |

## 8. Get unstuck

- Service won't deploy → `pavedroad doctor` then `pavedroad status`.
- ArgoCD won't sync → check `kubectl -n argocd describe app <svc>-<env>`.
- CI fails at "Verify Cosign signature" → check OIDC trust for your CI provider.
- Helm renders incorrectly → `helm template helm --debug` locally; ensure the
  service's `helm/values.yaml` aligns with `acme-platform-lib`'s schema.

That's the paved road. Stay on it; deviations require a waiver via
`/app/docs/phase-1/hardened-runtimes/waiver-process.md`.
