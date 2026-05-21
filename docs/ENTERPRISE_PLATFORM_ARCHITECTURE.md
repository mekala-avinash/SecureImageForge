# Enterprise Platform Architecture & Evolution Blueprint

> **Document Class:** Principal Architecture / Platform Engineering Master Document
> **Audience:** Platform Engineering, DevSecOps, SRE, Security, Compliance, Engineering Leadership
> **Status:** Implementation-Ready Blueprint (v1.0)
> **Scope:** End-to-end platform transformation — hardened runtimes, multi-agent autonomous engineering, secure software supply chain, multi-tenant cloud-native delivery, governance, and compliance.

---

## Table of Contents

1.  [Executive Summary](#1-executive-summary)
2.  [Current State Assessment](#2-current-state-assessment)
3.  [Target State Architecture](#3-target-state-architecture)
4.  [Runtime Image Strategy](#4-runtime-image-strategy)
5.  [Multi-Agent System Design](#5-multi-agent-system-design)
6.  [Security Architecture](#6-security-architecture)
7.  [Platform Architecture](#7-platform-architecture)
8.  [Kubernetes & Cloud Design](#8-kubernetes--cloud-design)
9.  [CI/CD & GitOps](#9-cicd--gitops)
10. [Supply Chain Security](#10-supply-chain-security)
11. [Observability Strategy](#11-observability-strategy)
12. [Enterprise Integrations](#12-enterprise-integrations)
13. [Governance & Compliance](#13-governance--compliance)
14. [Scalability Strategy](#14-scalability-strategy)
15. [Reliability Engineering](#15-reliability-engineering)
16. [Disaster Recovery](#16-disaster-recovery)
17. [Cost Optimization](#17-cost-optimization)
18. [Operational Excellence](#18-operational-excellence)
19. [Implementation Roadmap](#19-implementation-roadmap)
20. [Appendix](#20-appendix)

---

## 1. Executive Summary

### 1.1 Mission

Transform the existing application repository into a **production-grade, multi-tenant, regulated-ready enterprise platform** that:

- Builds, ships, and operates software through **hardened minimal OCI images** (distroless / Wolfi-class).
- Delivers software via a **secure, attested, GitOps-driven supply chain** aligned to **SLSA L3+**, **in-toto**, and **NIST SSDF**.
- Operates a fleet of **15 autonomous engineering agents** that continuously test, refactor, secure, deploy, and optimize the platform under strict sandboxing and human-in-the-loop approval.
- Runs on **multi-region, multi-cloud Kubernetes** with **service mesh**, **progressive delivery**, and **active-active topology**.
- Satisfies **SOC 2 Type II, ISO 27001, PCI-DSS, HIPAA, GDPR**, and **FedRAMP Moderate** readiness through policy-as-code.

### 1.2 Strategic Pillars

| Pillar | Outcome | Primary Tooling |
|---|---|---|
| **Hardened Runtimes** | < 50 MB images, 0 CVE SLA, signed + SBOM | Wolfi, Chainguard, BuildKit, Cosign, Syft |
| **Autonomous Engineering** | 15 agents reduce MTTR & toil ≥ 60% | Temporal, NATS, LangGraph, OpenTelemetry |
| **Zero-Trust Supply Chain** | Every artifact signed, attested, verified | Sigstore, in-toto, SLSA L3, Kyverno |
| **Cloud-Native Platform** | 99.99% SLO, multi-region active-active | EKS/AKS/GKE, Istio, Argo, Crossplane |
| **Compliance by Default** | Continuous audit evidence, OPA policy enforcement | OPA/Gatekeeper, Falco, Drata/Vanta connectors |
| **Operational Excellence** | Full O11Y, error budgets, auto-remediation | Prometheus, Grafana, Tempo, Loki, OTel |

### 1.3 Key Outcomes (12-Month Horizon)

- **MTTR**: 4h → 25 min (auto-remediation by agents).
- **Lead time for change**: 5 days → 45 min (preview envs + progressive delivery).
- **Image CVE exposure**: ↓ 95% (distroless + auto-rebuild on CVE).
- **Compliance audit prep**: 6 weeks → 2 days (continuous evidence).
- **Infra cost**: ↓ 25–35% (Karpenter, spot, right-sizing agent).

### 1.4 Investment Summary

3 phases (12 months) — Foundation, Industrialization, Autonomy. Estimated **22–28 FTE-quarters** of platform engineering effort. See [Section 19](#19-implementation-roadmap).

---

## 2. Current State Assessment

### 2.1 Repository Snapshot

The analyzed repository is a typical **monolithic full-stack template**:

- **Backend:** FastAPI (Python 3.11), Uvicorn, MongoDB driver.
- **Frontend:** React (CRA), Tailwind, Yarn.
- **Process supervision:** Supervisor (single VM/container).
- **Secrets:** `.env` files (plain text).
- **No CI/CD, no IaC, no image hardening, no observability stack, no policy enforcement.**

### 2.2 Gap Analysis

| Domain | Current | Target | Risk |
|---|---|---|---|
| Image baseline | `python:3.11`, `node:20` (Debian, ~900 MB) | Distroless/Wolfi (~40 MB) | CVE blast radius, supply chain |
| Runtime user | root | non-root (UID 65532) | Container escape |
| Secrets | `.env` files | Vault + KMS + CSI | Plaintext exfil |
| Build provenance | none | SLSA L3 + Cosign | Tampering, dependency confusion |
| SDLC controls | none | SAST/DAST/SCA/IaC scan in PR | Vulnerable releases |
| Observability | stdout logs | OTel + Prom + Loki + Tempo | Blind ops |
| Topology | 1 region, 1 cluster | 3 regions, active-active | DR, RTO/RPO |
| Tenancy | single tenant | namespace + vCluster isolation | Noisy neighbor, data isolation |
| Compliance | none | SOC2/ISO/PCI/HIPAA/GDPR | Sales blockers |
| Identity | local creds | OIDC + SCIM + MFA + RBAC/ABAC | Insider risk |

### 2.3 Strengths to Preserve

- Clear frontend/backend split → maps naturally to microservices.
- FastAPI Pydantic models → strong foundation for OpenAPI contracts.
- Environment-driven config → already 12-factor aligned.

### 2.4 Constraints

- Multi-cloud (AWS primary, Azure secondary, GCP tertiary).
- Air-gapped deployment variant required (gov/regulated tenants).
- 24×7×365 operations with regional failover.
- Regulatory: SOC2 + ISO27001 (Year 1), PCI-DSS + HIPAA (Year 2), FedRAMP (Year 3).

---

## 3. Target State Architecture

### 3.1 North-Star Principles

1. **Everything is code** — infrastructure, policy, compliance evidence, runbooks.
2. **Zero-trust by default** — mTLS everywhere, signed artifacts, attested deploys, deny-by-default RBAC.
3. **Immutable & ephemeral** — no in-place mutation, no SSH, no snowflakes.
4. **Observable & explainable** — every action emits structured telemetry & audit log.
5. **Autonomous where safe, human where it matters** — agents propose, humans (or policy) approve high-blast-radius changes.
6. **Multi-tenant from day one** — namespace, network, data, and identity isolation.
7. **Cloud-portable** — Kubernetes as the substrate, Crossplane for cloud control planes.

### 3.2 Logical High-Level Architecture (C4 — Context)

```text
                 ┌──────────────────────────────────────────────────────────────┐
                 │                       External Actors                         │
                 │   End Users · Partners · Auditors · Regulators · Engineers   │
                 └─────────────┬──────────────────────┬─────────────────────────┘
                               │                      │
                       (OIDC/SAML/SCIM)         (Public APIs / Web)
                               │                      │
                 ┌─────────────▼──────────────────────▼─────────────────────────┐
                 │           Edge / Identity / API Gateway (Global)              │
                 │  Cloudflare · WAF · Bot Mgmt · Anycast · OIDC Broker (Keycloak)│
                 └─────────────┬─────────────────────────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       │                       │                       │
┌──────▼──────┐         ┌──────▼──────┐         ┌──────▼──────┐
│ Region A    │         │ Region B    │         │ Region C    │
│ (Primary)   │ <─────> │ (Secondary) │ <─────> │ (Tertiary)  │
│ EKS/AKS/GKE │  active │ EKS/AKS/GKE │ active  │ EKS/AKS/GKE │
└──────┬──────┘  -active└──────┬──────┘ -passive└──────┬──────┘
       │                       │                       │
       └───── Service Mesh (Istio Multi-Primary, mTLS) ┘
                               │
       ┌───────────────────────┼───────────────────────┐
       │            Platform Capability Plane          │
       │  GitOps(Argo) · Secrets(Vault) · Policy(OPA)  │
       │  Observability(OTel/Prom/Loki/Tempo)          │
       │  Supply Chain(Sigstore/Rekor) · Agents(Temporal)│
       └───────────────────────────────────────────────┘
```

### 3.3 Capability Planes

| Plane | Purpose | Components |
|---|---|---|
| **Edge Plane** | Global ingress, WAF, identity | Cloudflare, AWS Global Accelerator, Keycloak |
| **Control Plane** | Cluster mgmt, policy, GitOps | EKS/AKS/GKE control planes, ArgoCD, Crossplane |
| **Data Plane** | Workload execution | Karpenter-managed nodes, vClusters per tenant tier |
| **Platform Plane** | Developer & SRE services | Backstage IDP, Vault, Harbor, Prom/Grafana, Temporal |
| **Security Plane** | Continuous controls | OPA/Gatekeeper, Kyverno, Falco, Trivy Operator, Sigstore |
| **Data Persistence Plane** | Stateful services | RDS/Aurora multi-AZ, MongoDB Atlas global, Kafka, Redis |

See [Section 7](#7-platform-architecture) for component-level C4.

---

## 4. Runtime Image Strategy

### 4.1 Image Taxonomy

```
registry.acme.io/
├── base/                       # OS-level bases (rebuild nightly)
│   ├── wolfi-static:<digest>   # 2 MB, scratch-like + Wolfi packages
│   ├── wolfi-glibc:<digest>    # ~6 MB
│   └── distroless-cc:<digest>  # for cgo binaries
├── runtime/                    # Language runtimes (FROM base/*)
│   ├── nodejs/{18,20,22}-min:<digest>
│   ├── python/{3.11,3.12,3.13}-min:<digest>
│   ├── go/{1.22,1.23}-min:<digest>
│   ├── rust/{1.80,1.81}-min:<digest>
│   ├── java/{17,21}-jre-min:<digest>
│   ├── ruby/{3.2,3.3}-min:<digest>
│   ├── php/{8.2,8.3}-fpm-min:<digest>
│   └── dotnet/{8.0,9.0}-runtime-min:<digest>
├── tooling/                    # CI & ops tools (FROM base/*)
│   ├── git, docker-cli, buildkit, kubectl, helm
│   ├── terraform, opentofu, ansible, packer
│   ├── aws-cli, azure-cli, gcloud, vault
│   ├── trivy, syft, grype, cosign, jq, yq
│   └── k6, playwright
└── app/                        # Per-application images (FROM runtime/*)
    └── <team>/<service>:<semver>-<git-sha>
```

### 4.2 Per-Runtime Specifications

#### 4.2.1 Common Standards (apply to ALL runtimes)

| Standard | Value |
|---|---|
| **Base OS strategy** | Wolfi (preferred) → Chainguard equivalents → distroless (fallback for binary-only) |
| **Default user** | `nonroot` (UID 65532, GID 65532) |
| **Root FS** | read-only; `/tmp` and `/var/run` as `emptyDir` |
| **Capabilities** | drop ALL; add only explicitly required (e.g. `NET_BIND_SERVICE` for ports <1024 — preferred: bind to 8080+) |
| **Shell** | none in runtime (no `/bin/sh`); diagnostics via `kubectl debug` ephemeral container |
| **Package manager** | absent in runtime; only present in `-builder` stage |
| **Image size goal** | ≤ 100 MB runtime; ≤ 50 MB preferred |
| **CVE SLA** | Critical: 24h, High: 7d, Medium: 30d, Low: 90d |
| **Update cadence** | Nightly rebuild of bases; weekly rebuild of runtimes; daily CVE scan |
| **Signing** | Cosign keyless (Fulcio OIDC) + key-based for air-gap |
| **SBOM** | Syft → SPDX 2.3 JSON, attached as OCI artifact via Cosign |
| **Provenance** | SLSA L3 in-toto attestation, attached as OCI artifact |
| **Tag policy** | Immutable digests only in production manifests; semver tags for humans |
| **Multi-arch** | `linux/amd64` + `linux/arm64` mandatory; `linux/s390x` for FSI tenants |
| **Reproducible builds** | SOURCE_DATE_EPOCH, deterministic ordering, BuildKit `--output=type=oci,rewrite-timestamp=true` |

#### 4.2.2 Runtime-Specific Matrix

| Runtime | Base | Size Goal | Notes |
|---|---|---|---|
| Node.js 20 | `wolfi-glibc` + `nodejs-20` apk | ≤ 80 MB | `NODE_ENV=production`, `--no-experimental-fetch` off |
| Python 3.12 | `wolfi-glibc` + `python-3.12` apk | ≤ 75 MB | wheels-only install, no `pip` in runtime |
| Go 1.23 | `wolfi-static` (scratch+ca) | ≤ 15 MB | static binary, CGO_ENABLED=0 |
| Rust 1.81 | `wolfi-static` | ≤ 12 MB | `--target x86_64-unknown-linux-musl` |
| Java 21 | `wolfi-glibc` + `openjdk-21-jre-headless` | ≤ 180 MB | jlink custom JRE, G1GC, `-XX:+UseContainerSupport` |
| Ruby 3.3 | `wolfi-glibc` + `ruby-3.3` apk | ≤ 90 MB | bundler frozen, jemalloc |
| PHP 8.3 | `wolfi-glibc` + `php-8.3-fpm` apk | ≤ 95 MB | OPcache enabled, no dev modules |
| .NET 9 | `wolfi-glibc` + `dotnet-9.0-runtime` apk | ≤ 110 MB | `--runtime linux-musl-x64`, AOT where possible |

See `/app/docs/runtime-images/runtimes/` for full Dockerfile templates.

### 4.3 Image Lifecycle Management

```
   ┌───────────┐    nightly    ┌───────────┐   PR      ┌───────────┐  promote  ┌───────────┐
   │  Source    │ ──BuildKit──▶│  staging  │ ──scan──▶│  signed   │ ──Cosign──▶│  prod     │
   │ Dockerfile │              │ registry  │  Trivy   │  artifact │  verify    │ registry  │
   └───────────┘               └───────────┘  Grype   └───────────┘            └───────────┘
                                                 │
                                            CVE > SLA?
                                                 │
                                                 ▼
                                         Auto-PR rebuild
```

### 4.4 Deprecation Policy

- **N-2 versions supported** for language runtimes.
- **6-month deprecation window** announced via Backstage TechDocs + Slack `#platform-announcements`.
- **Hard cutoff** enforced via Kyverno policy (`disallow-deprecated-images`).

### 4.5 Registry Architecture

- **Primary:** Harbor (self-hosted, HA) with replication to AWS ECR, Azure ACR, GCP Artifact Registry.
- **Geo-replication:** pull-through caches in each region.
- **Air-gapped:** offline tarball bundle generated by `cosign save` → transferred via approved one-way diode.
- **Quotas & retention:** 90 days for untagged, 2 years for tagged, indefinite for signed-prod.

### 4.6 Templates

Concrete Dockerfile templates and BuildKit strategies live in:
- `/app/docs/runtime-images/runtimes/Dockerfile.nodejs.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.python.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.go.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.rust.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.java.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.ruby.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.php.template`
- `/app/docs/runtime-images/runtimes/Dockerfile.dotnet.template`
- `/app/docs/runtime-images/tooling/Dockerfile.tooling-multi.template`
- `/app/docs/runtime-images/templates/buildkit-build.sh`

---

## 5. Multi-Agent System Design

### 5.1 Vision

A **fleet of 15 specialized autonomous agents** acts as a continuously-running engineering team — proposing PRs, validating policies, mitigating incidents, optimizing cost, and keeping the platform compliant. Agents are **sandboxed**, **auditable**, and **bounded** by explicit policy.

### 5.2 Agent Catalog

| # | Agent | Trigger | Output | Approval |
|---|---|---|---|---|
| 1 | **Unit Testing Agent** | PR open, code merge | Generated unit tests, coverage report | Auto-merge if ≥90% & green |
| 2 | **Integration Testing Agent** | Pre-deploy gate | Contract & e2e test results | Block on fail |
| 3 | **Security Scanning Agent** | PR, nightly | SAST/SCA/secret findings → PR comments / Jira | CISO group on Critical |
| 4 | **Documentation Agent** | Code merge, API change | OpenAPI diff, TechDocs PR, ADR scaffolds | Tech writer review |
| 5 | **Refactoring Agent** | Weekly schedule, complexity threshold | Refactor PR with proofs of behavior preservation | Senior eng |
| 6 | **CI/CD Optimization Agent** | Pipeline metrics window | Pipeline change PR (cache, parallelism) | Platform team |
| 7 | **Deployment Automation Agent** | Argo Rollouts events | Promotes canary, executes rollback | Auto with SLO gate |
| 8 | **Monitoring & Observability Agent** | New service, drift | Dashboards, alerts, SLOs as code | SRE review |
| 9 | **Dependency Management Agent** | CVE feed, weekly | Renovate-style PRs with risk score | Auto-merge low-risk |
| 10 | **Incident Analysis Agent** | PagerDuty page | Timeline, blameless RCA draft, action items | SRE lead |
| 11 | **Cost Optimization Agent** | Daily billing pull | Right-sizing PRs, savings plan recs | FinOps + owner |
| 12 | **Infrastructure Drift Agent** | Hourly poll | Drift report, remediation PR | IaC owner |
| 13 | **Kubernetes Optimization Agent** | KRR / VPA signals | Resource request PRs, HPA tuning | Service owner |
| 14 | **API Contract Validation Agent** | OpenAPI diff | Breaking-change report, consumer notify | API council |
| 15 | **Supply Chain Security Agent** | Build event | Verifies signatures, SBOMs, provenance; blocks if missing | Security on bypass |

### 5.3 Agent Anatomy (applies to ALL)

```yaml
agent_template:
  identity:
    workload_identity: IRSA / WIF (cloud-native, no static creds)
    spiffe_id: spiffe://platform.acme.io/agents/<name>/<version>
  permissions:
    rbac: least-privilege ClusterRole/Role
    abac_attrs: { team, env, blast_radius }
    secrets: Vault dynamic, TTL ≤ 1h
  sandboxing:
    runtime: gVisor (runsc) or Kata
    fs: read-only root, ephemeral /tmp
    net: egress allowlist via Cilium NetworkPolicy
    cpu/mem: bounded (250m/512Mi default)
  state:
    short_term: Redis (per-task ctx, TTL 24h)
    long_term: pgvector (semantic memory) + S3 (artifacts)
    coordination: Temporal workflows (durable, replayable)
  observability:
    traces: OTel → Tempo
    logs: structured JSON → Loki, includes correlation_id, agent_id, decision_id
    metrics: Prom (decisions_total, action_latency, approval_rate, rollback_count)
  audit:
    every decision → immutable WORM log (S3 Object Lock, 7-year retention)
    every action → signed by agent identity (Cosign)
  rollback:
    every change carries a reverse-operation; one-click revert via Backstage
  approval:
    blast_radius < LOW → auto
    LOW–MED → human-in-the-loop (Slack interactive)
    HIGH/CRITICAL → CAB (Change Advisory Board) + 2-person rule
  escalation:
    timeout / repeated failure → PagerDuty + auto-pause workflow
```

### 5.4 Agent Runtime Architecture

**Orchestration:** Temporal Cloud (or self-hosted) for durable workflows.
**Eventing:** NATS JetStream (low-latency control plane events) + Kafka (high-throughput data events).
**Memory:** Postgres + pgvector for semantic recall; Redis for hot state.
**Workflow DAGs:** LangGraph for in-agent reasoning; Argo Workflows for cluster-side execution.
**Tracing:** OpenTelemetry (W3C TraceContext propagated across NATS/Kafka).

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                       Agent Control Plane                                │
│                                                                          │
│  ┌──────────┐    ┌─────────────┐    ┌─────────────┐   ┌──────────────┐  │
│  │ Triggers │──▶│ Temporal     │──▶│ Agent Pods   │──▶│ Action Bus    │  │
│  │ (cron/   │    │ Workflow    │    │ (gVisor)     │   │ (NATS/Kafka) │  │
│  │  webhook)│    │ Engine      │    │              │   │              │  │
│  └──────────┘    └─────────────┘    └─────┬───────┘   └──────┬───────┘  │
│                                            │                  │          │
│                          ┌─────────────────┴──────┐           │          │
│                          │ Shared Context Store    │◀──────────┘          │
│                          │ Postgres + pgvector +   │                     │
│                          │ Redis + S3              │                     │
│                          └─────────────────────────┘                     │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Approval Bus → Slack/Teams interactive · CAB UI · auto-policy   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.5 Sequence: Dependency Agent Auto-Patch

```text
CVE Feed       Dep Agent     Temporal      Sandbox Pod    GitHub      Argo CD
   │               │            │              │             │            │
   │── CVE event ─▶│            │              │             │            │
   │               │── start ──▶│              │             │            │
   │               │            │── spawn ────▶│             │            │
   │               │            │              │── PR open──▶│            │
   │               │            │              │             │── PR built │
   │               │            │              │             │── tests ✔  │
   │               │            │◀── result ───│             │            │
   │               │            │── policy check (risk score)│            │
   │               │            │── auto-merge if LOW ──────▶│            │
   │               │            │                                          │
   │               │            │── notify staging deploy ────────────────▶│
   │               │            │── monitor SLO 30 min ───────────────────▶│
   │               │            │── promote to prod (canary)──────────────▶│
   │               │            │── audit log to S3 WORM                   │
```

### 5.6 Failure Recovery

- **Idempotent activities** (Temporal): every step replayable.
- **Compensating actions:** every mutating activity registers an inverse (e.g., create → delete).
- **Circuit breakers:** per agent (max 3 failed actions/15m → pause).
- **Quarantine queue:** failed workflows held for human triage.

### 5.7 Agent Communication Contract

```protobuf
// shared.proto
message AgentDecision {
  string decision_id = 1;          // ULID
  string agent_id = 2;             // spiffe://...
  string correlation_id = 3;
  google.protobuf.Timestamp ts = 4;
  string action = 5;               // verb: create_pr, restart_pod, ...
  string resource = 6;             // urn:k8s:ns/svc, urn:github:repo, ...
  Risk risk = 7;                   // LOW | MED | HIGH | CRITICAL
  bytes payload = 8;               // signed protobuf
  bytes signature = 9;             // cosign signature (DSSE)
  Approval approval = 10;
}
```

Full agent specs in `/app/docs/agents/`.

---

## 6. Security Architecture

### 6.1 Zero-Trust Reference

```text
Principal ──(OIDC/SAML)──▶ IdP (Keycloak) ──▶ STS (short-lived JWT, 5–15 min)
                                                  │
                                                  ▼
            ┌────────── API Gateway (mTLS terminate, JWT verify) ──────┐
            │              ↓ ABAC policy (OPA decision API)            │
            │   Workload ◀──── mTLS (SPIFFE/SPIRE) ────▶ Workload       │
            └──────────── all calls audited, all secrets dynamic ──────┘
```

### 6.2 Identity & Access

| Layer | Mechanism |
|---|---|
| Human SSO | OIDC + SAML via Keycloak; MFA mandatory (WebAuthn preferred, TOTP fallback) |
| User lifecycle | SCIM 2.0 from HRIS (Workday/Okta) → Keycloak → IdP-of-record |
| Workload identity | SPIFFE/SPIRE issuing X.509 SVIDs + JWT SVIDs; cloud-native (IRSA/WIF) |
| RBAC | Kubernetes RBAC + Argo RBAC + Vault policies |
| ABAC | OPA decisions on attributes: `team`, `env`, `data_class`, `geo`, `time_of_day` |
| Session | 8h max human session; 15-min idle; refresh requires re-MFA after 24h |
| Token rotation | Workload tokens TTL ≤ 1h; Vault DB creds TTL ≤ 15 min |
| Break-glass | 2-person rule, time-boxed (≤ 4h), full session recording |
| Audit | Every authN/Z event → SIEM (Splunk/Sentinel) within 60s |

### 6.3 Secrets & Key Management

- **Vault (HA, auto-unseal via KMS)** is the single source of truth.
- **CSI Secrets Store** mounts secrets into pods — never in env vars committed to manifests.
- **External Secrets Operator** syncs Vault → K8s for legacy workloads.
- **KMS hierarchy:** AWS KMS / Azure Key Vault / GCP KMS → Vault Transit → workload data keys.
- **Rotation:** static secrets ≤ 90d; dynamic ≤ 1h; certs auto-rotated by cert-manager (ACME).
- **mTLS:** Istio + SPIRE; cluster-internal PKI rotates every 24h, leaf certs every 1h.

### 6.4 Continuous Security Pipeline

```
PR Open
  ├─ Secret scan (gitleaks, trufflehog)            [BLOCK]
  ├─ SAST (Semgrep, CodeQL)                        [BLOCK on HIGH+]
  ├─ Dependency scan (Snyk/OSV/Grype)              [BLOCK on CRIT]
  ├─ IaC scan (Checkov, tfsec, KICS)               [BLOCK on HIGH+]
  ├─ Dockerfile lint (hadolint)                    [WARN]
  ├─ License scan (FOSSA/ScanCode)                 [BLOCK on banned]
  └─ DCO/CLA check                                 [BLOCK]

Build
  ├─ Reproducible build (BuildKit, --reproducible)
  ├─ SBOM (Syft → SPDX)                            [ATTEST]
  ├─ Provenance (SLSA L3 in-toto)                  [ATTEST]
  ├─ Image scan (Trivy/Grype)                      [BLOCK on CRIT]
  └─ Cosign sign (Fulcio keyless)                  [ATTEST]

Pre-Deploy
  ├─ Cosign verify policy (Kyverno verifyImages)   [BLOCK]
  ├─ SBOM diff vs baseline                         [WARN/BLOCK]
  ├─ DAST (ZAP) against staging                    [BLOCK on HIGH]
  └─ Policy bundle (OPA Conftest)                  [BLOCK]

Runtime
  ├─ Falco rules (syscalls, drift)                 [ALERT/QUARANTINE]
  ├─ Network policies (Cilium default deny)        [BLOCK]
  ├─ Tetragon eBPF observability                   [ALERT]
  └─ KubeArmor LSM enforcement                     [BLOCK]
```

### 6.5 Admission Controllers

- **Kyverno** for image signature verification, label requirements, registry allowlist.
- **OPA/Gatekeeper** for cross-cutting policies (resource limits, hostPath denial, runAsNonRoot).
- **ValidatingAdmissionPolicy (CEL)** for low-latency native checks.

See `/app/docs/security/policies/` for concrete policies.

### 6.6 SLSA L3+ Attainment

| Requirement | Implementation |
|---|---|
| Scripted build | GitHub Actions / GitLab CI / Tekton |
| Build service | Hermetic runners (no network), ephemeral, signed builder identity |
| Isolated, parameterless builds | BuildKit `--no-cache --frozen-lockfile` |
| Provenance generated | `slsa-github-generator` or in-toto attestor |
| Provenance signed | Cosign DSSE → Rekor (transparency log) |
| Non-falsifiable provenance | OIDC ID token from builder, stored in Fulcio |
| Two-party review | CODEOWNERS + branch protection requiring 2 reviewers for `/security/**` |

### 6.7 Runtime Detection & Response

- **Falco** rules: shell in container, privilege escalation, sensitive mount, k8s API anomalies.
- **Tetragon** eBPF-based process & network introspection.
- **Sysdig/Datadog CWPP** optional commercial layer.
- **Auto-response:** quarantine pod (NetworkPolicy deny-all + isolate label) → SOC ticket.

---

## 7. Platform Architecture

### 7.1 C4 — Container View (Logical)

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          ACME Platform                                   │
│                                                                          │
│  ┌─────────────┐   ┌──────────────┐   ┌───────────────┐   ┌──────────┐  │
│  │ Edge / WAF  │──▶│ API Gateway  │──▶│ Service Mesh  │──▶│ Workloads │  │
│  │ Cloudflare  │   │ Kong/Envoy   │   │ Istio (mTLS) │   │ (Pods)    │  │
│  └─────────────┘   └──────┬───────┘   └──────┬───────┘   └─────┬────┘  │
│                            │                  │                 │       │
│                            ▼                  ▼                 ▼       │
│                  ┌─────────────────┐ ┌────────────────┐ ┌──────────────┐│
│                  │ Identity (KC)   │ │ Policy (OPA)   │ │ Data Stores  ││
│                  │ OIDC/SAML/SCIM  │ │ Kyverno/Falco  │ │ Aurora/Mongo ││
│                  └─────────────────┘ └────────────────┘ │ Kafka/Redis  ││
│                                                         └──────────────┘│
│                                                                          │
│  ┌─────────────────────────── Platform Services ────────────────────┐   │
│  │  Backstage IDP · Vault · Harbor · Argo (CD+Workflows+Rollouts)   │   │
│  │  Temporal · NATS · Prometheus · Loki · Tempo · Grafana · Falco   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Multi-Tenancy Model

Three tenancy tiers:

| Tier | Isolation | Example Use |
|---|---|---|
| **Soft (Namespace)** | NS + NetworkPolicy + ResourceQuota + LimitRange | Internal squads |
| **Medium (vCluster)** | vCluster inside host cluster | External customers, dev sandboxes |
| **Hard (Dedicated Cluster)** | Separate EKS/AKS/GKE cluster | Regulated tenants (PCI/HIPAA), gov |

**Data isolation:** per-tenant DB schemas or databases; row-level security where shared; encryption-at-rest with tenant-scoped KMS keys (BYOK supported).

### 7.3 Sharding & Routing

- **Tenant routing key** in JWT (`tid`) → API Gateway routes to shard cluster.
- **Database sharding:** consistent hashing on `tid` for horizontal MongoDB / Vitess for MySQL.
- **Queue topology:** topic-per-tenant for high-volume; shared topic with `tid` partition key otherwise.

### 7.4 API Design

- **External APIs:** REST (OpenAPI 3.1) + GraphQL gateway for aggregation.
- **Internal APIs:** gRPC with mTLS; Protobuf in central schema registry (Buf Schema Registry).
- **Async:** CloudEvents 1.0 over NATS/Kafka; AsyncAPI 3.0 contracts.
- **Webhooks:** signed (HMAC SHA-256), at-least-once delivery, exponential backoff, DLQ.

### 7.5 Caching Strategy

- **Edge:** Cloudflare + CDN, micro-cache 5s for read-heavy public.
- **API:** Redis (managed) per region; cache-aside + write-through patterns; SWR invalidation via Kafka events.
- **DB:** read replicas + materialized views; query plan caching.

---

## 8. Kubernetes & Cloud Design

### 8.1 Cluster Topology

| Cluster | Purpose | Nodegroups |
|---|---|---|
| `mgmt-<region>` | Platform services (Argo, Vault, Backstage, Harbor) | system, platform |
| `prod-<region>` | Production workloads | general, memory-opt, gpu, spot-batch |
| `nonprod-<region>` | Dev/staging/preview | general, spot-heavy |
| `data-<region>` | Stateful (DBs, Kafka) | local-ssd, network-opt |
| `pci-<region>` | PCI-scoped, isolated | hardened-only |

**Cluster mgmt:** Crossplane + Cluster API; bootstrapped via Argo App-of-Apps from `gitops/` repo.

### 8.2 Namespace Strategy

```
<env>-<team>-<service>          # workload ns
platform-system                 # platform components
security-system                 # falco, kyverno, gatekeeper, trivy-operator
observability                   # prometheus, loki, tempo, grafana
istio-system / cert-manager / external-secrets / argocd / vault
tenant-<tid>                    # for soft-tenant model
```

### 8.3 Cilium Default-Deny + Tiered Allow

```yaml
# /app/docs/platform/k8s/cilium-default-deny.yaml
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata: { name: default-deny, namespace: prod-orders-api }
spec:
  endpointSelector: {}
  ingress: []
  egress:
    - toEndpoints: [{ matchLabels: { "k8s:io.kubernetes.pod.namespace": kube-system, "k8s:k8s-app": kube-dns } }]
      toPorts: [{ ports: [{ port: "53", protocol: UDP }] }]
```

### 8.4 Pod Security Standards

- **Baseline** in `nonprod-*`; **Restricted** in `prod-*` and `pci-*`.
- All pods: `runAsNonRoot: true`, `readOnlyRootFilesystem: true`, `seccompProfile: RuntimeDefault`, `capabilities.drop: [ALL]`.

### 8.5 Autoscaling

- **Karpenter** for node provisioning (consolidation, spot mix, AZ spread).
- **HPA v2** on RPS, latency, custom Prom metrics (KEDA for event-driven).
- **VPA** in `Auto` mode for batch, `Off` (recommendation only) for prod online workloads (paired with K8s Optimization Agent PRs).

### 8.6 Multi-Region Active-Active

- **Global LB:** Cloudflare or AWS Global Accelerator (Anycast).
- **Data:** Aurora Global Database / Cosmos DB multi-region writes / MongoDB Atlas global clusters.
- **Mesh:** Istio multi-primary with east-west gateway; SPIRE federated trust domains.
- **GitOps:** ApplicationSet generates per-region apps from one source.

### 8.7 Example: Hardened Deployment

```yaml
# /app/docs/platform/k8s/hardened-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata: { name: orders-api, namespace: prod-orders-api }
spec:
  replicas: 6
  strategy: { type: RollingUpdate, rollingUpdate: { maxSurge: 25%, maxUnavailable: 0 } }
  template:
    metadata:
      labels: { app: orders-api, version: v1.42.0 }
      annotations:
        seccomp.security.alpha.kubernetes.io/pod: runtime/default
    spec:
      automountServiceAccountToken: false
      serviceAccountName: orders-api
      securityContext:
        runAsNonRoot: true
        runAsUser: 65532
        fsGroup: 65532
        seccompProfile: { type: RuntimeDefault }
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: DoNotSchedule
          labelSelector: { matchLabels: { app: orders-api } }
      containers:
        - name: app
          image: registry.acme.io/orders/api@sha256:DEADBEEF...
          imagePullPolicy: IfNotPresent
          ports: [{ containerPort: 8080, name: http }]
          resources:
            requests: { cpu: 200m, memory: 256Mi }
            limits:   { cpu: 1,    memory: 512Mi }
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities: { drop: ["ALL"] }
          livenessProbe:  { httpGet: { path: /healthz, port: http }, periodSeconds: 10 }
          readinessProbe: { httpGet: { path: /ready,   port: http }, periodSeconds: 5  }
          startupProbe:   { httpGet: { path: /healthz, port: http }, failureThreshold: 30, periodSeconds: 2 }
          volumeMounts:
            - { name: tmp, mountPath: /tmp }
            - { name: secrets, mountPath: /var/run/secrets/app, readOnly: true }
      volumes:
        - { name: tmp, emptyDir: { sizeLimit: 64Mi } }
        - name: secrets
          csi:
            driver: secrets-store.csi.k8s.io
            readOnly: true
            volumeAttributes: { secretProviderClass: orders-api-vault }
```

---

## 9. CI/CD & GitOps

### 9.1 Pipeline Topology

```
   Dev push ─▶ PR ─▶ CI (build/test/scan/sign) ─▶ Registry ─▶ GitOps repo bump
                                                                    │
                                       Argo CD detect change ◀──────┘
                                       │
                       ┌───────────────┼───────────────┐
                       ▼               ▼               ▼
                  dev cluster    staging cluster   prod cluster
                  auto-sync      auto-sync         manual-promote + canary
```

### 9.2 Pipeline Phases

1. **Source:** PR open, OIDC identity, signed commits (sigstore gitsign).
2. **Lint & Static:** ESLint/Ruff/golangci, Semgrep, Checkov, hadolint.
3. **Build:** BuildKit, multi-arch, remote cache (S3/GCS), reproducible.
4. **Test:** unit, integration, contract (Pact), mutation (PIT/Stryker), perf smoke (k6).
5. **Scan:** Trivy, Grype, Snyk, OSV-scanner.
6. **Attest:** Syft SBOM + SLSA provenance; Cosign sign; push to Rekor.
7. **Publish:** push to Harbor → mirror to ECR/ACR/GAR.
8. **Verify:** Kyverno admission verifies signatures + SBOM presence.
9. **Deploy:** Argo CD ApplicationSet → cluster → progressive delivery.
10. **Promote:** Argo Rollouts canary with SLO analysis (Flagger-style).

### 9.3 Progressive Delivery

```yaml
# /app/docs/cicd/argo/rollout-canary.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata: { name: orders-api }
spec:
  strategy:
    canary:
      maxSurge: 25%
      maxUnavailable: 0
      analysis:
        templates: [{ templateName: success-rate }]
        args: [{ name: service-name, value: orders-api }]
      steps:
        - setWeight: 5
        - pause: { duration: 10m }
        - setWeight: 25
        - pause: { duration: 15m }
        - setWeight: 50
        - pause: { duration: 30m }
        - setWeight: 100
```

### 9.4 Preview & Ephemeral Environments

- Every PR → `pr-<num>` namespace in `nonprod` cluster, TTL 72h.
- Backed by Argo CD `ApplicationSet` PR generator.
- Seeded data from anonymized prod snapshot (data-masking pipeline).

### 9.5 Multi-Vendor Support

Pipeline templates exist as reusable workflows:
- **GitHub:** `.github/workflows/_reusable-build.yml`
- **GitLab:** `.gitlab/ci/templates/build.yml`
- **Azure DevOps:** `azure-pipelines/templates/build.yml`
- **Jenkins:** `shared-library/vars/standardBuild.groovy`
- **Tekton:** `tekton/pipelines/standard-build.yaml`

Concrete files in `/app/docs/cicd/`.

---

## 10. Supply Chain Security

### 10.1 Threat Model (SLSA-aligned)

| Threat | Control |
|---|---|
| Source tampering | Signed commits (gitsign), CODEOWNERS, branch protection, mandatory reviews |
| Dependency confusion | Internal registry mirror, scoped packages, lockfile + checksum pinning |
| Build tampering | Hermetic builds, ephemeral runners, signed build identity (OIDC) |
| Artifact tampering | Cosign signature, Rekor transparency log, Kyverno verify on admission |
| Provenance forgery | SLSA L3 in-toto attestation signed by builder OIDC identity |
| Registry compromise | Replication + Cosign verify at pull, content trust enabled |
| Runtime injection | Read-only FS, Falco syscall rules, drift detection |

### 10.2 Attestation Stack

- **Identity:** Sigstore Fulcio (OIDC → short-lived cert).
- **Signing:** Cosign DSSE.
- **Transparency:** Rekor immutable log.
- **Policy:** Kyverno `verifyImages` rules require ≥ 2 attestations: `slsa-provenance` + `cyclonedx-sbom`.

### 10.3 Kyverno verifyImages Example

```yaml
# /app/docs/security/admission/kyverno-verify-images.yaml
apiVersion: kyverno.io/v2beta1
kind: ClusterPolicy
metadata: { name: verify-images }
spec:
  validationFailureAction: Enforce
  webhookTimeoutSeconds: 30
  rules:
    - name: require-signature-and-attestations
      match: { any: [{ resources: { kinds: [Pod] } }] }
      verifyImages:
        - imageReferences: ["registry.acme.io/*"]
          attestors:
            - entries:
                - keyless:
                    subject: "https://github.com/acme/*"
                    issuer:  "https://token.actions.githubusercontent.com"
          attestations:
            - type: https://slsa.dev/provenance/v1
              conditions:
                - all:
                    - key: "{{ buildDefinition.buildType }}"
                      operator: Equals
                      value: "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1"
            - type: https://cyclonedx.org/bom
```

### 10.4 Air-Gapped Supply Chain

- Approved artifacts are exported via `cosign save` to a bundle tarball.
- Bundle transferred through approved one-way diode → loaded via `cosign load` into air-gapped Harbor.
- Rekor mirror runs internally; offline verification keys distributed via HSM-rooted PKI.

---

## 11. Observability Strategy

### 11.1 Three Pillars + Events + Profiles

| Pillar | Tool | Retention |
|---|---|---|
| Metrics | Prometheus (Mimir for long-term) | 13 months |
| Logs | Loki (S3 backing) | 90d hot, 7y cold (compliance) |
| Traces | Tempo | 7d hot, 30d sampled |
| Profiles (continuous) | Pyroscope/Parca | 30d |
| Events | NATS → Loki + S3 | 7y |

All instrumentation through **OpenTelemetry SDK + Collector** (no vendor lock-in).

### 11.2 SLO Catalog (sample)

| Service | SLI | SLO | Error Budget |
|---|---|---|---|
| orders-api | success-rate (2xx/3xx ÷ total) over 30d | 99.95% | 21.6 min/30d |
| orders-api | p99 latency over 30d | < 250 ms | 1% violations |
| payment-svc | success-rate | 99.99% (PCI) | 4.32 min/30d |
| login | availability | 99.99% | 4.32 min/30d |

### 11.3 SLOs as Code

```yaml
# /app/docs/observability/prometheus/slo-orders-api.yaml
apiVersion: sloth.slok.dev/v1
kind: PrometheusServiceLevel
metadata: { name: orders-api, namespace: prod-orders-api }
spec:
  service: orders-api
  slos:
    - name: requests-availability
      objective: 99.95
      sli:
        events:
          error_query:  'sum(rate(http_requests_total{job="orders-api",code=~"5.."}[5m]))'
          total_query:  'sum(rate(http_requests_total{job="orders-api"}[5m]))'
      alerting:
        page_alert:   { labels: { severity: page,   team: orders } }
        ticket_alert: { labels: { severity: ticket, team: orders } }
```

### 11.4 Auto-Remediation

- **Prometheus AlertManager** → **Argo Events** → **Argo Workflows** runbook execution.
- Common auto-remediations: pod restart, HPA bump, drain & cordon node, switch to read-replica.
- Every auto-remediation logged to incident timeline + Slack thread.

### 11.5 Runbook Standard

Every alert annotation must include:
- `runbook_url` (Backstage TechDocs link)
- `dashboard_url`
- `severity` (page | ticket | info)
- `summary` and `description` with templated values

---

## 12. Enterprise Integrations

### 12.1 Integration Matrix

| System | Direction | Protocol | Auth | Purpose |
|---|---|---|---|---|
| Jira | bi | REST + webhooks | OAuth2 | Incident & change tickets, agent-created issues |
| Slack | bi | Web API + Events | OAuth2 + signing secret | Notifications, agent approvals |
| MS Teams | bi | Graph + webhooks | App registration | Notifications, approvals |
| GitHub Enterprise | bi | REST + GraphQL + webhooks | GitHub App | PRs, status checks, code search |
| GitLab | bi | REST + webhooks | Group access token | PRs, pipelines |
| ServiceNow | bi | Table API + webhooks | OAuth2 | Change records, CMDB |
| PagerDuty | bi | Events API v2 + webhooks | Routing key + token | Paging, on-call |
| Datadog | out | Metrics/Logs/Events API | API key (Vault) | Dual-write for legacy dashboards |
| Splunk | out | HEC | HEC token (Vault) | Compliance SIEM |

### 12.2 Standards

- **Outbound webhooks** signed with HMAC-SHA-256, header `X-Signature: t=<unix>,v1=<hex>`.
- **Inbound webhooks** validated by gateway (origin IP + signature + replay protection: ±5 min skew, nonce cache 10 min).
- **Retries:** exponential backoff (base 2s, cap 5 min, jitter), max 12 attempts → DLQ.
- **Rate limiting:** token bucket per integration; tenant-scoped quotas.
- **CloudEvents 1.0** schema for internal event bus; per-integration AsyncAPI 3.0 contracts.

### 12.3 Sample Event Schema

```json
{
  "specversion": "1.0",
  "id": "01HX...ULID",
  "source": "/platform/deployment-agent",
  "type": "com.acme.deployment.promoted.v1",
  "subject": "orders-api@v1.42.0",
  "time": "2026-01-15T14:32:00Z",
  "datacontenttype": "application/json",
  "tenantid": "acme-prod",
  "traceparent": "00-...-...-01",
  "data": {
    "service": "orders-api",
    "from_version": "v1.41.3",
    "to_version": "v1.42.0",
    "strategy": "canary",
    "approver": "spiffe://platform.acme.io/agents/deployment/v3"
  }
}
```

---

## 13. Governance & Compliance

### 13.1 Frameworks & Mapping

| Framework | Year-1 | Year-2 | Year-3 |
|---|---|---|---|
| SOC 2 Type II | ✅ | maintain | maintain |
| ISO 27001 | ✅ | maintain | maintain |
| GDPR | ✅ | ✅ | ✅ |
| PCI-DSS v4.0 | scope-out | ✅ | maintain |
| HIPAA | scope-out | ✅ | maintain |
| FedRAMP Moderate | readiness | readiness | ✅ ATO |

### 13.2 Control Mapping (excerpt)

| Control | SOC2 CC | ISO 27001 | PCI | HIPAA | Implementation |
|---|---|---|---|---|---|
| Access reviews quarterly | CC6.1 | A.9.2.5 | 7.2 | §164.308(a)(4) | Keycloak + SCIM + Drata workflow |
| Encryption in transit | CC6.7 | A.10.1 | 4.1 | §164.312(e) | Istio mTLS, TLS 1.3 only |
| Encryption at rest | CC6.7 | A.10.1 | 3.5 | §164.312(a) | KMS-backed disks, app-level for PII |
| Vulnerability mgmt | CC7.1 | A.12.6 | 6.3 | §164.308(a)(5) | Trivy + Grype + auto-PR agent |
| Logging & monitoring | CC7.2 | A.12.4 | 10 | §164.308(a)(1) | Loki + Splunk SIEM, 7y retention |
| Change management | CC8.1 | A.12.1 | 6.4 | §164.308(a)(8) | GitOps, signed commits, CAB on HIGH |
| Incident response | CC7.4 | A.16 | 12.10 | §164.308(a)(6) | PagerDuty + Incident Agent + RCA |
| Data classification | CC6.1 | A.8.2 | 3.1 | §164.514 | Tag-based ABAC, OPA enforcement |

Full mapping in `/app/docs/compliance/control-mapping.md`.

### 13.3 Continuous Evidence

- **Drata / Vanta / Secureframe connectors** pull from K8s, IdP, Vault, GitHub, AWS, MDM.
- **Policy as code** (OPA bundles) — every policy = control evidence.
- **Audit log lake** in S3 (Object Lock, 7y). Immutable.
- **Quarterly access reviews** auto-generated from SCIM + RBAC dumps.
- **Separation of duties** enforced by Kyverno: same identity cannot author + approve + deploy.

### 13.4 Data Residency & GDPR

- **Tenant geo-pinning** via tenant metadata (`region: eu-west-1`).
- **Data subject requests (DSAR)** workflow: 30-day SLA, automated via Backstage scaffold + agent.
- **DPIA template** in `/app/docs/compliance/dpia-template.md`.

---

## 14. Scalability Strategy

| Dimension | Approach |
|---|---|
| Compute | Karpenter consolidation, spot mix 70/30, scale-to-zero for idle envs |
| Stateless services | HPA on RPS/latency + KEDA on queue depth |
| Stateful | Read replicas, sharding (consistent hashing), Vitess/Citus for SQL |
| Cache | Tiered (CDN → Redis → in-memory), SWR pattern, event-driven invalidation |
| Queue | Kafka partitions ≥ peak concurrent consumers × 2; NATS JetStream for low-latency |
| Hot tenants | Dedicated shard + isolated namespace promotion path |
| Global latency | Anycast edge + regional read endpoints; write-forward to home region |
| Backpressure | Token bucket at gateway, circuit breakers (Envoy), bulkheads (per-tenant quotas) |
| Load tests | k6 + Locust nightly against staging; gating with SLO drift alerts |

---

## 15. Reliability Engineering

- **Error budgets** computed per service from SLOs (Sloth-generated rules).
- **Burn-rate alerts** at 2%, 5%, 10% / 1h, 6h, 3d windows.
- **Chaos engineering** monthly via LitmusChaos / Chaos Mesh (game days).
- **Resilience patterns:** retries with jitter, circuit breakers, bulkheads, timeouts, idempotency keys on writes.
- **Capacity planning** quarterly: forecast on growth curve + headroom 30%.
- **On-call:** 24×7 follow-the-sun, primary + secondary, PagerDuty escalation policy, max 2 pages/shift target.

---

## 16. Disaster Recovery

| Tier | RTO | RPO | Strategy |
|---|---|---|---|
| Tier 1 (revenue critical) | 5 min | 0 (sync) | Active-active multi-region, sync replication, anycast failover |
| Tier 2 | 1 h | 5 min | Active-passive, async replication, automated failover |
| Tier 3 | 4 h | 1 h | Warm standby, scheduled snapshots, scripted restore |
| Tier 4 (batch) | 24 h | 24 h | Cold backups, S3 cross-region replication |

- **DR drills:** quarterly per tier, fully recorded, evidence stored.
- **Backup tooling:** Velero (K8s objects + PV snapshots) + native DB tooling + AWS Backup / Azure Backup.
- **Immutable backups:** S3 Object Lock (Compliance mode), separate AWS account, separate KMS key.
- **Runbooks:** every DR scenario has a tested runbook; auto-remediation where safe.

---

## 17. Cost Optimization

- **FinOps capability** with Backstage cost plugin per service.
- **Karpenter** consolidation + spot for stateless workloads.
- **Right-sizing agent** generates VPA-driven request PRs weekly.
- **Idle env reaper:** nonprod namespaces with no traffic 48h → scale to zero.
- **Storage lifecycle:** Loki/Tempo tiered S3 (Standard → IA → Glacier); RDS snapshot pruning.
- **Reserved/Savings Plans** for baseline; spot for elastic; on-demand for spikes.
- **Showback & chargeback** dashboards by team/tenant.
- **Egress optimization:** VPC endpoints, region affinity, CDN offload.
- **Target:** 25–35% reduction year-over-year, with auto-tracked KPIs.

---

## 18. Operational Excellence

### 18.1 Internal Developer Platform (IDP)

**Backstage** as the single pane of glass:
- Service catalog (auto-discovered via GitOps).
- Software templates (scaffolders for new microservices, all hardened by default).
- TechDocs (every repo ships docs).
- Cost, SLO, security, compliance, deployments tabs per service.

### 18.2 Golden Paths

Pre-paved paths for:
- New microservice (any language) → repo + CI + Helm chart + dashboards + on-call.
- New tenant onboarding → namespace + DB + secrets + observability.
- New cloud region → cluster + mesh + GitOps app-of-apps.

### 18.3 Documentation Standards

- **ADRs** in every repo (`docs/adr/`).
- **Runbooks** linked from alerts.
- **API docs** auto-published to internal portal from OpenAPI/AsyncAPI.
- **Postmortems** within 5 business days, blameless template.

### 18.4 Quality Gates

- Coverage ≥ 80% (unit), 60% (integration), mutation score ≥ 60%.
- DORA metrics tracked: deployment frequency, lead time, MTTR, CFR.
- Targets: Elite tier per DORA report.

---

## 19. Implementation Roadmap

### 19.1 Phase 0 — Discovery & Bootstrap (Weeks 1–4)

- Stand up GitOps repos, Argo CD, Vault, Harbor, Backstage skeleton.
- Choose CI vendor & set baseline reusable workflows.
- Draft RBAC + IdP integration.
- Quick wins: hadolint, secret scan, branch protection.

### 19.2 Phase 1 — Foundation (Months 2–4)

- Hardened base + 4 priority runtimes (Node, Python, Go, Java).
- Cosign + SBOM + Kyverno verifyImages in non-prod.
- Prometheus + Loki + Tempo + Grafana baseline.
- Single-region multi-AZ EKS prod cluster.
- SOC 2 readiness assessment.

### 19.3 Phase 2 — Industrialization (Months 5–8)

- All 8 runtimes + tooling images live.
- Multi-region active-passive.
- Service mesh (Istio) with mTLS.
- 5 priority agents live (Security, Dep, Deployment, Incident, Cost).
- Progressive delivery via Argo Rollouts.
- SOC 2 Type II audit window starts.
- ISO 27001 Stage 1.

### 19.4 Phase 3 — Autonomy & Scale (Months 9–12)

- Remaining 10 agents live with HITL approvals.
- Active-active multi-region.
- vCluster medium-tenancy GA.
- FedRAMP Moderate readiness package.
- DR drills quarterly.
- Cost optimization KPIs hit.

### 19.5 Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Image rebuild churn (CVE noise) | H | M | Auto-PR agent, severity SLA, batching |
| Mesh complexity | M | H | Istio-managed (Anthos/Tetrate) option, training, golden config |
| Agent runaway action | M | H | Sandboxing, blast radius caps, kill-switch, audit + WORM |
| Compliance scope creep | M | M | Quarterly compliance roadmap reviews |
| Cloud egress costs | M | M | VPC endpoints, region affinity, monitoring |
| Skills gap | H | M | Platform engineering enablement, paved roads, Backstage docs |

### 19.6 Migration Strategy

Strangler-fig pattern: existing monolith fronted by API Gateway; new capabilities built on the platform; routes migrated piecemeal. Every cut-over has a tested rollback (DNS TTL ≤ 60s + feature flag).

---

## 20. Appendix

- **A.1** Dockerfile templates — `/app/docs/runtime-images/runtimes/`
- **A.2** Tooling images — `/app/docs/runtime-images/tooling/`
- **A.3** Agent specifications — `/app/docs/agents/`
- **A.4** Security policies — `/app/docs/security/policies/` and `/admission/`
- **A.5** Kubernetes manifests — `/app/docs/platform/k8s/`
- **A.6** CI/CD pipelines — `/app/docs/cicd/`
- **A.7** Observability configs — `/app/docs/observability/`
- **A.8** Integration contracts — `/app/docs/integrations/`
- **A.9** Compliance mapping — `/app/docs/compliance/`
- **A.10** Roadmap detail — `/app/docs/roadmap/`
- **A.11** Glossary & references — `/app/docs/appendix/glossary.md`

### 20.1 Reference Standards

- SLSA v1.0 — https://slsa.dev
- in-toto Attestation Framework
- NIST SP 800-218 (SSDF), 800-190 (Containers), 800-207 (Zero Trust)
- CIS Kubernetes Benchmark v1.9
- OWASP ASVS 4.0, Top 10 for LLMs
- CNCF Cloud Native Trail Map
- Google SRE Workbook, Platform Engineering Topologies

---

*End of master document. Continue to the per-section artifacts under `/app/docs/`.*
