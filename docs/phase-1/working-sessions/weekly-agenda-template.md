# Phase 1 — Weekly Working Session Agenda Template

> **Cadence:** Wednesdays 14:00–15:30 (90 min) · **Format:** 4 workstream leads + Program PM + invited squad leads · **Owner:** VP Platform Engineering (chair)
> **Difference from SteerCo:** SteerCo = decisions, risks, budget. Working session = unblock, demo, technical alignment.

## Pre-read (sent Tuesday 17:00)

- Burndown screenshot from Jira/Linear.
- Updated `week-N-milestones/README.md`.
- Demo URLs (Grafana, Backstage, etc.) — must be reachable before the meeting.
- Open ADRs needing discussion.

## Agenda (90 min)

### 0:00–0:10 — Roll & async pre-read confirmation
- Confirm everyone read the milestone tracker (no in-meeting status reading).
- Capture any topics that bubbled up since the pre-read.

### 0:10–0:30 — Workstream demos (5 min × 4)
Each workstream lead shows **one concrete artifact** that landed since last session. Not a status update — an actual demo.

- **Hardened runtimes:** image push, scaffold output, migration PR diff, …
- **Observability:** trace in Tempo, alert firing, dashboard rendering, …
- **Prod cluster:** Terraform plan output, cluster bootstrap step, kube-bench score, …
- **SOC 2:** Drata connector showing live evidence pull, control evidence sample, …

### 0:30–0:55 — Blockers & coordination (25 min)
For each blocker:
- 90 seconds to describe.
- 30 seconds to identify the owner.
- Decision: solve in-room, async, or escalate to SteerCo.

If a topic needs > 5 min, owner books a follow-up; it does not eat the working session.

### 0:55–1:15 — ADRs & technical decisions (20 min)
- Walk through any open ADR PRs (target: 1–2 per session, not more).
- Decisions captured in the ADR text + Program PM logs to decisions ledger.

### 1:15–1:25 — Forward-look (10 min)
- Next-week milestone preview.
- Cross-workstream dependencies confirmed for next week.
- Demo lineup for next session.

### 1:25–1:30 — Wrap
- Recap action items + owners + due dates.
- Confirm next session attendees.

## Action items table

| # | Action | Owner | Due | Status |
|---|---|---|---|---|
| 1 | | | | ☐ |

## Standing topics (revisit monthly)

- DORA metrics trend (Program PM)
- KPI dashboard against Phase-1 targets (Chair)
- Risk register movements (Chair)
- Headcount & contractor utilization (Program PM)
- Vendor RFP status (Procurement)

## Discipline rules

- No laptops open unless presenting. (Camera optional, microphone-on default.)
- Status updates happen async in standup, not here.
- Demos must be live, not slideware.
- Decisions captured in writing within 24h.
- 25-minute hard cap on blocker section; overflow → side meeting.
