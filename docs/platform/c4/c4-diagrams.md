# C4 Architecture Diagrams (text-form)

## C1 — System Context

```text
              ┌──────────────────────────────────────────────────────────┐
              │                External Actors                           │
              │                                                          │
              │  End Users   Partners   Auditors   Engineers   Regulators│
              └─────────┬──────┬──────┬───────┬──────┬───────┬──────────┘
                        │      │      │       │      │       │
                        ▼      ▼      ▼       ▼      ▼       ▼
              ╔══════════════════════════════════════════════════════════╗
              ║                  ACME Enterprise Platform                ║
              ║                                                          ║
              ║   - Multi-tenant SaaS workloads                          ║
              ║   - Internal Developer Platform (Backstage)              ║
              ║   - Continuous Delivery via GitOps                       ║
              ║   - Autonomous Engineering Agents                        ║
              ║   - Continuous Compliance Evidence                       ║
              ╚═══════════════╤══════════════════════════════════════════╝
                              │
              ┌───────────────┼────────────────────────────────────────┐
              ▼               ▼                                        ▼
       ┌──────────────┐  ┌──────────────┐                       ┌──────────────┐
       │ Cloud (AWS / │  │ Identity     │                       │ External APIs│
       │ Azure / GCP) │  │ Providers    │                       │ (Stripe, etc)│
       └──────────────┘  │ (Okta/AAD)   │                       └──────────────┘
                         └──────────────┘
```

## C2 — Container View

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Edge Plane                                                                  │
│   Cloudflare (WAF, Bot, Anycast) ──▶  Global Accelerator                    │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
┌──────────────────────────────────▼──────────────────────────────────────────┐
│ Identity & API Gateway                                                       │
│   Keycloak (OIDC/SAML/SCIM)   Kong/Envoy Gateway   OPA decision API          │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │
                          ┌────────┼────────┐
                          ▼        ▼        ▼
                  ┌────────────┐ ┌────────────┐ ┌────────────┐
                  │ Region A   │ │ Region B   │ │ Region C   │
                  │ EKS Prod   │◀│ EKS Prod   │▶│ EKS Prod   │   (active-active)
                  └─────┬──────┘ └─────┬──────┘ └─────┬──────┘
                        │ Istio multi-primary (mTLS + east-west gateway)
                        ▼
        ┌───────────────────────────────────────────────────────┐
        │ Workloads (per-tenant namespaces or vClusters)         │
        │   Microservices (gRPC, REST), Jobs, CronJobs           │
        └─────────────┬─────────────────────┬───────────────────┘
                      │                     │
        ┌─────────────▼─────────┐  ┌────────▼──────────────────┐
        │ Data Plane             │  │ Platform Plane             │
        │ Aurora Global, Atlas,  │  │ Vault, Harbor, Backstage, │
        │ Kafka, NATS, Redis     │  │ ArgoCD/Rollouts/Events,    │
        │ S3, OpenSearch, Pinot  │  │ Temporal, Prom/Loki/Tempo  │
        └────────────────────────┘  └────────────────────────────┘
                      ▲                       │
                      │                       │
        ┌─────────────┴─────────┐  ┌──────────▼────────────────┐
        │ Security Plane         │  │ Agent Plane                │
        │ OPA, Kyverno, Falco,   │  │ 15 autonomous engineering │
        │ SPIRE, Sigstore        │  │ agents (Temporal-driven)   │
        └────────────────────────┘  └────────────────────────────┘
```

## C3 — Component View (Sample: Orders Domain)

```text
┌──────────────────── orders-domain (namespace: prod-orders-*) ────────────────┐
│                                                                              │
│   orders-api (Go, gRPC+REST)  ◀──▶  orders-worker (jobs)                     │
│            │                                  │                              │
│            ▼                                  ▼                              │
│   PostgreSQL (Aurora)              Kafka topics (orders.events.v1)           │
│            │                                  │                              │
│            ▼                                  ▼                              │
│   Redis (cache)                     Event-driven consumers (analytics,        │
│                                     fulfillment, notifications)              │
│                                                                              │
│   sidecars:                                                                  │
│     - istio-proxy (mTLS)                                                     │
│     - opentelemetry agent                                                    │
│     - vault csi (secrets mount)                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## C4 — Code-Level (illustrative, not exhaustive)

Each repository ships with package-level diagrams in its TechDocs via Backstage Plugin `tech-insights`. Standard layout enforced by golden-path scaffold:

```
service/
  api/                # OpenAPI / Proto definitions
  cmd/server/         # main entrypoint (or app/main.py for Python)
  internal/
    handler/          # transport
    domain/           # business logic (no external deps)
    repo/             # persistence
    event/            # message producers/consumers
  pkg/                # public helpers
  observability/      # tracer, metrics, logger
  config/             # env-driven config
  build/              # Dockerfile, helm chart
  docs/               # adr/, runbooks/, openapi.yaml
```
