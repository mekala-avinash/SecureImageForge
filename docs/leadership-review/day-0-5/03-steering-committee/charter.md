# Enterprise Platform Transformation — Steering Committee Charter

> Status: DRAFT (effective on program approval) · Version: 1.0 · Owner: Program PM

## 1. Purpose
The Steering Committee (SteerCo) is the governance body for the Enterprise Platform Transformation program. It owns scope, schedule, budget, risk, and phase-gate decisions for the 12-month program.

## 2. Authority
The SteerCo holds delegated authority from the CTO to:
- Approve phase-gate progression (go / no-go / re-baseline).
- Approve in-flight scope changes within ±10% FTE and ±15% budget without escalation.
- Approve risk acceptances at MEDIUM and below.
- Escalate to CTO / CFO / CISO when above thresholds.

## 3. Composition

| Role | Member | Voting | Notes |
|---|---|---|---|
| Chair | VP Platform Engineering | ✓ | Casting vote |
| Sponsor | CTO | ✓ | Attends quarterly + on phase gates |
| Security | CISO | ✓ | |
| Finance | CFO or delegate | ✓ | |
| SRE | Head of SRE | ✓ | |
| Compliance | Head of Compliance | ✓ | |
| DevEx | Head of Developer Experience | ✓ | |
| FinOps | FinOps Lead | observer | |
| Revenue (Enterprise) | VP Revenue | observer | |
| Revenue (Regulated) | VP Regulated Revenue | observer | Joins from Phase 2 |
| Program Manager | Program PM | — | Chair-of-staff, owns minutes |

Quorum: 5 of 7 voting members. Voting decisions captured in minutes.

## 4. Cadence

| Cadence | Format | Duration | Purpose |
|---|---|---|---|
| Biweekly | Standing meeting | 60 min | Status, blockers, decisions |
| Phase gate | Extended meeting | 2 hours | Formal go/no-go review |
| Ad-hoc | Async or sync | as needed | Material risk or scope change |
| Quarterly | Exec readout | 60 min | CTO + board-cycle alignment |

Standing slot: **Fridays 10:00–11:00** (local to chair).

## 5. Agenda Template
See [`agenda-template.md`](./agenda-template.md).

## 6. Decision Process
- Default: consensus.
- Tie-breaker: chair, with formal dissent captured.
- Material decisions (phase progression, budget re-baseline, scope addition >10%) require a recorded vote with quorum.

## 7. Minutes & Transparency
- Minutes published in `#platform-transformation` within 24 hours.
- Decisions tracked in `docs/leadership-review/decisions-log.md` (created by Program PM).
- Risk register reviewed every meeting.
- All material decisions PR-merged to the program docs.

## 8. Phase-Gate Criteria
Binding criteria are recorded in `PHASING_AND_FTE_APPROVAL.md` §3. SteerCo cannot lower a gate criterion without CTO sign-off.

## 9. Escalation Path
- L1: Program PM → Chair.
- L2: Chair → CTO.
- L3: CTO → Board (only for re-baseline > 25% budget or compliance scope change).

## 10. Lifecycle
- SteerCo dissolves at end of Phase 3, with operational handover to the standing Platform Leadership Forum.
- A final retrospective is conducted within 30 days of program close.
