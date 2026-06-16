# {{service_name}}

{{description}}

Paved-road Rust (axum) microservice. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap && make run     # :8080
make test                       # cargo test + clippy -D warnings
```

## Endpoints

- `GET /healthz`, `/ready`, `/metrics`, `/api/v1/items`

## Architecture

- Runtime: `cgr.dev/chainguard/static` (~6 MB, distroless, non-root 65532).
- Logging: tracing-subscriber JSON.
- Observability: OTLP/gRPC (`OTEL_EXPORTER_OTLP_ENDPOINT`).
- CI: GitHub Actions, GitLab CI, Azure DevOps.
