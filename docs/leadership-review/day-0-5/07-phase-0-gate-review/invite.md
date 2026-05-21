Subject: Calendar invite — Phase 0 Gate Review (End of Week 4)

To: SteerCo voting members + Program PM
Cc: CTO
When: <Week 4, Friday, 10:00–12:00 local>
Where: <Zoom + Slack #platform-transformation>
Duration: 2 hours

---

This is the **Phase 0 gate review** for the Enterprise Platform Transformation program.
We will decide GO / CONDITIONAL GO / NO-GO into Phase 1 based on the binding exit criteria in `PHASING_AND_FTE_APPROVAL.md` §3.

Agenda
- 0:00–0:10  Roll call, quorum, prior-minutes approval.
- 0:10–0:30  Phase 0 status walkthrough (Program PM): scope completed, in-flight, deferred.
- 0:30–1:00  Exit-criteria verification (gate-keeper: VP Platform):
              • Pilot service builds → scans → signs → deploys via GitOps.
              • GitOps + Vault + Harbor + Backstage skeleton live.
              • Signed commits + secret scan + lint on 100% of repos.
              • Baseline reusable CI workflow in use by ≥ 3 pilot services.
- 1:00–1:20  Risk + budget delta (Chair + CFO delegate).
- 1:20–1:40  Phase 1 readiness (workstream owners present readiness):
              • Hardened runtimes
              • Observability stack
              • Single-region prod cluster
              • SOC 2 readiness
              • Backstage golden-path scaffolders
- 1:40–1:55  Decision: GO / CONDITIONAL GO / NO-GO. Capture conditions or corrective window.
- 1:55–2:00  Action items + owners + due dates.

Pre-read (sent 48h prior)
- This week's status report.
- Evidence pack: artifact links proving each exit criterion (CI run links, screenshots, Argo CD UI dumps).
- Risk register diff.
- Updated FTE + hiring funnel.

— Program PM
