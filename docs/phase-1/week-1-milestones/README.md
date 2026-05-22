# Phase 1 — Week 1 Milestone Tracker

> **Window:** 2026-02-09 → 2026-02-13 (Mon → Fri)
> **Source of truth:** This file (committed). Synced to Backstage delivery board.
> **Standup:** daily 09:30 (15 min, workstream leads only).
> **Weekly working session:** Wednesday 14:00–15:30 (see `working-sessions/`).

## Cross-workstream coordination
- Drata production tenancy is a **shared dependency**. Compliance PM unblocks Workstream 2 + 3 + 4.
- Tetrate POC scoping conversation continues; not on critical path for Week 1.
- All Week 1 deliverables PR-merged through `acme/gitops` or `acme/platform`.

---

## Workstream 1 — Hardened Runtimes (Lead: Platform Lead)

| # | Deliverable | Owner | Done? | Notes |
|---|---|---|---|---|
| W1.1 | Cut `runtime/nodejs@20-v1.0.0`, `runtime/python@3.12-v1.0.0`, `runtime/go@1.23-v1.0.0` images from `docs/runtime-images/runtimes/Dockerfile.*.template` | Platform | ☐ | Push to Harbor `registry.acme.io/runtime/*` |
| W1.2 | Cosign sign + SBOM attest + SLSA provenance for the 3 base runtimes | DevSecOps | ☐ | Verified by Kyverno cluster policy |
| W1.3 | Publish Backstage scaffolder v1 for Python + Go (Node v1.1 next week — carryover from P0 condition C-01) | DevEx | ☐ | Scaffolder emits hardened image + GH workflow + Helm chart |
| W1.4 | Pair migration kickoff with `orders-api` team (lighthouse Tier-A #1) | Platform + Orders team | ☐ | 2-hour pairing session Thursday |
| W1.5 | Adoption tracker spreadsheet live + linked from Backstage | Platform | ☐ | |

## Workstream 2 — Observability (Lead: Head of SRE)

| # | Deliverable | Owner | Done? | Notes |
|---|---|---|---|---|
| W2.1 | Vendor `kube-prometheus-stack` + `loki` + `tempo` Helm charts under `gitops/platform/observability/` with values from `docs/phase-1/02-observability/*-values.yaml` | SRE + Platform | ☐ | App-of-Apps wave −20 |
| W2.2 | OTel Operator + auto-instrumentation Custom Resource | SRE | ☐ | |
| W2.3 | Grafana via `grafana-operator`; OIDC via Keycloak | SRE + Identity | ☐ | |
| W2.4 | First service emitting OTel traces end-to-end (`hello-platform` pilot) | SRE + Platform | ☐ | Visible in Tempo by EOD Thursday |
| W2.5 | Onboarding doc + example PR in `docs/phase-1/02-observability/` | SRE | ☐ | Used by Wednesday working session |

## Workstream 3 — Single-Region Prod Cluster (Lead: Cloud Infra Lead + Platform Lead)

| # | Deliverable | Owner | Done? | Notes |
|---|---|---|---|---|
| W3.1 | AWS account `acme-prod-use1` vended via AFT/Control Tower; org policies attached | Cloud Infra | ☐ | Pre-condition met if already vended in P0 |
| W3.2 | `terraform plan` for `eks-terraform-module.tf` succeeds in `acme-prod-use1`; reviewed | Cloud Infra | ☐ | No apply yet |
| W3.3 | KMS CMK + S3 buckets for Loki/Tempo + Velero pre-created (via separate terraform) | Cloud Infra | ☐ | |
| W3.4 | Karpenter EC2NodeClass IAM role + Pod Identity association validated in plan | Cloud Infra | ☐ | |
| W3.5 | Architecture review with CISO sign-off on the blueprint | Platform Lead + CISO | ☐ | |

## Workstream 4 — SOC 2 Type II Prep (Lead: Compliance PM)

| # | Deliverable | Owner | Done? | Notes |
|---|---|---|---|---|
| W4.1 | Drata moved from pilot to **production tenancy** | Compliance PM | ☐ | Re-onboards connectors |
| W4.2 | Connectors live: K8s (mgmt-use1), GitHub Org, Keycloak, Vault, AWS Org, JAMF | Compliance PM + IT | ☐ | 6 connectors |
| W4.3 | Control owners assigned for all CC + A + C controls | Compliance PM | ☐ | Owner map in Drata + control-test-plan.md |
| W4.4 | Audit-window plan socialized at SteerCo | Compliance PM | ☐ | Friday SteerCo standing meeting |
| W4.5 | Auditor RFP shortlist drafted (Coalfire / Schellman / A-LIGN / Prescient) | Compliance PM | ☐ | Engagement letter signed by Week 5 |

---

## Daily standup format (15 min, async pre-write in #platform-transformation)

For each lead, post by 09:25:

```
:workstream: <name>
Yesterday: <1–3 bullets>
Today:     <1–3 bullets>
Blockers:  <none | <name them>>
Risk movement: <none | description>
```

Standup is for **synchronization, not status**. If a workstream needs > 2 min of group time, it goes to the Wednesday working session.

## Friday wrap (SteerCo #2)

End-of-week status against this tracker is reported in SteerCo #2 (2026-02-13). Format:

```
W1 Hardened Runtimes:   X/5 done · 1 blocker · risk movement: none
W2 Observability:       X/5 done · 0 blockers · risk: P1-R2 → L
W3 Prod cluster:        X/5 done · 0 blockers
W4 SOC 2:               X/5 done · 0 blockers · note: auditor shortlist ready Mon
```
