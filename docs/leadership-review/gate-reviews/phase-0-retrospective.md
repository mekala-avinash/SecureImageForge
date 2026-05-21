# Phase 0 Retrospective (Blameless)

> **Date:** 2026-02-13 · **Facilitator:** Program PM · **Format:** 60 min, async pre-write + 30 min sync
> **Status:** WORKED EXAMPLE — populate with real input.

## Scope
Phase 0 of the Enterprise Platform Transformation — 4 weeks, ~$0.61M spend, 1.9 FTE-Q.

## What went well

- **Comms cadence.** Day-0 all-hands + Slack post + email + FAQ landed with high engagement. ~92% of eng joined `#platform-transformation` within 48h.
- **GitOps repo bootstrap.** Single-command `push-all.sh` automated Day-5 push + branch protection in ~12 minutes total.
- **Quick wins were genuinely quick.** Org-wide secret scan + hadolint + semgrep + signed commits enforced by end of Week 2 with only 2 historical-secret rotations needed.
- **Hiring funnel exceeded plan.** First platform offer out by Week 4.
- **Vendor RFPs returned with no proprietary lock-in once Procurement redlined MSAs upfront.**

## What didn't go well

- **Backstage scaffolder lagged.** Node.js variant slipped — root cause: under-scoped initial estimate by DevEx; partial workaround used in Week 4. Material to Phase-1 readiness.
- **KMS auto-unseal IAM gap (R-02).** Caught during code review, not by automated test. Action: extend Cloud Infra Terratest to assert KMS principal allow-list.
- **gitsign install friction on macOS arm64.** Two office-hour sessions needed; ~30 engineers needed individual help. Action: prebuilt Homebrew tap.
- **No standing "demo Friday".** Engineers wanted more visibility into platform progress. Action: 15-min demo at end of every other Friday.

## Surprises (good and bad)

- 🟢 Drata's K8s + GitHub connectors covered ~70% of in-scope SOC 2 controls out of the box — better than we modeled.
- 🟡 Tetrate POC scoping conversation surfaced an Argo Rollouts + east-west gateway interaction we hadn't accounted for; ticket opened.
- 🔴 One platform engineer left during week 3 (unrelated). Re-allocating their Phase-1 scope; backfill in flight.

## Actions

| # | Action | Owner | Due |
|---|---|---|---|
| 1 | Add Terratest assertion for Vault auto-unseal IAM | Cloud Infra | 2026-02-20 |
| 2 | Publish prebuilt `gitsign` Homebrew tap + Windows MSI | DevEx | 2026-02-27 |
| 3 | Re-estimate Backstage scaffolder backlog; commit to dates | DevEx Lead | 2026-02-20 |
| 4 | Stand up biweekly Demo Friday (15 min, recorded) | DevEx + Program PM | 2026-02-13 |
| 5 | Replan attrition: re-distribute Phase-1 scope | VP Platform | 2026-02-20 |

## Metrics snapshot (baseline for the rest of the program)

| Metric | Baseline | P0 close | Target end-of-P1 |
|---|---|---|---|
| Avg PR lead time | 4.5 days | 3.1 days | < 1 day |
| % repos with signed commits | 0% | 100% | 100% |
| % repos with secret scan | 28% | 100% | 100% |
| Services in Backstage | 0 | 7 | 30 |
| Compliance controls auto-evidenced | 0 | 41 | 80 |
| MTTR (incident-weighted avg) | 4h 12m | n/a (no incidents) | 90 min |

## Lessons captured for Phase 1
- Schedule scaffolder work earlier in the phase, not as the closing item.
- Estimate vendor commercial cycles at 2× engineering estimate (procurement is the bottleneck).
- Demo Friday from day one (we should have started this in Phase 0).
