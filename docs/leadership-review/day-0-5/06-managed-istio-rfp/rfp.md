# Managed Istio — Request for Proposal

> Action #6 in Day 0–5 · Owner: Platform Lead · Co-owner: Head of SRE · Target close: 6 weeks from kickoff

## 1. Background
ACME is standing up a multi-region (us-east-1, eu-west-1, ap-south-1) Kubernetes platform with Istio as the service mesh providing mTLS, AuthorizationPolicy, and progressive delivery integration (Argo Rollouts). We are evaluating **managed Istio** offerings to reduce operational burden, accelerate the multi-region cutover, and benefit from vendor-driven upgrade paths and CVE response.

## 2. Strategic options

| Option | Vendor | Strengths | Watch-outs |
|---|---|---|---|
| Tetrate Service Bridge | Tetrate | Multi-cluster, multi-tenant, FIPS variant, strong support | Per-workload pricing |
| Anthos Service Mesh | Google Cloud | Tight GCP integration; GA across clouds | Best fit if GCP-first |
| Solo.io Gloo Mesh | Solo.io | Strong UX + east-west gateway tooling | Some features tied to Gloo Gateway |
| Self-managed Istio | (none) | Full control; CNCF-native | Operational cost; CVE response burden |

Default position: managed (one of the three commercial options). Self-managed only if managed options materially fail evaluation.

## 3. Required capabilities

### 3.1 Mesh
- Istio version ≥ 1.23 supported within 60 days of upstream release.
- Multi-primary topology across ≥ 3 regions with SPIFFE-federated trust domains.
- East-west gateway with mTLS-only (no plaintext fallback).
- ZTLS / strict mTLS by default; opt-in PERMISSIVE only.

### 3.2 Identity & policy
- AuthorizationPolicy + RequestAuthentication with JWKS from Keycloak.
- Workload identity bridged to SPIRE / IRSA / WIF.
- Cross-cluster identity propagation without shared secrets.

### 3.3 Progressive delivery
- First-class Argo Rollouts integration (canary VirtualService traffic split).
- Header-based / cohort-based routing.
- Native fault injection + traffic mirroring.

### 3.4 Observability
- OTel-native metrics + tracing + access logs (no proprietary agents required).
- Service topology + L7 RED metrics dashboards.

### 3.5 Lifecycle
- Zero-downtime control-plane upgrades.
- CVE response SLA ≤ 72h for HIGH/CRITICAL.
- Air-gapped install variant (gov tenants).
- FIPS-validated build available (path to FedRAMP).

### 3.6 Operability
- Helm + GitOps friendly (no UI-only configuration).
- Full configuration reflected in Git.
- API + CRDs match upstream Istio (no vendor lock-in for spec).

### 3.7 Support
- 24×7 with ≤ 1h response on SEV1.
- Named TAM.
- Quarterly architecture reviews.

## 4. Evaluation criteria (weighted)

| Criterion | Weight |
|---|---|
| Multi-region / multi-primary maturity | 20% |
| Argo Rollouts + canary integration | 15% |
| FIPS / FedRAMP path | 10% |
| Air-gapped support | 10% |
| Operability (GitOps, CRDs, no UI-only) | 10% |
| OTel-native observability | 10% |
| Lifecycle (upgrades, CVE SLA) | 10% |
| Pricing transparency + 3-year TCO | 10% |
| Reference customers (multi-region) | 5% |

## 5. Required deliverables from vendor
- Reference architecture for our 3-region topology.
- POC plan (4–6 weeks) covering: multi-primary install, AuthorizationPolicy, Argo Rollouts canary, OTel pipeline.
- CVE response process documented.
- Pricing for: 50, 200, 1000 workloads (Y1/Y2/Y3).
- 3 reference customers, at least 1 in regulated industry.
- Air-gapped install documentation.

## 6. POC acceptance criteria
- 3-region active-active mesh installed via Git only.
- AuthorizationPolicy enforced with Keycloak-issued JWTs.
- Argo Rollouts canary completes with SLO analysis from Prometheus.
- Failover drill: drain 1 region; traffic moves to remaining 2 with ≤ 5 min RTO.
- All telemetry available via OTel collector (no vendor-specific agents).

## 7. Timeline
- Week 1: issue RFP to Tetrate, Solo.io, Google (Anthos), and capture self-managed baseline.
- Week 2: vendor responses due.
- Week 3–4: scoring + reference calls.
- Week 5–6: POC with top 1–2.
- End of Week 6: recommendation to CTO + VP Platform; contract decision.

## 8. Stakeholders
- Decision: CTO + VP Platform + CFO.
- Engineering owners: Platform Lead, Head of SRE, Staff DevSecOps.
- Procurement: <Procurement lead>.

## 9. Out of scope
- API gateway selection (separate decision: Kong vs Envoy Gateway).
- Ingress controller (will align with whichever mesh wins).
- Linkerd evaluation — explicitly deferred; our roadmap depends on Istio AuthorizationPolicy semantics.
