# Compliance Automation Platform — Vendor Evaluation Brief

> Action #5 in Day 0–5 · Owner: Compliance PM · Co-owner: CISO · Target close: 4 weeks from kickoff

## 1. Objective
Select and engage a compliance automation platform that:
- Continuously evidences SOC 2, ISO 27001, PCI-DSS, HIPAA, GDPR (Year 1–2), and FedRAMP Moderate (Year 3).
- Integrates with our actual stack (K8s, GitHub, AWS/Azure/GCP, Vault, Keycloak, MDM).
- Reduces auditor on-site days by ≥ 70% via automation.

## 2. Shortlist
Three credible vendors in this space — evaluate all three.

| Vendor | Strengths | Watch-outs |
|---|---|---|
| **Drata** | Broad framework coverage incl. FedRAMP; strong K8s + cloud connectors; mature workflow engine | Pricing scales with employees + frameworks |
| **Vanta** | Fastest time-to-SOC2; best DX; large connector library | FedRAMP support more recent; PCI depth varies |
| **Secureframe** | Strong custom-controls + tailored programs; good GRC consultant network | Smaller ecosystem; some connectors via API only |

## 3. Evaluation criteria (weighted)

| Criterion | Weight | Notes |
|---|---|---|
| Framework coverage (SOC2/ISO/PCI/HIPAA/GDPR/FedRAMP) | 20% | Must support all 6 by program end |
| Native connectors for our stack | 20% | EKS/AKS/GKE, GitHub, AWS/Azure/GCP, Vault, Keycloak, JAMF/Intune, Crowdstrike |
| Custom controls + policy-as-code import | 15% | We will own a large OPA/Kyverno policy bundle |
| Auditor portal & evidence export | 10% | Time-boxed, scoped access; signed export bundles |
| Workflow automation (access reviews, vendor risk, training tracking) | 10% | Off-the-shelf workflows reduce burden |
| API maturity & webhook support | 10% | We will integrate with Backstage + Slack |
| Data residency options | 5% | EU residency mandatory for some tenants |
| Pricing transparency & 3-year TCO | 5% | Reject opaque pricing |
| Customer references in regulated industries | 5% | At least 2 references in FSI or Health |

## 4. Required demos (90 min each)
- Live K8s + GitHub connectors pulling evidence end-to-end.
- Custom control creation + evidence mapping.
- Auditor portal walkthrough.
- API/webhook walkthrough.
- FedRAMP package generation demo.
- Customer reference call (regulated industry, ≥ 200 employees).

## 5. Pilot scope (4 weeks, post-selection)
- Connect to 1 cluster, 1 AWS account, GitHub org, Vault, Keycloak.
- Map 40 SOC 2 + 20 ISO 27001 controls.
- Run a mock auditor scenario.
- Deliverable: evidence completeness report + gap list.

## 6. Required commercial terms
- MSA + DPA + standard SCCs.
- ≤ 30-day data export + termination assistance clause.
- SLA: 99.5% platform availability with credits.
- Pricing locked for 24 months.
- No "trueing up" mid-contract.

## 7. Timeline
- Week 1: kickoff RFP, send to 3 vendors.
- Week 2: receive RFP responses; schedule demos.
- Week 3: demos + reference calls.
- Week 4: recommendation to CISO + CFO; sign of selected vendor; pilot starts.

## 8. Stakeholders
- Decision: CISO + Head of Compliance + CFO (co-sign).
- Engineering input: VP Platform, Staff DevSecOps Engineer (when hired).
- Procurement: <Procurement lead>.

## 9. Out of scope
- Replacing existing SIEM (Splunk) or SOC tooling.
- Replacing Vault, Keycloak, or any technical control tooling — compliance platform observes; it does not run controls.
