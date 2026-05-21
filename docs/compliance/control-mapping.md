# Compliance Control Mapping

Cross-framework mapping of platform controls. Each control row is **continuously evidenced** by an OPA policy bundle, Kyverno rule, IaC check, or telemetry pipeline. Evidence is collected automatically by Drata/Vanta connectors.

| # | Control | SOC 2 (TSC) | ISO 27001:2022 | PCI-DSS 4.0 | HIPAA Security Rule | GDPR | FedRAMP Moderate | Implementation Reference |
|---|---|---|---|---|---|---|---|---|
| 1 | MFA for all human access | CC6.1 | A.5.17 | 8.4 | §164.308(a)(5)(ii)(D) | Art.32 | IA-2(1)(2) | Keycloak + WebAuthn; Kyverno: only OIDC-authenticated kubectl |
| 2 | Quarterly access reviews | CC6.2 | A.5.18 | 7.2.4 | §164.308(a)(4) | Art.32 | AC-2(j) | Drata workflow + SCIM diff job |
| 3 | Encryption in transit (TLS 1.3) | CC6.7 | A.8.24 | 4.1 | §164.312(e)(1) | Art.32 | SC-8(1) | Istio mTLS STRICT, gateway TLS 1.3-only |
| 4 | Encryption at rest (KMS) | CC6.7 | A.8.24 | 3.5 | §164.312(a)(2)(iv) | Art.32 | SC-28(1) | EBS/RDS CMK; app-level for PII; BYOK supported |
| 5 | Centralized logging w/ integrity | CC7.2 | A.8.15 | 10.x | §164.308(a)(1)(ii)(D) | Art.32 | AU-2/4/6/9 | Loki + Splunk HEC; S3 Object Lock 7y |
| 6 | Vulnerability mgmt SLA | CC7.1 | A.8.8 | 6.3.3 | §164.308(a)(5)(ii)(B) | Art.32 | RA-5 | Trivy/Grype + Dep Agent auto-PR + SLA dashboards |
| 7 | Change mgmt + SoD | CC8.1 | A.8.32 | 6.4.x | §164.308(a)(8) | Art.32 | CM-3 / AC-5 | GitOps + signed commits + Gatekeeper SoD constraint |
| 8 | Incident response | CC7.4 | A.5.24 | 12.10 | §164.308(a)(6) | Art.33/34 | IR-4/6/8 | PagerDuty + Incident Agent + RCA template |
| 9 | DR / BCP | A1.2 | A.5.29/A.5.30 | 12.10.1 | §164.308(a)(7) | Art.32 | CP-2/4/9/10 | Multi-region active-active; quarterly DR drills |
| 10 | Data classification | CC6.1 | A.5.12 | 3.1 | §164.514 | Art.30 | RA-2 | Tags + OPA ABAC enforcement; DLP at egress |
| 11 | Supply chain integrity (SLSA L3) | CC8.1 | A.8.30 | 6.3.2 | §164.308(a)(8) | Art.32 | SR-3/4/11 | Cosign + Rekor + Kyverno verifyImages |
| 12 | Secrets management | CC6.1 | A.5.17 | 8.6 | §164.312(d) | Art.32 | IA-5 | Vault dynamic; CSI Secrets Store; never in env |
| 13 | Patch mgmt | CC7.1 | A.8.8 | 6.3.3 | §164.308(a)(5)(ii)(B) | Art.32 | SI-2 | Nightly base rebuilds + Dep Agent |
| 14 | Network segmentation | CC6.6 | A.8.20-22 | 1.x | §164.312(c) | Art.32 | SC-7 | Cilium default-deny + namespace tiers + PCI cluster |
| 15 | Data subject rights (DSAR) | n/a | A.5.34 | n/a | n/a | Art.15-22 | n/a | Backstage scaffold + workflow; 30d SLA |
| 16 | Audit log retention 7y | CC7.2 | A.8.15 | 10.5 | §164.316(b)(2)(i) | Art.30 | AU-11 | S3 Object Lock (Compliance), KMS-encrypted |
| 17 | Vendor / third-party risk | CC9.2 | A.5.19-A.5.22 | 12.8.x | §164.308(b)(1) | Art.28 | SA-9 | Vendor inventory + DPIA + contractual SCC |
| 18 | Backup & recovery test | A1.3 | A.8.13 | 12.10.1 | §164.308(a)(7)(ii)(D) | Art.32 | CP-9/10 | Velero + DB snapshots; quarterly restore tests |
| 19 | Workforce training | CC1.4 | A.6.3 | 12.6 | §164.308(a)(5)(i) | Art.39 | AT-2/3 | Annual + role-based, tracked in HRIS |
| 20 | Physical security (cloud) | CC6.4 | A.7 | 9 | §164.310 | Art.32 | PE-* | Inherited from CSP (AWS/Azure/GCP) — SOC2 reports collected |

## Continuous Evidence Pipeline

```
[Source systems]
  K8s API · Vault · GitHub · IdP · Cloud APIs · MDM · Endpoint EDR
            │
            ▼
[Drata / Vanta / Secureframe connectors]
            │
            ▼
[Control evidence store]
  immutable S3 + reports
            │
            ▼
[Auditor portal] (read-only, scoped, time-boxed access via Vault dynamic creds)
```

## Separation of Duties (concrete examples)

| Activity | Role |
|---|---|
| Code author | dev_engineer |
| Code reviewer (≥1, ≥2 for /security or /payment) | sr_engineer (different from author) |
| Build pipeline operator | ci_system (machine identity, OIDC) |
| Deploy approver (HIGH+) | release_manager (different from author) |
| Production access (break-glass) | sre_oncall + manager (2-person rule) |
| Audit log access | auditor (read-only, time-boxed) |

Enforced by:
- CODEOWNERS + branch protection (≥2 reviewers).
- Gatekeeper `K8sSod` constraint (author != approver).
- Vault break-glass policy (requires 2 approvers + session recording).
