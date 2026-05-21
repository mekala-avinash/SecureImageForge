# SOC 2 Type II Audit Window — Phase 1 Prep Plan

> Phase-1 Workstream 4 · Lead: Compliance PM · Co-lead: CISO · Window: Weeks 1–12

## Strategic Posture

We open the **SOC 2 Type II audit window in Week 8**. The audit observes our controls operating effectively over a ≥ 6-month observation period — meaning the window we open in Week 8 of Phase 1 closes during Phase 2/3. Our Phase-1 goal is to ensure controls are designed, deployed, and producing evidence well before the window opens.

## Reference framework

We use Trust Services Criteria (TSC 2017, as revised) — **Common Criteria + Availability + Confidentiality** (Security + Availability + Confidentiality). Privacy + Processing Integrity scoped out of Year 1.

| TSC Category | In scope Year 1? |
|---|---|
| CC (Common Criteria) — Security | ✅ |
| A — Availability | ✅ |
| C — Confidentiality | ✅ |
| PI — Processing Integrity | ❌ (Year 2) |
| P — Privacy | ❌ (Year 2) |

## Auditor Selection

Engage one of: Coalfire, Schellman, A-LIGN, Prescient Assurance. Engagement letter by end of Week 5. Selection criteria:
- Has performed ≥ 50 SOC 2 Type II for SaaS companies.
- Has examined organizations using Kubernetes + GitOps + multi-cloud.
- Has experience with FedRAMP path (Year 3 dependency).
- Pricing transparent for Type II Year 1 + Year 2 + Year 3.

## In-Scope Systems (Year 1)

| Category | Systems |
|---|---|
| Production K8s | `prod-use1` EKS cluster |
| Identity | Keycloak (OIDC/SAML/SCIM) |
| Secrets | Vault HA + AWS KMS |
| Source control | GitHub Enterprise (`acme/*` orgs) |
| CI/CD | GitHub Actions reusable workflow |
| GitOps | ArgoCD on `mgmt-use1` + `prod-use1` |
| Registry | Harbor + ECR replica |
| Observability | Prometheus + Loki + Tempo + Grafana |
| Endpoint | JAMF (macOS) + Intune (Windows) |
| SIEM | Splunk |
| Compliance platform | Drata |
| Cloud accounts | AWS Org (root + 6 sub-accounts) |

Explicitly **out of scope** Year 1: nonprod clusters, marketing site, sandbox accounts.

## Control Test Plan

The complete cross-framework control mapping is at [`/app/docs/compliance/control-mapping.md`](../../compliance/control-mapping.md). For SOC 2 Type II Year 1 the auditor will test **operating effectiveness over the observation window** for the controls below. Each must have evidence flowing into Drata by Week 8.

### Common Criteria — Logical Access (CC6)

| Control | Test procedure | Evidence source | Owner |
|---|---|---|---|
| CC6.1 Logical access provisioning via SSO + MFA | Auditor samples 25 user provisioning events | Keycloak audit log → Splunk; SCIM events | Compliance PM |
| CC6.2 Access reviews quarterly | Auditor reviews 4 quarters of reviews | Drata access-review workflow + SCIM diff exports | Compliance PM |
| CC6.3 Deprovisioning within 24h of termination | Auditor samples 25 terminations | HRIS → SCIM → Keycloak deactivation timestamp | HR + Compliance PM |
| CC6.6 Network segmentation | Auditor reviews Cilium policies + VPC architecture | `gitops/.../cilium/`, Terraform plans | Platform Lead |
| CC6.7 Encryption in transit + at rest | Auditor reviews Istio mTLS config + KMS keys | Helm values + Terraform state | DevSecOps |

### Common Criteria — System Operations (CC7)

| Control | Test | Evidence | Owner |
|---|---|---|---|
| CC7.1 Vulnerability management | Auditor reviews CVE SLA adherence | Trivy Operator + Grype reports; auto-PR history | DevSecOps |
| CC7.2 Logging + monitoring | Auditor verifies log retention + integrity | Loki retention config + S3 Object Lock | SRE |
| CC7.4 Incident response | Auditor reviews ≥ 5 incidents end-to-end | PagerDuty + Incident Analysis Agent timelines + RCAs | SRE |
| CC7.5 BCDR | Auditor reviews DR drill results | DR runbooks + Velero restore logs | SRE + Platform |

### Common Criteria — Change Management (CC8)

| Control | Test | Evidence | Owner |
|---|---|---|---|
| CC8.1 Change management | Auditor samples 40 production changes | Argo CD sync events + GitHub PR records + CODEOWNERS approvals | Platform |
| CC8.1.2 Separation of duties | Auditor verifies same identity cannot author + approve + deploy | Gatekeeper `K8sSod` constraint logs | DevSecOps |

### Availability (A1)

| Control | Test | Evidence | Owner |
|---|---|---|---|
| A1.1 Capacity planning | Quarterly capacity report | Grafana + Karpenter reports | SRE |
| A1.2 BCP / DR | Quarterly DR drill | Velero + DB restore logs + cross-AZ drain | SRE |
| A1.3 Backup integrity | Quarterly restore test | Restore validation reports | SRE |

### Confidentiality (C1)

| Control | Test | Evidence | Owner |
|---|---|---|---|
| C1.1 Data classification + handling | Sample 25 PII records, verify ABAC + redaction | OPA ABAC logs + OTel collector redaction sampling | DevSecOps + Compliance |
| C1.2 Secure disposal | Sample 10 retired media events | KMS key destruction logs + AWS data destruction attestations | Cloud Infra |

## Evidence Collection Schedule

| Week | Drata milestones |
|---|---|
| 1 | Drata production tenancy; first 6 connectors: K8s (`mgmt-use1`), GitHub Org, Keycloak, Vault, AWS Org, JAMF |
| 2 | Connectors: Splunk, Intune, Harbor, ECR, Crowdstrike (if applicable). 40 controls mapped |
| 3 | Custom controls authored for K8s-specific evidence (Kyverno policy enforcement, Cosign verify counts) |
| 4 | Mock auditor walkthrough #1 — gap list documented |
| 5 | Gap remediation; auditor engagement letter signed |
| 6 | Mock auditor walkthrough #2 — gap list closed to ≤ 5 items |
| 7 | Pre-audit readiness review with selected auditor |
| 8 | **Audit window opens** (TSC observation period starts) |
| 9–11 | Auditor walkthroughs / interviews |
| 12 | Mid-period checkpoint; address any operating-effectiveness gaps before they accumulate |

## Specific Controls Implemented in Phase 1 (mapped to evidence)

| Phase-1 deliverable | Maps to controls |
|---|---|
| Kyverno verifyImages (Phase 0 + enforce in nonprod Phase 1) | CC7.1, CC8.1, supply-chain |
| Vault HA + KMS auto-unseal | CC6.1, CC6.7 |
| Cilium default-deny + L7 NetworkPolicy | CC6.6 |
| Istio STRICT mTLS | CC6.7 |
| OTel + Loki + Tempo + 7y S3 Object Lock | CC7.2, AU-* |
| GitOps via Argo + signed commits + CODEOWNERS + Gatekeeper SoD | CC8.1, separation-of-duties |
| Backstage scaffolders with secure defaults | CC8.1 (consistent change baseline) |
| Trivy Operator + auto-PR via Dependency Agent | CC7.1 |
| Velero backups + DR runbook | A1.2, A1.3, CC7.5 |

## Risks

| ID | Risk | Mitigation |
|---|---|---|
| SOC-R1 | Auditor finds evidence gap during walkthrough | Mock walkthroughs Weeks 4 + 6 + 7 |
| SOC-R2 | Drata connector lag for newly-added systems | Drata SLA + manual ingestion fallback |
| SOC-R3 | Type II requires ≥ 6-month observation; audit window slip risks Year-1 report timing | Buffer in plan; Week 8 open keeps issuance in Q3 2026 |
| SOC-R4 | Engineers bypass policy in good faith | Kyverno enforce mode + Gatekeeper SoD + audit log review weekly |
| SOC-R5 | Vendor sub-processor (AWS, GitHub, etc.) report unavailable | Collect bridge letters early |

## Acceptance criteria (Phase-1 gate)

- [ ] Drata in production tenancy with ≥ 12 connectors live and pulling.
- [ ] ≥ 80% of in-scope SOC 2 controls auto-evidenced.
- [ ] Auditor selected; engagement letter signed.
- [ ] Two mock auditor walkthroughs completed; gap list ≤ 5 items at Week 7.
- [ ] Audit window opened in Week 8.
- [ ] ISO 27001 Stage 1 readiness review completed alongside (shared evidence).

## Companion artifacts

- [`control-test-plan.md`](./control-test-plan.md) — full per-control test procedure.
- [`evidence-collection-schedule.md`](./evidence-collection-schedule.md) — weekly Drata connector + control rollout.
- Existing cross-framework mapping: [`/app/docs/compliance/control-mapping.md`](../../compliance/control-mapping.md)
