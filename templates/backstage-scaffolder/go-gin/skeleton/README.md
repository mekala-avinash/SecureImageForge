# {{service_name}}

{{description}}

Paved-road Go (Gin) microservice. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap
make run        # serves on :8080
make test
```

## Endpoints

- `GET /healthz` — liveness
- `GET /ready`   — readiness
- `GET /metrics` — Prometheus metrics
- `GET /api/v1/items` — example

## Architecture

- **Runtime**: `cgr.dev/chainguard/static` (~12 MB, distroless, non-root 65532).
- **Framework**: Gin + zerolog (JSON, trace-correlated).
- **Observability**: OTLP/gRPC tracer (`OTEL_EXPORTER_OTLP_ENDPOINT`) + Prometheus middleware.
- **Deploy**: Helm chart imports `acme-platform-lib`; ArgoCD GitOps.
- **CI**: GitHub Actions (default), GitLab CI, and Azure DevOps templates included.
