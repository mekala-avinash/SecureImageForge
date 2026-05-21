# Observability Rollout — Phase 1

> Phase-1 Workstream 2 · Lead: Head of SRE · Window: Weeks 1–12

## Goal

Every in-scope service emits **traces, metrics, logs, and SLOs as code**, flowing through a single OpenTelemetry pipeline to Prometheus (Mimir for long-term), Loki, and Tempo, with PII redaction at the collector edge and tail sampling. **≥ 80% of in-scope services** must be fully instrumented by end of Phase 1.

## Stack (vendored under `gitops/platform/observability/`)

| Component | Helm chart | Storage | Retention | Notes |
|---|---|---|---|---|
| Prometheus | `prometheus-community/kube-prometheus-stack` | EBS gp3 (per cluster) + Mimir (S3 long-term) | 30d local + 13mo Mimir | Mimir from Week 6 |
| Loki | `grafana/loki` | S3 chunks | 90d hot + 7y cold | Log lake doubles as compliance archive |
| Tempo | `grafana/tempo` | S3 blocks | 7d hot + 30d sampled | Cost-controlled via tail sampling |
| OTel Collector (gateway + agent) | `open-telemetry/opentelemetry-collector` | n/a | n/a | DaemonSet + Deployment |
| Grafana | `grafana/grafana` | Postgres (RDS) | n/a | OIDC via Keycloak |
| Pyroscope (continuous profiling) | `grafana/pyroscope` | S3 | 30d | Optional in Phase 1; mandatory in P2 |

## Rollout sequence

| Week | Milestone |
|---|---|
| 1 | Prom + Loki + Tempo + OTel gateway live in `mgmt-use1`; Grafana with Keycloak SSO |
| 2 | 3 lighthouse services emit traces + metrics + logs |
| 3 | OTel auto-instrumentation Operator deployed; first 5 services onboarded |
| 4 | First SLOs as code (Sloth); first burn-rate alerts firing in test mode |
| 5 | Dashboards as code (`grafana-operator`) for top 10 services |
| 6 | Loki PII redaction processors active; OTel tail sampling tuned |
| 7 | Auto-remediation playbook for top 3 alert classes (pod restart, HPA bump, drain) |
| 8 | Mimir long-term storage active; Prom local retention reduced to 30d |
| 9 | 10 services fully instrumented end-to-end |
| 10 | Chaos drill validates burn-rate alerts trigger correctly |
| 11 | Runbook coverage: every page-severity alert has a linked runbook |
| 12 | Phase-1 gate prep — KPI verification |

## Instrumentation Guide (per service)

The simplest path uses **zero-code OTel auto-instrumentation** via the operator (Java/Python/Node.js/.NET/Go). For Rust + custom Go, use the SDK directly.

### Auto-instrumentation (recommended)

```yaml
# annotation on Deployment template — operator injects sidecar / env
metadata:
  annotations:
    instrumentation.opentelemetry.io/inject-python: "platform-observability/default"
    instrumentation.opentelemetry.io/container-names: "app"
```

### Manual (when auto-instrumentation isn't supported)

```bash
# Add to service dependencies
pip install opentelemetry-distro opentelemetry-exporter-otlp
```

```python
# main entry — only needed if not using auto-instrumentation
from opentelemetry import trace
from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
FastAPIInstrumentor.instrument_app(app)
```

Configure via env (set in Helm values):

```yaml
env:
  - { name: OTEL_SERVICE_NAME, value: orders-api }
  - { name: OTEL_EXPORTER_OTLP_ENDPOINT, value: http://otel-gateway.observability:4317 }
  - { name: OTEL_TRACES_SAMPLER, value: parentbased_traceidratio }
  - { name: OTEL_TRACES_SAMPLER_ARG, value: "0.1" }   # collector tail-samples; this is parent rate
  - { name: OTEL_RESOURCE_ATTRIBUTES, value: "deployment.environment=prod,team=orders" }
```

### Logs (structured JSON)

All services emit JSON to stdout with required fields:

```json
{
  "ts": "2026-01-15T14:32:00Z",
  "level": "info",
  "service": "orders-api",
  "trace_id": "00-...-...-01",
  "span_id": "...",
  "correlation_id": "01HX...",
  "msg": "...",
  "data": {}
}
```

Loki ingests via Promtail / Alloy. PII fields (`email`, `phone`, `ip`) are hashed at the OTel collector — see processor config in `/app/docs/observability/otel/collector-gateway.yaml`.

## SLOs as Code

Every service ships at least 2 SLOs:
- Availability (success-rate)
- Latency (p99)

Use the Sloth pattern in `/app/docs/observability/prometheus/slo-orders-api.yaml`. Multi-window multi-burn-rate alerts are auto-generated.

## Dashboard standards

- Every dashboard lives in Git (`gitops/platform/observability/dashboards/`), provisioned via `grafana-operator`.
- Every service gets a **standard dashboard** (RED + USE + SLO) auto-generated from a Backstage scaffolder.
- No "owned by one person" dashboards. CODEOWNERS apply.

## Auto-remediation playbooks (Phase-1 scope)

| Alert | Auto-action | Approval |
|---|---|---|
| Pod CrashLoopBackOff > 3min | Delete pod (restart) | auto |
| Memory pressure on node | Cordon + drain (Karpenter rebalances) | auto |
| HPA at max, RPS climbing | Bump max replicas by 25% (one step) | auto-with-cooloff |
| DB connection pool exhausted | Switch reads to replica | human-in-loop |

Workflows live in `gitops/platform/observability/runbooks/`. Each links from the alert's `runbook_url`.

## KPIs

| KPI | Target |
|---|---|
| % services with traces flowing | ≥ 80% of in-scope |
| % services with SLOs as code | ≥ 80% of in-scope |
| Mean time to first dashboard (new service) | ≤ 5 min |
| Alert noise (page/week per service) | < 2 |
| Runbook coverage of page-severity alerts | 100% |
| Log PII redaction coverage | 100% (verified by sampling) |
