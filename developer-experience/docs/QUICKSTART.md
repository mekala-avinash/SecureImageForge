# Developer Quickstart

> 0 → first commit on the paved road in under an hour.

## 1. Install the CLI (one time)

```bash
pipx install -e /app/developer-experience/pavedroad-cli
# or in a real org:  brew install acme/tap/pavedroad
pavedroad version
```

## 2. Scaffold a new service

```bash
pavedroad new service --name orders-api --language python --team orders --out ~/code
cd ~/code/orders-api
```

You now have a complete, paved-road-compliant Python FastAPI service with:
- Hardened multi-arch Dockerfile from `/app/images/runtimes/python/Dockerfile`.
- `.github/workflows/build.yml` that calls the reusable build workflow.
- `helm/` Chart that imports the `acme-platform-lib` library chart.
- OpenTelemetry SDK pre-wired.
- `healthz` + `ready` + `metrics` endpoints.
- `catalog-info.yaml` for Backstage.
- Tests, ADR scaffold, runbook scaffold.

## 3. Local dev loop

Open in VS Code; the devcontainer starts automatically (`/app/developer-experience/devcontainer/`).

```bash
make bootstrap        # install deps
make dev              # docker compose up Postgres + Redis + OTel + Jaeger
make run              # uvicorn auto-reload
make test             # pytest
```

Open:
- `http://localhost:8080/healthz` — should return `{"ok": true}`.
- `http://localhost:8080/metrics` — Prometheus format metrics.
- `http://localhost:16686` — Jaeger UI; you should see traces appear after your first request.

## 4. Push → CI does the rest

```bash
git init && git add . && git commit -S -m "feat: initial scaffold"
gh repo create acme/orders-api --private --source . --push
```

On push, the reusable workflow (`ci/github-actions/reusable-build.yml`):
1. Runs gitleaks, semgrep, osv-scanner, hadolint.
2. Builds multi-arch image (amd64 + arm64) with reproducible timestamps.
3. Scans with Trivy + Grype (fails on HIGH/CRITICAL).
4. Generates SBOM (SPDX + CycloneDX).
5. Cosign-signs (keyless via Fulcio + Rekor).
6. Generates SLSA L3 provenance.
7. Pushes to registry by digest.
8. Opens a GitOps PR in `acme/gitops` bumping the **dev** overlay's image digest.

## 5. See it in production-shaped infrastructure

Once the GitOps PR merges, Argo CD reconciles and your service runs in the dev cluster. Watch:

```bash
pavedroad watch --service orders-api --env dev
```

## 6. Promote

```bash
pavedroad promote --service orders-api --to staging   # opens GitOps PR for staging
pavedroad promote --service orders-api --to prod      # canary via Argo Rollouts
```

## 7. Observability — already wired

- Backstage page: `https://backstage.acme.io/catalog/default/component/orders-api`
- Grafana dashboard: `https://grafana.acme.io/d/svc-orders-api`
- Tempo traces, Loki logs, Prom metrics — all auto-wired by the library chart.

## 8. Common operations

| Task | Command |
|---|---|
| Run unit tests | `make test` |
| Run e2e against local stack | `make e2e` |
| Lint Helm chart | `helm template helm \| kubeconform --strict` |
| Re-render OpenAPI | `make openapi` |
| Scan local image | `trivy image $(make image-tag)` |
| Roll back prod | `pavedroad rollback --service orders-api --env prod` |

## 9. Off-road escape hatches

If the paved road doesn't fit your service, see the waiver process at:
`/app/docs/phase-1/01-hardened-runtimes/waiver-process.md`

But before you go off-road, drop into `#platform-pavedroad` — we'll usually pair on a solution that keeps you on-road.
