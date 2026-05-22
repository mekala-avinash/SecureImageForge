# Observability Stack

OpenTelemetry-native; vendor-portable. Every signal flows through the OTel collector and out to backends (Tempo/Mimir/Loki) we control. PII is redacted at the collector edge.

## Layout

```
observability/
├── otel-collector/values.yaml      # Gateway collector Helm values (THIS IS THE PRIMARY ENTRY POINT)
├── prometheus/                     # Use values from /app/docs/phase-1/02-observability/prometheus-values.yaml
├── loki/                           # Use values from /app/docs/phase-1/02-observability/loki-values.yaml
├── tempo/                          # Use values from /app/docs/phase-1/02-observability/tempo-values.yaml
└── grafana-dashboards/             # JSON dashboards (provisioned via grafana-operator)
```

## Architecture rationale

- Single OTel pipeline; backends are interchangeable.
- PII redaction at the **collector**, not the service — services emit raw, collector sanitizes.
- Tail sampling preserves errors + slow traces 100%; 5% of others; cost stays bounded.
- Logs/traces/metrics carry a stable `service.name` resource attribute set by the Helm library chart automatically.

## Service integration (zero-code)

Helm library chart sets these env vars automatically when `otel.enabled: true`:

```
OTEL_SERVICE_NAME            = <values.name>
OTEL_EXPORTER_OTLP_ENDPOINT  = http://otel-collector.observability:4317
OTEL_RESOURCE_ATTRIBUTES     = service.name=...,service.namespace=...,team=...
```

For zero-code auto-instrumentation the OTel Operator injects an SDK sidecar based on the pod annotation `instrumentation.opentelemetry.io/inject-sdk: "true"` (set automatically when `otel.sdk: auto`).

## SLOs as code

The library chart's `acme.slo` helper renders a Sloth `PrometheusServiceLevel` from the service's `values.yaml`. Sloth generates burn-rate alerts and records SLO error budgets — runbooks autolinked to Backstage TechDocs.

## Operational guidance

- **Drop-in upgrades.** Renovate keeps the OTel Collector chart and image current.
- **Cost control.** Tempo retention 7d hot / 30d sampled; Loki 90d hot + 7y S3 cold; Prom 30d local + 13mo Mimir.
- **Multi-cluster.** Each cluster runs its own collector and pushes to per-region Tempo/Mimir/Loki; Grafana federates queries.
