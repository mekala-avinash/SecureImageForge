# Slack #platform-announcements — Phase 0 Kickoff

> Channel: `#platform-announcements` · Audience: full eng org · Post: Day 0, 09:00 local · Author: VP Platform

---

:rocket: **Enterprise Platform Transformation — Phase 0 starts today**

TL;DR: We're kicking off a 12-month program to harden, scale, and modernize our platform — and unlock the SOC 2 / ISO / PCI / HIPAA controls our enterprise and regulated customers have been asking for. Phase 0 is 4 weeks of foundation work (no scary changes to your daily workflow yet).

**Why**
- Unblock enterprise + regulated revenue (SOC 2 → ISO → PCI → HIPAA → FedRAMP).
- Cut deploy lead time from days to under an hour.
- Reduce MTTR materially via SLO-driven progressive delivery.
- Modernize our supply chain (signed artifacts, SBOMs, SLSA L3).

**What lands in Phase 0 (Weeks 1–4)**
- GitOps repos (`gitops/`, `platform/`, `tenants/`) bootstrapped with CODEOWNERS.
- Vault HA + KMS auto-unseal.
- Harbor registry + Cosign signing.
- Backstage skeleton (service catalog).
- Quick wins everywhere: signed commits, secret scanning, hadolint, semgrep, branch protection.

**What changes for you in Phase 0**
- Your PRs will run more checks. Most findings auto-fix via suggestions.
- Signed commits become required (we'll provide gitsign setup in a doc + office hours).
- Nothing else changes — your service deploys, your dashboards, your alerts all stay where they are.

**What's coming after**
- Phase 1 (Mo 2–4): hardened distroless images, OTel everywhere, Kyverno verify in nonprod.
- Phase 2 (Mo 5–8): multi-region, Argo Rollouts canary, first 5 autonomous agents (HITL).
- Phase 3 (Mo 9–12): all 15 agents, active-active, FedRAMP body of evidence, PCI/HIPAA tenants.

**Hiring**
- 3× Sr. Platform · 1× Staff DevSecOps · 1× Compliance PM · 1× Sr. SRE (Q2)
- Internal candidates encouraged: :point_right: `#platform-careers`

**Where to find things**
- :compass: Program channel: `#platform-transformation` (join us!)
- :books: Docs root: `docs/README.md`
- :clipboard: Roadmap: `docs/roadmap/IMPLEMENTATION_ROADMAP.md`
- :question: FAQ: `docs/leadership-review/day-0-5/01-kickoff-comms/faq.md`
- :calendar: All-hands recording: link <after the meeting>

**Steering committee**
Biweekly Fridays 10:00. Notes published to `#platform-transformation`.

**Got questions?**
Drop them in `#platform-transformation` or DM me. We'll add anything common to the FAQ.

— <VP Platform>
:thread: *Use this thread for questions; we'll keep the channel clean.*
