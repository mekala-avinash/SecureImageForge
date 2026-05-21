# Phasing & FTE Allocation — Formal Approval Record

> **Status:** AWAITING APPROVAL
> **Linked artifacts:**
> – Executive briefing: [`EXECUTIVE_BRIEFING.md`](./EXECUTIVE_BRIEFING.md)
> – Detailed roadmap: [`../roadmap/IMPLEMENTATION_ROADMAP.md`](../roadmap/IMPLEMENTATION_ROADMAP.md)
> – Master architecture: [`../ENTERPRISE_PLATFORM_ARCHITECTURE.md`](../ENTERPRISE_PLATFORM_ARCHITECTURE.md)

---

## 1. Program Summary

| Item | Value |
|---|---|
| Program name | Enterprise Platform Transformation (EPT) |
| Sponsor | CTO |
| Executive owner | VP Platform Engineering |
| Duration | 12 months, 3 phases (+ Phase 0 bootstrap) |
| Total effort | ~23 FTE-quarters |
| Indicative all-in cost | ~$14.8M (people + cloud + tooling); Phase 0 ~$0.6M |
| Compliance scope | Yr1 SOC2 Type II + ISO 27001 · Yr2 PCI v4.0 + HIPAA · Yr3 FedRAMP Moderate |
| Cloud strategy | AWS primary · Azure secondary · GCP tertiary · K8s-portable (Crossplane) |
| Go-live (Phase 0 start) | Pending approval |

---

## 2. Decision Register

| # | Decision | Recommendation | Approver Role | Status |
|---|---|---|---|---|
| D-1 | Approve overall 12-month program & charter | Approve | CTO | ☐ |
| D-2 | Approve Phase 0 immediate kickoff (4 wks, ~$0.6M, 2 FTE-Q) | Approve | CTO + CFO | ☐ |
| D-3 | Approve Yr1 compliance scope: SOC 2 Type II + ISO 27001 | Approve | CTO + CISO + VP Revenue | ☐ |
| D-4 | Approve Yr2 scope: PCI-DSS v4.0 + HIPAA | Approve | CTO + CISO + VP Regulated Revenue | ☐ |
| D-5 | Approve Yr3 scope: FedRAMP Moderate readiness | Approve | CTO + CISO + Public-Sector Lead | ☐ |
| D-6 | Approve cloud strategy (AWS primary, multi-cloud portable) | Approve | CTO + Head of Infra | ☐ |
| D-7 | Approve +4 hires (Sr Platform ×3, Staff DevSecOps ×1) by end Q1 | Approve | CTO + CFO + VP HR | ☐ |
| D-8 | Approve compliance PM hire (1.0 FTE, by Q1) | Approve | CTO + Head of Compliance | ☐ |
| D-9 | Approve phase-gate model with explicit go/no-go reviews | Approve | CTO + VP Platform | ☐ |
| D-10 | Approve agent program with blast-radius governance | Approve | CTO + CISO | ☐ |
| D-11 | Approve indicative ~$14.8M program budget envelope (subject to phase-gate re-baseline) | Approve in principle | CFO | ☐ |
| D-12 | Approve adoption of Backstage as canonical IDP | Approve | VP Platform + DevEx | ☐ |

---

## 3. Phase-Gate Criteria (binding)

Each phase advances **only** when its exit gate has been independently verified by the named gate-keeper. Failure to meet a gate triggers a 2-week corrective window before re-baselining.

| Gate | Exit Criteria | Gate-keeper |
|---|---|---|
| 0 → 1 | Pilot service builds, scans, signs, deploys via GitOps; Vault/Harbor/Backstage live | VP Platform |
| 1 → 2 | New service from Backstage scaffold ships to staging in <1h; OTel telemetry green across 10 services; Kyverno verifyImages enforced in nonprod | VP Platform + CISO |
| 2 → 3 | Multi-region active-passive failover drill passes (RTO < 5 min); Argo Rollouts canary in 100% of prod deploys; 5 wave-1 agents live in HITL | VP Platform + Head of SRE |
| End of 3 | SOC2 Type II report issued; FedRAMP body-of-evidence assembled; 25–35% YoY cost reduction realized | CTO + CISO + CFO |

---

## 4. FTE Allocation Detail (binding once signed)

### 4.1 Quarterly view (FTE-quarters)

| Function | Q1 (P0/P1) | Q2 (P1) | Q3 (P2) | Q4 (P2/P3) | **Total** |
|---|---|---|---|---|---|
| Platform Engineering | 2.0 | 2.5 | 3.0 | 1.5 | **9.0** |
| DevSecOps / AppSec | 1.0 | 1.0 | 1.0 | 1.0 | **4.0** |
| SRE | 0.5 | 1.0 | 1.5 | 1.0 | **4.0** |
| Compliance / GRC | 0.5 | 0.75 | 0.75 | 0.75 | **2.75** |
| Autonomy / Agents | 0.25 | 0.25 | 0.5 | 0.5 | **1.5** |
| FinOps | 0.0 | 0.25 | 0.25 | 0.25 | **0.75** |
| DevEx (Backstage) | 0.25 | 0.25 | 0.25 | 0.25 | **1.0** |
| **Quarterly total** | **4.5** | **6.0** | **7.25** | **5.25** | **23.0** |

### 4.2 Hiring requirements (binding requisitions)

| Role | Count | Target close | Owner | Status |
|---|---|---|---|---|
| Sr. Platform Engineer | 3 | End Q1 | VP HR + VP Platform | ☐ Open req |
| Staff DevSecOps Engineer | 1 | End Q1 | VP HR + CISO | ☐ Open req |
| Compliance Program Manager | 1 | End Q1 | VP HR + Head of Compliance | ☐ Open req |
| Sr. SRE | 1 | End Q2 | VP HR + Head of SRE | ☐ Open req |

Without these hires landing on time, **Phase 2 slips by 6–10 weeks**. This is the single largest schedule risk.

### 4.3 Contractor backfill (contingency)

If hiring lags >4 weeks vs target, pre-approved contractor backfill (≤ 6-month engagements) is authorized for:
- Backstage IDP setup
- Vault HA bootstrap
- Initial Kyverno/OPA policy library

Contractor cap: **$1.2M within Phase 0/1 envelope**, no extension without re-approval.

---

## 5. Budget Envelope (indicative)

| Category | Phase 0 | Phase 1 | Phase 2 | Phase 3 | **Total** |
|---|---|---|---|---|---|
| Personnel (loaded) | $0.40M | $1.60M | $2.10M | $1.85M | $5.95M |
| Cloud (transition double-spend) | $0.05M | $0.40M | $1.60M | $2.80M | $4.85M |
| Tooling & licenses | $0.10M | $0.30M | $0.80M | $1.00M | $2.20M |
| Audit & compliance | $0.05M | $0.10M | $0.30M | $0.45M | $0.90M |
| Contingency (10%) | $0.06M | $0.24M | $0.48M | $0.61M | $1.39M |
| **Phase total** | **$0.66M** | **$2.64M** | **$5.28M** | **$6.71M** | **$15.29M** |

Notes:
- Cloud cost rises during transition (double-spend) and falls steeply once Phase 3 FinOps measures land — **net Y2 run-rate is projected 25–35% below baseline**.
- All figures order-of-magnitude; **rebaseline at each phase gate**.

---

## 6. Out-of-Scope (this approval)

- Application-level rewrites beyond strangler-pattern routing.
- Net-new product features.
- Data-warehouse / lakehouse modernization (separate program).
- Vendor consolidation outside platform tooling.

---

## 7. Assumptions

1. Existing monolith remains operable throughout the program; no parallel re-platform.
2. Cloud account structures (org/multi-account landing zone) already in place or land in Phase 0.
3. HRIS (Workday/Okta) supports SCIM provisioning to Keycloak.
4. Existing CI/CD vendor (GitHub or GitLab) remains primary.
5. Network connectivity between regions is provisioned (Transit Gateway/Hub-Spoke).

If any assumption breaks, re-baseline the corresponding phase.

---

## 8. Approvals

By signing below, approvers commit to the program, phase gates, FTE allocation, and budget envelope captured above. Each phase gate retains an explicit go/no-go review with re-baselining authority.

| Approver | Role | Decision | Date | Signature |
|---|---|---|---|---|
| _________________________ | CTO | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | CISO | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | CFO | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | VP Platform Engineering | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | Head of SRE | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | Head of Compliance | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | VP HR / TA | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | VP Revenue (Enterprise) | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |
| _________________________ | VP Revenue (Regulated) | ☐ Approved ☐ Approved with conditions ☐ Rejected | __________ | __________________________ |

### Conditions / Dissents (record here)

```
[Approver name] — [date] — [condition or dissent]
________________________________________________________________
________________________________________________________________
________________________________________________________________
________________________________________________________________
```

---

## 9. Post-Approval Actions (Day 0–5)

| # | Action | Owner | Due |
|---|---|---|---|
| 1 | Issue Phase 0 kickoff comms (eng all-hands + #platform-announcements) | VP Platform | Day 0 |
| 2 | Open 4 requisitions (see § 4.2) | VP HR | Day 1 |
| 3 | Stand up program steering committee (biweekly cadence) | Program PM | Day 3 |
| 4 | Bootstrap `gitops/`, `platform/`, `tenants/` repos with CODEOWNERS | Platform Lead | Day 5 |
| 5 | Engage Drata/Vanta/Secureframe for compliance automation eval | Compliance PM | Day 5 |
| 6 | Begin RFP/eval of managed Istio offering (Tetrate / Anthos / Solo) | Platform Lead | Day 5 |
| 7 | Schedule first phase-gate review (end of Phase 0, ~Week 4) | Program PM | Day 1 |

---

## 10. Change Control

This document is the **canonical record** of phasing & FTE allocation. Material changes (phase scope, ±10% FTE, ±15% budget, compliance scope) require:

1. Written change request in `/app/docs/leadership-review/changes/CR-NNN.md`.
2. Re-approval by **CTO + CFO + CISO** at minimum.
3. New revision of this document, with prior version archived.

---

*Once all required approvers have signed, this document moves to status* **APPROVED — IN EXECUTION** *and Phase 0 kickoff is authorized immediately.*
