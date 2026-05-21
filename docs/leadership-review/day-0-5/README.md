# Day 0–5 Execution Pack

Once approval is signed in [`../PHASING_AND_FTE_APPROVAL.md`](../PHASING_AND_FTE_APPROVAL.md), execute these 7 actions in the order shown. Every action is fully drafted — owners only need to **send / open / file / schedule**.

| # | Action | Artifact(s) | Owner | Target | Status |
|---|---|---|---|---|---|
| 1 | Issue kickoff comms | [`01-kickoff-comms/`](./01-kickoff-comms/) | VP Platform | Day 0 | ☐ |
| 2 | Open 4 requisitions | [`02-requisitions/`](./02-requisitions/) (4 JDs) | VP HR + hiring managers | Day 1 | ☐ |
| 3 | Stand up steering committee | [`03-steering-committee/`](./03-steering-committee/) | Program PM | Day 3 | ☐ |
| 4 | Bootstrap GitOps + Platform + Tenants repos | [`04-gitops-bootstrap/`](./04-gitops-bootstrap/) + `/app/gitops-bootstrap/` | Platform Lead | Day 5 | ☐ |
| 5 | Engage compliance automation vendor | [`05-compliance-platform-rfp/`](./05-compliance-platform-rfp/) | Compliance PM | Day 5 | ☐ |
| 6 | Issue managed-Istio RFP | [`06-managed-istio-rfp/`](./06-managed-istio-rfp/) | Platform Lead | Day 5 | ☐ |
| 7 | Schedule Phase-0 gate review | [`07-phase-0-gate-review/`](./07-phase-0-gate-review/) | Program PM | Day 1 (schedule) → Week 4 (run) | ☐ |

### Sequencing notes
- Items 1 → 2 → 3 must be in that order on Day 0–3 so people know what's coming **before** they see job reqs or meeting invites.
- Items 4–6 run in parallel from Day 3 onward.
- Item 7 (schedule) goes on calendars on Day 1 even though the meeting happens at end of Phase 0.

### Communication discipline
- All program comms originate from `#platform-announcements` (Slack) and `platform-eng@acme.io` (email).
- Every external (vendor) interaction is logged in the steering committee minutes.
- No reqs posted publicly until Day 1 morning to align with all-hands comms.
