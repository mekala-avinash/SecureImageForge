# Helm Library Chart — `acme-platform-lib`

> A Helm **library chart** (not installable) that emits all paved-road Kubernetes primitives from a single small `values.yaml` in each service chart.

## Why a library chart?

- One canonical implementation of security defaults (non-root, RO FS, dropped caps, RuntimeDefault seccomp).
- A chart-version bump flows the latest hardening to every service.
- Service charts stay ~30 lines of `values.yaml` + a 1-file `templates/everything.yaml` that includes the library helpers.

## Emitted primitives

| Helper | Resource(s) |
|---|---|
| `acme.workload`        | Deployment **or** Argo Rollout (toggled by `values.kind`) |
| `acme.service`         | ServiceAccount + Service |
| `acme.hpa`             | HorizontalPodAutoscaler v2 (CPU + custom Prom metrics) |
| `acme.pdb`             | PodDisruptionBudget |
| `acme.networkPolicy`   | CiliumNetworkPolicy (default-deny + Istio ingress + DNS + OTel egress) |
| `acme.istioAuthz`      | PeerAuthentication (STRICT mTLS) + AuthorizationPolicy |
| `acme.serviceMonitor`  | Prometheus Operator ServiceMonitor |
| `acme.slo`             | Sloth PrometheusServiceLevel (multi-window burn-rate alerts) |

## Usage in a service chart

`Chart.yaml`:

```yaml
apiVersion: v2
name: orders-api
type: application
version: 0.1.0
appVersion: "1.0.0"
dependencies:
  - name: acme-platform-lib
    version: "1.0.0"
    repository: "file://../../platform/helm-library"
```

`templates/all.yaml`:

```yaml
{{ include "acme.workload" . }}
---
{{ include "acme.service" . }}
---
{{ include "acme.hpa" . }}
---
{{ include "acme.pdb" . }}
---
{{ include "acme.networkPolicy" . }}
---
{{ include "acme.istioAuthz" . }}
---
{{ include "acme.serviceMonitor" . }}
---
{{ include "acme.slo" . }}
```

`values.yaml`:

```yaml
name: orders-api
team: orders
image:
  repository: registry.acme.io/orders/api
  # digest set by GitOps PR
hpa:
  minReplicas: 6
  maxReplicas: 60
slos:
  objectives:
    - name: requests-availability
      objective: 99.95
      # sli.errorQuery / totalQuery default to job=={{ .Values.name }}
```

## Security guarantees (always on, can't be disabled by service overrides)

- Pod runs as UID/GID **65532** (`nonroot`).
- Read-only root filesystem; `/tmp` is `emptyDir` in-memory (`medium: Memory`).
- `seccompProfile: RuntimeDefault`.
- All capabilities dropped.
- `automountServiceAccountToken: false` (services must opt in if they need K8s API).
- Istio PeerAuthentication mTLS STRICT.
- Cilium default-deny + explicit ingress/egress.
- Image must be a digest in prod overlays (`acme.image` helper fails otherwise).

## Testing the chart locally

```bash
cd platform/helm-library
helm dependency update ../reference-service/helm
helm template ../reference-service/helm --debug | yq
helm unittest .   # tests/ directory holds chart unit tests (TBD)
kubeconform <(helm template ../reference-service/helm) --strict
```

## Versioning

- `1.x.x` — breaking changes require a major bump + 6-month deprecation window.
- Changes flow through the Renovate config in `acme/gitops` automatically.

## Operational guidance

- **Service charts** are owned by service teams; this library chart is owned by Platform Engineering.
- **Breaking schema changes** (e.g., renaming `containerSecurityContext` → `securityContext`) → never. Add fields, deprecate slowly, remove only at major.
- **New primitives** (e.g., adding a `KEDA ScaledObject`) → add as a new helper; do not bundle into existing helpers.
