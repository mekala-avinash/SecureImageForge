# Daily Standup Cheat Sheet

Phase 1 standup runs **async in `#platform-transformation`**, not live. Each workstream lead posts by 09:25; chair scans by 09:30.

## Format (paste this template every day)

```
:hardened-runtimes:
Yesterday:
  • <bullet>
Today:
  • <bullet>
Blockers:
  • none
Risk movement: none
```

(Same for `:observability:`, `:prod-cluster:`, `:soc2:`.)

## Rules

- ≤ 3 bullets per Yesterday/Today section.
- "Blockers: none" is acceptable; missing the post is not.
- If a blocker is open ≥ 24h, it auto-escalates to Wednesday working session.
- If a risk movement is HIGH, ping the chair directly in addition to posting.

## What standup is NOT

- Status reporting for SteerCo (that's Friday).
- Architecture discussion (that's the working session or an ADR).
- A place to debug (DM/call the relevant owner).

## Standup health metric

Program PM tracks standup posting compliance weekly. Target: ≥ 95%. < 80% triggers a chair conversation.
