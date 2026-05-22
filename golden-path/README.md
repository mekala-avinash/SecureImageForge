# The Paved Road

> A "golden path" / "paved road" is the **opinionated, supported, best-practice route** through the platform. Off-road is allowed for legitimate reasons, but on-road is faster, safer, and free.

## Architecture rationale

The paved road exists because:

1. **Repetition is the enemy of security.** If every team writes their own Dockerfile, mTLS config, and CI pipeline, the security posture is the worst of any individual choice. A paved road inverts this: the best implementation becomes the easiest.
2. **Cognitive load is the limiting factor in delivery.** Engineers' time is better spent on business logic than on YAML.
3. **Compliance is a byproduct of how you build, not a separate audit scramble.** Bake controls into the road; evidence collects itself.

## Principles

| Principle | Practical consequence |
|---|---|
| Composition over duplication | One Helm library chart; service charts compose it. |
| Secure by default | Pod security restricted, signed images, mTLS, read-only FS — without ceremony. |
| Multi-arch native | `linux/amd64` + `linux/arm64` mandatory; opt-in for more. |
| GitOps-only deploy | Argo CD reconciles; no imperative changes. |
| Observable from line 1 | OTel SDK auto-instrumentation before user code lands. |
| Reversible | Every change has a rollback path. Image digests + Argo revert + Terraform state. |
| Vendor portable | CNCF-native — never one acquisition away from re-platforming. |

## What "on the paved road" guarantees

Any service that uses the paved road end-to-end gets:

- ✅ Hardened distroless image, ≤ 100 MB runtime, multi-arch.
- ✅ Image signed by Cosign keyless (Fulcio OIDC) → Rekor.
- ✅ SBOM (SPDX + CycloneDX) attached as OCI artifact.
- ✅ SLSA L3 in-toto provenance.
- ✅ Kyverno verify-images admission policy passes.
- ✅ Cilium default-deny + service-scoped allow.
- ✅ Istio STRICT mTLS.
- ✅ Pod Security Restricted profile.
- ✅ OTel traces + metrics + logs flowing to Tempo/Prom/Loki.
- ✅ SLOs as code (Sloth-style PrometheusServiceLevel).
- ✅ Multi-window multi-burn-rate alerts with runbook URLs.
- ✅ Auto-scaling (HPA on CPU/RPS, KEDA optional).
- ✅ Pod Disruption Budget.
- ✅ Zone + host topology spread.
- ✅ GitOps deployment via Argo CD ApplicationSet.
- ✅ Progressive delivery (Argo Rollouts canary) with SLO analysis.
- ✅ Vault dynamic secrets (CSI Secrets Store).
- ✅ Backstage catalog entry + TechDocs.
- ✅ Runbook scaffold + on-call routing.
- ✅ Continuous compliance evidence (Drata controls satisfied automatically).

## What's off-road (and why you'd want it)

| Off-road choice | When valid | What you give up |
|---|---|---|
| Custom base image (not from `images/runtimes/`) | Hardware-specific native deps not in Wolfi | CVE rebuild automation, base-image SLA |
| Custom CI workflow (not from `ci/`) | Toolchain genuinely not covered | Auto-sign, auto-SBOM, auto-SLSA, auto-GitOps PR |
| Imperative `kubectl apply` | Emergency only (break-glass) | Auditability, repeatability — opens an incident |
| Self-managed Postgres on a Pod | Almost never | Operational maturity of managed RDS/Aurora |

Off-road requires a waiver (see `/app/docs/phase-1/01-hardened-runtimes/waiver-process.md`).

## Adoption flow (read next)

[`ADOPTION_WORKFLOW.md`](./ADOPTION_WORKFLOW.md) — how a team migrates onto the paved road in 1–3 days.

## Composition diagram

```
                            ┌────────────────────────────┐
                            │   developer-experience/    │
                            │   pavedroad CLI            │
                            │   `pavedroad new service`  │
                            └────────────┬───────────────┘
                                         │ scaffolds
                                         ▼
       ┌──────────────────────────────────────────────────┐
       │                templates/                         │
       │   Backstage scaffolder + service skeleton         │
       └────────────┬─────────────────────┬───────────────┘
                    │                     │
                    │ uses                │ uses
                    ▼                     ▼
       ┌────────────────────┐    ┌────────────────────────┐
       │   images/runtimes  │    │   ci/github-actions    │
       │   hardened base    │    │   reusable workflow    │
       └─────────┬──────────┘    └──────────┬─────────────┘
                 │                          │
                 │ extends                  │ produces signed image
                 │                          │
                 ▼                          ▼
       ┌────────────────────────────────────────────────┐
       │              platform/helm-library             │
       │  Deployment, Service, HPA, PDB, Rollout,       │
       │  ServiceMonitor, PrometheusRule, etc.          │
       └────────────────────┬───────────────────────────┘
                            │ rendered by ArgoCD
                            ▼
       ┌────────────────────────────────────────────────┐
       │  infra/ Terraform: EKS + IRSA + KMS + Argo CD  │
       │  observability/ + security/ deployed in cluster │
       └────────────────────────────────────────────────┘
```
