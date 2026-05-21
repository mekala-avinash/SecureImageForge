# Phase 1 — Foundation (Months 2–4)

> **Status:** IN EXECUTION (Phase-0 gate review GO with 2 conditions)
> **Window:** 2026-02-09 → 2026-05-08 (12 weeks)
> **Owner:** VP Platform Engineering
> **Companion docs:**
> – Master architecture: [`../ENTERPRISE_PLATFORM_ARCHITECTURE.md`](../ENTERPRISE_PLATFORM_ARCHITECTURE.md)
> – Roadmap: [`../roadmap/IMPLEMENTATION_ROADMAP.md`](../roadmap/IMPLEMENTATION_ROADMAP.md)

---

## 1. Phase-1 Exit Criteria (binding)

(Mirrored from `PHASING_AND_FTE_APPROVAL.md` §3 — "1 → 2 gate")

1. New service from Backstage scaffold ships to staging in **< 1 hour**, fully observable.
2. OpenTelemetry instrumented across **≥ 10 production services** with traces + metrics + logs all flowing.
3. Kyverno `verifyImages` (signature + SBOM + SLSA provenance) **enforced in nonprod**.
4. Single-region multi-AZ EKS prod cluster operational; pilot service migrated.
5. **SOC 2 Type II audit window opened**; ≥ 80% of in-scope controls automated-evidenced via Drata.
6. ISO 27001 Stage 1 readiness review passed.

Gate-keeper: VP Platform + CISO. Vote on 2026-05-08.

## 2. Four Workstreams

| # | Workstream | Lead | Sub-folder |
|---|---|---|---|
| 1 | Hardened runtimes adoption | Platform Lead | [`01-hardened-runtimes/`](./01-hardened-runtimes/) |
| 2 | Observability rollout | Head of SRE | [`02-observability/`](./02-observability/) |
| 3 | Single-region prod cluster | Cloud Infra Lead + Platform Lead | [`03-single-region-prod-cluster/`](./03-single-region-prod-cluster/) |
| 4 | SOC 2 Type II audit-window prep | Compliance PM | [`04-soc2-audit-prep/`](./04-soc2-audit-prep/) |

## 3. 12-Week Schedule (high-level)

| Week | Hardened runtimes | Observability | Prod cluster | SOC 2 |
|---|---|---|---|---|
| 1 | Backstage scaffolder GA for Node/Python/Go; runtime images published v1.0.0 | Prom + Loki + Tempo Helm vendored; OTel collector v1 | Terraform module skeleton; account vending | Drata pilot → production tenancy; control owners assigned |
| 2 | Onboard pilot service #1 (orders-api-poc) end-to-end | OTel SDK examples in 3 reference repos | EKS bootstrap on `prod-use1`; private subnets | Control test plan v1 |
| 3 | Onboard services #2–3 | First service emitting traces; Tempo dashboards | Karpenter + Cilium + Istio install | Evidence collectors live for K8s + GitHub + AWS |
| 4 | Services #4–6; waiver process live | 5 services instrumented; SLOs as code | Argo CD bootstrap on prod cluster; CSI Secrets Store | First mock auditor walkthrough |
| 5 | Java + Rust runtimes v1.0.0 | Loki log pipelines per service; PII redaction | First workload (non-revenue) deployed to prod cluster | Auditor selected; engagement letter signed |
| 6 | Services #7–10 | 10 services instrumented end-to-end | Backup + DR runbook tested | SOC 2 readiness gap list closed to ≤ 5 items |
| 7 | Ruby + PHP + .NET v1.0.0 | Auto-remediation playbooks for top 3 alert classes | Pilot tenant migration (low-risk service) | Auditor on-site planning week |
| 8 | Tooling image fleet GA | Grafana dashboards-as-code for all services | Cluster hardening checklist passed | **SOC 2 audit window opens** |
| 9 | All new services scaffold-only; legacy migration plan published | OTel collector tail-sampling + PII redaction prod-ready | Production traffic % ramp to 25% | Audit walkthroughs Week 1 |
| 10 | 60% of repos on hardened runtimes | Loki/Tempo retention policies enforced | Production traffic ramp to 60% | Audit walkthroughs Week 2 |
| 11 | Migration completion plan signed off | Burn-rate alerts validated by chaos drill | Production traffic ramp to 100% | Evidence remediation |
| 12 | Phase-1 gate prep | Phase-1 gate prep | Phase-1 gate prep | Audit fieldwork concludes |

## 4. Phase-1 KPIs (tracked weekly in SteerCo)

| KPI | Baseline (P0 close) | Phase-1 target |
|---|---|---|
| % services on hardened runtime image | 0% | ≥ 60% (of new + top 10 critical) |
| % services with OTel traces flowing | 0% | ≥ 80% of in-scope |
| Mean PR lead time (top 10 services) | 3.1 days | < 1 day |
| New-service scaffold → staging | ~6h | < 1h |
| Kyverno admission blocks (nonprod) | n/a | enforced, 0 bypass |
| SOC 2 controls auto-evidenced | 41 | ≥ 80% of in-scope |
| Active Backstage users | 25 | ≥ 200 |
| Image avg size (hardened) | n/a | ≤ 100MB runtime images |
| Mean MTTR (incident-weighted) | n/a | < 90 min |

## 5. Risks & Mitigations (Phase-1 view)

| ID | Risk | L | I | Mitigation |
|---|---|---|---|---|
| P1-R1 | Per-team migration to hardened runtimes drags | M | M | "Pave the Path" bootcamp Week 2; per-team adoption SLA |
| P1-R2 | OTel SDK adoption gaps in legacy code | M | M | Auto-instrumentation agents (OTel zero-code) for languages that support it |
| P1-R3 | EKS prod cluster timing slip | M | H | Account vending pre-staged in Phase 0; Crossplane fallback |
| P1-R4 | SOC 2 evidence gaps surface late | M | H | Week-4 mock auditor walkthrough; weekly Drata gap review |
| P1-R5 | Tetrate POC discovery delays Istio install | M | M | Self-managed Istio fallback path pre-scripted |
| P1-R6 | Backstage Node.js scaffolder slips (carryover from Phase-0 condition C-01) | L | M | Parity gate enforced 2026-02-20 (status check at SteerCo) |

## 6. Cadence

- **Daily standup:** 15 min, workstream leads only.
- **Weekly working session:** Wednesdays 90 min, all squads.
- **SteerCo:** biweekly Fridays 10:00 (unchanged).
- **Demo Friday:** biweekly 15-min recorded.
- **Phase-1 mid-point review:** Week 6 (2026-03-20).
- **Phase-1 gate review:** Week 12 (2026-05-08).

## 7. Reference Artifacts

- Hardened runtime templates → `/app/docs/runtime-images/runtimes/`
- BuildKit + Cosign + SBOM + SLSA build script → `/app/docs/runtime-images/templates/buildkit-build.sh`
- Kyverno verifyImages policy → `/app/docs/security/admission/kyverno-verify-images.yaml`
- Argo CD ApplicationSet → `/app/docs/cicd/argo/applicationset-orders.yaml`
- OTel Collector config → `/app/docs/observability/otel/collector-gateway.yaml`
- SLO + burn-rate alerts → `/app/docs/observability/prometheus/slo-orders-api.yaml`
- Control mapping → `/app/docs/compliance/control-mapping.md`

## 8. Read Next

- [`01-hardened-runtimes/adoption-playbook.md`](./01-hardened-runtimes/adoption-playbook.md)
- [`02-observability/rollout-plan.md`](./02-observability/rollout-plan.md)
- [`03-single-region-prod-cluster/blueprint.md`](./03-single-region-prod-cluster/blueprint.md)
- [`04-soc2-audit-prep/audit-window-plan.md`](./04-soc2-audit-prep/audit-window-plan.md)
