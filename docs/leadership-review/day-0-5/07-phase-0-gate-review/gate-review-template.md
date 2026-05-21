# Phase 0 → Phase 1 Gate Review Template

> Reusable template — also serves as the structure for Phase 1→2, 2→3, and final close-out reviews.

## 1. Meta
- **Phase reviewed:** 0 (Bootstrap)
- **Date:** ___________
- **Gate-keeper:** VP Platform Engineering
- **Sponsor present:** CTO ☐
- **Quorum (5/7):** ☐

## 2. Exit-criteria checklist (binding)

| # | Criterion | Evidence link | Verified? |
|---|---|---|---|
| 1 | Pilot service builds → scans → signs → deploys via GitOps end-to-end | <CI link> + <Argo CD link> | ☐ |
| 2 | `acme/gitops`, `acme/platform`, `acme/tenants` repos created with CODEOWNERS, branch protection, signed-commit requirement | <gh links> | ☐ |
| 3 | Vault HA + KMS auto-unseal operational; first secret consumed by pilot service via CSI | <runbook + grafana> | ☐ |
| 4 | Harbor registry operational; first signed image pushed and verified by Cosign | <harbor link + cosign output> | ☐ |
| 5 | Backstage skeleton deployed; service catalog auto-discovery working for ≥ 5 services | <backstage URL> | ☐ |
| 6 | Quick wins live on 100% of repos: signed commits, secret scan, hadolint, semgrep baseline, branch protection | <org policy screenshot> | ☐ |
| 7 | Reusable build workflow used by ≥ 3 pilot services | <action links> | ☐ |
| 8 | Steering committee operational; minutes published for all sessions | <links> | ☐ |
| 9 | Hiring funnel: 4 reqs open; ≥ 6 candidates in-stage | <ATS export> | ☐ |
| 10 | Compliance platform pilot kicked off | <Drata/Vanta link> | ☐ |
| 11 | Managed-Istio RFP issued; ≥ 2 vendor responses received | <RFP tracker> | ☐ |

## 3. Status by workstream

| Workstream | RAG | Notes |
|---|---|---|
| GitOps & repos | 🟢/🟡/🔴 | |
| Vault | | |
| Harbor + Cosign | | |
| Backstage skeleton | | |
| CI baseline | | |
| Quick wins (signed commits etc.) | | |
| Hiring | | |
| Compliance platform | | |
| Managed-Istio RFP | | |

## 4. Risk register diff
- New risks (since last review): ___________
- Closed: ___________
- Movements (likelihood/impact): ___________
- Risk acceptances proposed: ___________

## 5. Budget & FTE
- Planned spend: $___ · Actual: $___ · Variance: ___%
- Planned FTE-Q: 2.0 · Actual: ___ · Variance: ___
- Contractor spend (capped $1.2M for P0/1): $___

## 6. Phase 1 readiness checks (forward-looking)

| Item | Owner | Ready? |
|---|---|---|
| Hardened Node.js + Python + Go runtime images drafted (Dockerfile templates) | Platform | ☐ |
| Prometheus / Loki / Tempo Helm charts vendored under `platform/observability/` | Platform + SRE | ☐ |
| Single-region prod cluster blueprint reviewed (EKS + Karpenter + Cilium) | Platform + Cloud Infra | ☐ |
| SOC 2 control gap analysis delivered by Compliance PM | Compliance | ☐ |
| Backstage scaffolder for "new microservice" drafted | DevEx | ☐ |

## 7. Decision

- ☐ **GO** — proceed to Phase 1 immediately.
- ☐ **CONDITIONAL GO** — proceed with named conditions (capture below); status check in 2 weeks.
- ☐ **NO-GO** — 2-week corrective window agreed; re-review on ___________.

### Conditions (if conditional)
1. ___________
2. ___________
3. ___________

### Dissents (if any)
- ___________ (named approver) — ___________ reason.

## 8. Action items

| # | Action | Owner | Due |
|---|---|---|---|
| 1 | | | |
| 2 | | | |
| 3 | | | |

## 9. Sign-off

| Approver | Decision | Signature | Date |
|---|---|---|---|
| Chair (VP Platform) | | | |
| CTO | | | |
| CISO | | | |
| CFO delegate | | | |
| Head of SRE | | | |
| Head of Compliance | | | |
| Head of DevEx | | | |

---

*Minutes archived under `docs/leadership-review/gate-reviews/phase-0-<date>.md` on completion.*
