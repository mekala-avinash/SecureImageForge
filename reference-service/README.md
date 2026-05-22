# Reference Service — `paved-road-demo`

A fully working FastAPI service that uses **every** component of the paved road end to end.

## What it demonstrates

| Layer | Component |
|---|---|
| Runtime image | `images/runtimes/python/Dockerfile` |
| Build pipeline | `ci/github-actions/reusable-build.yml` (via `.github/workflows/build.yml`) |
| Helm chart | `platform/helm-library` (via `helm/` subchart) |
| Observability | OTel SDK auto-instrumented → OTel collector → Tempo / Mimir / Loki |
| Security | Pod Security Restricted + Kyverno verifyImages + Istio mTLS + Cilium default-deny |
| Supply chain | Cosign signed + SBOM + SLSA L3 attested |
| SLOs | Sloth `PrometheusServiceLevel` rendered by the library chart |
| DX | `pavedroad` CLI scaffolds equivalent of this service in 30 seconds |

## Run locally

```bash
make bootstrap
make dev          # docker compose deps
make run          # uvicorn on :8080
curl -s localhost:8080/healthz   # → {"ok": true}
curl -s localhost:8080/metrics   # → Prometheus metrics
open http://localhost:16686      # → Jaeger UI (your traces are here)
```

## Architecture rationale

The reference service exists so that:
1. Platform changes can be smoke-tested against a real consumer before rolling to all services.
2. Adopters have a complete, working example — not just docs.
3. CI for the platform monorepo `helm template`s + `kubeconform`s the reference service to catch library-chart breakages.

## Files

```
reference-service/
├── README.md
├── Dockerfile                 # thin wrapper of images/runtimes/python/Dockerfile
├── Makefile                   # includes developer-experience/makefiles/standard.mk
├── requirements.txt
├── catalog-info.yaml          # Backstage Component + API
├── src/reference_service/     # the actual code
│   ├── __init__.py
│   └── main.py
├── helm/                      # service Helm chart
│   ├── Chart.yaml
│   ├── values.yaml
│   └── templates/all.yaml
├── .github/workflows/build.yml
└── tests/test_smoke.py
```
