# SOC 2 Type II — Control Test Plan (Per-Control Detail)

> Companion to `audit-window-plan.md`. This file is the working document the Compliance PM updates weekly as evidence pipelines come online.

For each in-scope control we capture: control id, statement, design (how implemented), operating effectiveness (how it runs continuously), evidence sources, sample size for auditor testing, frequency, owner.

## Schema

```yaml
- id: CC6.1
  category: Common Criteria
  statement: "The entity implements logical access security software ..."
  design:
    - "All human access is via OIDC SSO (Keycloak) with MFA enforced (WebAuthn preferred, TOTP fallback)."
    - "Workload identity uses SPIFFE/SPIRE + cloud-native (IRSA/Pod Identity / WIF)."
    - "Vault dynamic credentials (TTL ≤ 1h) for any database / cloud-API access."
  operating_effectiveness:
    - "Keycloak audit log streams to Splunk in < 60s of event."
    - "MFA enforcement validated daily by a Drata custom control."
  evidence_sources:
    - keycloak_audit_log
    - splunk_index:auth
    - drata_control:mfa_enforcement
    - scim_provisioning_log
  test_procedure:
    - "Auditor selects 25 user provisioning events across the observation window."
    - "Verifies each event has an associated SCIM record + Keycloak audit entry + MFA enrollment."
    - "Verifies deprovisioning within 24h for any termination event."
  sample_size: 25
  test_frequency: per-audit
  owner: Compliance PM
  remediation_sla:
    finding_severity_high: 7 days
    finding_severity_med:  30 days
```

## In-scope controls (Year 1)

(Each control below uses the schema above; abbreviated for brevity.)

### CC1 — Control Environment
- CC1.1 Demonstrates commitment to integrity (Code of Conduct + training)
- CC1.4 Demonstrates commitment to competence (training records)

### CC2 — Communication & Information
- CC2.1 Obtains/generates relevant info to support functioning of internal control (compliance dashboards)
- CC2.2 Internal comms (Slack policy + #platform-announcements record)
- CC2.3 External comms (status page + customer security portal)

### CC3 — Risk Assessment
- CC3.1 Specifies objectives (program charter)
- CC3.2 Identifies and analyzes risks (risk register, SteerCo cadence)
- CC3.4 Identifies + assesses changes (change management process)

### CC4 — Monitoring Activities
- CC4.1 Ongoing/separate evaluations (continuous evidence via Drata)
- CC4.2 Communicates control deficiencies (SteerCo + PagerDuty audit alerts)

### CC5 — Control Activities
- CC5.1 Selects + develops control activities (policy-as-code library)
- CC5.2 Selects + develops general controls over technology (CIS Benchmark, K8s baseline)
- CC5.3 Deploys through policies + procedures (Backstage TechDocs + Kyverno)

### CC6 — Logical & Physical Access
- CC6.1 Logical access (covered above)
- CC6.2 New user provisioning + periodic review (SCIM + Drata access-reviews)
- CC6.3 Removes access on termination
- CC6.4 Physical access (cloud — CSP SOC 2 report)
- CC6.6 Network segmentation (Cilium + VPC + namespace tenancy)
- CC6.7 Encryption + key management (Vault + KMS + Istio mTLS)
- CC6.8 Prevention/detection of unauthorized software (Kyverno verifyImages + Falco)

### CC7 — System Operations
- CC7.1 Vulnerability management
- CC7.2 Logging + monitoring (Loki + Splunk + 7y retention with Object Lock)
- CC7.3 Detects + analyzes security incidents (Falco + Tetragon + SIEM)
- CC7.4 Incident response (PagerDuty + Incident Agent + RCA)
- CC7.5 Continuity & recovery (Velero + DR drills)

### CC8 — Change Management
- CC8.1 Authorizes + tests + approves + implements changes (GitOps + signed commits + CODEOWNERS + Argo + SoD)

### CC9 — Risk Mitigation
- CC9.1 Identifies + mitigates risks of business disruption (DR + BCP)
- CC9.2 Vendor + business partner risk (vendor inventory + DPIA + SCCs)

### Availability
- A1.1 Capacity (Prom + Karpenter)
- A1.2 Backup + recovery (Velero + DB snapshots)
- A1.3 Tests business continuity (quarterly DR drill)

### Confidentiality
- C1.1 Data classification + handling
- C1.2 Disposal of confidential information (KMS destruction + AWS attestations)

## Working Tracker

| Control | Designed? | Implemented? | Evidence flowing? | Mock-tested? | Notes |
|---|---|---|---|---|---|
| CC1.1 | ✅ | ✅ | ✅ | ☐ | |
| CC1.4 | ✅ | ✅ | 🟡 | ☐ | LMS connector pending |
| CC6.1 | ✅ | ✅ | ✅ | ☐ | |
| CC6.2 | ✅ | ✅ | 🟡 | ☐ | Drata access review workflow Week 3 |
| CC6.3 | ✅ | ✅ | ✅ | ☐ | |
| CC6.6 | ✅ | ✅ | ✅ | ☐ | |
| CC6.7 | ✅ | 🟡 | 🟡 | ☐ | Istio prod cluster Week 9 |
| CC6.8 | ✅ | ✅ | ✅ | ☐ | |
| CC7.1 | ✅ | ✅ | ✅ | ☐ | |
| CC7.2 | ✅ | ✅ | ✅ | ☐ | |
| CC7.4 | ✅ | ✅ | 🟡 | ☐ | First sample of incidents Week 5 |
| CC7.5 | ✅ | 🟡 | ☐ | ☐ | First DR drill Week 6 |
| CC8.1 | ✅ | ✅ | ✅ | ☐ | |
| A1.1  | ✅ | ✅ | ✅ | ☐ | |
| A1.2  | ✅ | 🟡 | 🟡 | ☐ | Velero install Week 3 |
| A1.3  | ✅ | ☐ | ☐ | ☐ | First restore Week 6 |
| C1.1  | ✅ | ✅ | 🟡 | ☐ | ABAC log sampling pipeline Week 4 |
| C1.2  | ✅ | ✅ | ✅ | ☐ | |

Updated weekly in SteerCo by Compliance PM.
