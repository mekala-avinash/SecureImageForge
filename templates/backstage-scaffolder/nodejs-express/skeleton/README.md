# {{service_name}}

{{description}}

Paved-road Node.js Express service. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap
make run        # tsx watch on :8080
make test       # vitest + coverage
```

## Endpoints

- `GET /healthz` — liveness
- `GET /ready`   — readiness
- `GET /metrics` — Prometheus metrics
- `GET /api/v1/items` — example

## Architecture

- **Runtime**: `cgr.dev/chainguard/node:20` (~80 MB, distroless, non-root 65532).
- **Logging**: pino JSON with trace correlation.
- **Observability**: OTel auto-instrumentation; OTLP/gRPC via `OTEL_EXPORTER_OTLP_ENDPOINT`.
- **Config**: zod-validated env (`PORT`, `LOG_LEVEL`, `OTEL_SERVICE_NAME`, `DRAIN_TIMEOUT_MS`).
- **Deploy**: Helm chart imports `acme-platform-lib`; ArgoCD GitOps.
- **CI**: GitHub Actions, GitLab CI, and Azure DevOps templates included.
