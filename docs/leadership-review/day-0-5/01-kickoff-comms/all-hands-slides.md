# Engineering All-Hands — Phase 0 Kickoff (15 min)

> **Date:** Day 0 (post-approval) · **Audience:** Engineering org + invited Product/Security/Compliance · **Owner:** VP Platform

---

## Slide 1 — Title
**Enterprise Platform Transformation**
*Phase 0 starts today.*

---

## Slide 2 — Why now (60 sec)

- Enterprise deals stuck at security review → SOC 2 / ISO blocking $15–25M ARR.
- Regulated verticals (FSI, Health) need PCI / HIPAA scope → $40M TAM.
- Engineering: 5-day lead time, 4h MTTR, manual deploys. We can do better.
- One bad CVE event would be a brand-level incident — supply-chain controls are overdue.

---

## Slide 3 — What we're building (90 sec)

A **production-grade enterprise platform**:
1. Hardened distroless / Wolfi runtime images — ~95% smaller, signed, attested.
2. GitOps everywhere — Argo CD as the canonical deploy mechanism.
3. Multi-region active-active K8s — Istio mTLS, progressive delivery via Argo Rollouts.
4. Continuous compliance — SOC 2 + ISO (Y1), PCI + HIPAA (Y2), FedRAMP (Y3).
5. Backstage as our Internal Developer Platform — golden paths, scaffolders, TechDocs.
6. 15 autonomous engineering agents — testing, security, deploys, incidents, cost, all sandboxed + HITL.
7. Zero-trust supply chain — Cosign + SLSA L3 + SBOM at admission.

---

## Slide 4 — Three phases / 12 months (60 sec)

| Phase | Months | Outcome |
|---|---|---|
| 0 — Bootstrap | 1 | GitOps + Vault + Harbor + Backstage skeleton live |
| 1 — Foundation | 2–4 | Hardened images, observability, single-region prod, SOC 2 readiness |
| 2 — Industrialization | 5–8 | Multi-region, progressive delivery, first 5 agents, SOC 2 Type II audit |
| 3 — Autonomy & Scale | 9–12 | All 15 agents, active-active, FedRAMP readiness, 25–35% cost ↓ |

Each phase has an **explicit go/no-go gate**. We will not advance unless we earn it.

---

## Slide 5 — What changes for you (90 sec)

- **PR experience:** signed commits + scanners run on every PR. Most findings will be auto-fixable.
- **New services:** scaffolded from Backstage → secure & observable by default.
- **Deploys:** GitOps via Argo CD; progressive delivery means most rollbacks happen automatically.
- **On-call:** SLO-driven, with auto-remediation playbooks. We expect MTTR to drop materially.
- **Production access:** SSO + MFA; break-glass via Vault with full session recording.

> If you're touching infrastructure or services, your day-to-day will improve. If you're shipping product, you should barely notice the change beyond your PRs running more checks.

---

## Slide 6 — Hiring (30 sec)

We're opening 4 requisitions this week:
- 3× Sr. Platform Engineer
- 1× Staff DevSecOps Engineer
- 1× Compliance Program Manager
- 1× Sr. SRE (Q2)

Internal candidates strongly encouraged — see #platform-careers.

---

## Slide 7 — Timeline & next steps (30 sec)

- **Week 1–4:** Phase 0 bootstrap — quick wins (signed commits, scanners, branch protection) + standing up GitOps repos.
- **End of Week 4:** Phase 0 gate review.
- **Steering committee:** biweekly, Fridays 10:00.
- **Channel:** `#platform-transformation` for program; `#platform-announcements` for broadcast-only updates.

---

## Slide 8 — Read more

- Master architecture: `/docs/ENTERPRISE_PLATFORM_ARCHITECTURE.md`
- Roadmap: `/docs/roadmap/IMPLEMENTATION_ROADMAP.md`
- Exec briefing: `/docs/leadership-review/EXECUTIVE_BRIEFING.md`

---

## Slide 9 — Q&A

(10 min reserved. Capture unanswered questions → updated FAQ within 24h.)
