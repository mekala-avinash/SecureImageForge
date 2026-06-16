# {{service_name}}

{{description}}

Paved-road ASP.NET Core 9 service. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap && make run     # http://localhost:8080
make test
```

## Endpoints

- `GET /healthz`, `/ready`, `/metrics`, `/api/v1/items`

## Architecture

- Runtime: `cgr.dev/chainguard/dotnet-runtime:9.0` (distroless, non-root 65532).
- Logging: Serilog RenderedCompactJsonFormatter with trace correlation.
- Observability: OpenTelemetry (OTLP/gRPC) + prometheus-net auto-metrics.
- CI: GitHub Actions, GitLab CI, Azure DevOps.
