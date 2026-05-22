# Phase 1 — Mid-Point Review Template

> **Scheduled:** Week 6 (2026-03-20) Friday 10:00–12:00 · **Type:** 2-hour SteerCo deep dive
> **Purpose:** Mid-program health check + course correction. Not a gate decision (gate is Week 12).

## Why a mid-point exists

Phase 1 is 12 weeks. Without an explicit mid-point we discover problems at Week 11, too late to correct. The mid-point is a **course-correction checkpoint**, not a vote.

## Pre-read (sent Wednesday 2026-03-18)

| Document | Owner |
|---|---|
| Phase-1 KPI dashboard snapshot (Grafana) | SRE |
| Hardened runtime adoption tracker | Platform Lead |
| Observability instrumentation tracker | SRE |
| Prod cluster readiness (terraform output + kube-bench + Polaris) | Cloud Infra Lead |
| SOC 2 evidence completeness (Drata report) | Compliance PM |
| Risk register diff since Phase-1 kickoff | Program PM |
| Budget burn vs plan (Phase 1) | CFO delegate |

## Agenda (120 min)

### 0:00–0:10 — Roll call & framing (Chair)
- Quorum.
- Framing: this is course correction, not a vote.

### 0:10–0:40 — KPI snapshot vs Phase-1 targets (Program PM)
Walk through:

| KPI | Target (P1) | Mid-point baseline | On track? |
|---|---|---|---|
| % services on hardened runtime | ≥ 60% | __% | 🟢/🟡/🔴 |
| % services with OTel | ≥ 80% | __% | |
| New-service scaffold → staging | < 1h | __ min | |
| Kyverno admission blocks (nonprod) | enforced | __ | |
| SOC 2 controls auto-evidenced | ≥ 80% | __% | |
| MTTR (incident-weighted) | < 90 min | __ min | |
| Image avg size (hardened) | ≤ 100 MB | __ MB | |
| Backstage active users | ≥ 200 | __ | |

For each amber/red KPI: 2 min on cause + 1 min on remediation.

### 0:40–1:10 — Workstream deep-dive (5 × 6 min)
Each workstream lead presents:
- What we said we'd do by mid-point.
- What we actually shipped.
- Gap analysis + recovery plan if behind.
- Updated forecast for Phase-1 gate.

### 1:10–1:30 — Risk & budget delta
- New risks (≥ MEDIUM) since kickoff.
- Movements.
- Budget actual vs plan (Phase 1 only).
- Hiring funnel update.

### 1:30–1:50 — Course corrections (decisions)
Vote on any of:
- Scope reshape within Phase 1 (drop / defer / add).
- Resource reallocation between workstreams.
- Phase-2 dependency adjustments (e.g., Tetrate POC outcome).
- Contractor spend approval (if any).

### 1:50–2:00 — Wrap
- Confirm phase-gate is still scheduled (Week 12, 2026-05-08).
- Action items + owners + due dates.
- Communications plan to engineering.

## Course-correction options (pre-thought)

| If we're behind on… | Option A | Option B | Option C |
|---|---|---|---|
| Hardened runtimes | Pair migration sessions ×2/week | Trim Tier-B scope to 5 services | Defer Java/Rust to Phase 2 |
| Observability | Mandate auto-instrumentation for laggards | Defer Mimir long-term to Phase 2 | Reduce dashboard standardization scope |
| Prod cluster | Reduce hardening checklist to CIS 90% | Accept higher cost from on-demand | Delay first production traffic to Week 10 |
| SOC 2 | Reduce Year-1 in-scope controls to CC only | Engage Drata pro-services for acceleration | Push audit window open to Week 9 |

## Outputs

- Mid-point review minutes (archived `gate-reviews/phase-1-midpoint-YYYY-MM-DD.md`).
- Updated KPI dashboard with new forecast lines.
- Communications post to `#platform-announcements` summarizing status.

## After the meeting

- 30-min retrospective with the 4 workstream leads (just leads).
- Update `/app/docs/phase-1/README.md` schedule if course-corrections affect the timeline.
- Update PRD with any scope or schedule changes.
