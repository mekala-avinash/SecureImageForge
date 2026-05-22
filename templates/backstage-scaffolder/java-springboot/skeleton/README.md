# {{service_name}}

{{description}}

Paved-road Spring Boot 3 service on JRE 21. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap
make run        # spring-boot:run on :8080
make test
```

## Endpoints

- `GET /actuator/health/liveness`  — liveness probe
- `GET /actuator/health/readiness` — readiness probe
- `GET /actuator/prometheus`        — Prometheus metrics (Micrometer)
- `GET /api/v1/items`               — example

The Helm chart maps Kubernetes probes to actuator endpoints automatically (see `acme.workload`).

## Architecture

- **Runtime**: `cgr.dev/chainguard/jre:openjdk-21` (distroless, non-root 65532).
- **Logging**: Logstash JSON encoder with traceId/spanId via Micrometer Tracing.
- **Observability**: Micrometer + OTel bridge (OTLP/gRPC) + Prometheus registry.
- **Deploy**: Helm chart imports `acme-platform-lib`; ArgoCD GitOps.
- **CI**: GitHub Actions, GitLab CI, and Azure DevOps templates included.
