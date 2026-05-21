# Compliance Automation RFP — Vendor Cover Emails

> Send Day 5 (2026-01-11). Owner: Compliance PM. CC: CISO.
> Each email links to the full RFP at `docs/leadership-review/day-0-5/05-compliance-platform-rfp/drata-vanta-secureframe-eval.md`.

---

## Email 1 — Drata

**To:** <drata-account-exec@drata.com>
**Cc:** ciso@acme.io, compliance@acme.io
**Subject:** ACME — Compliance Automation RFP — response requested by 2026-01-23

Hi <name>,

ACME is launching a 12-month enterprise platform transformation that includes a SOC 2 Type II + ISO 27001 cycle in Year 1, PCI-DSS + HIPAA in Year 2, and FedRAMP Moderate readiness in Year 3. We are evaluating Drata, Vanta, and Secureframe to select a compliance automation platform that will continuously evidence these controls and reduce auditor on-site days by ≥ 70%.

The full RFP is attached (also available at: <internal SharePoint link>). Highlights:

- **Frameworks:** SOC 2, ISO 27001:2022, PCI-DSS v4.0, HIPAA, GDPR, FedRAMP Moderate.
- **Stack:** Kubernetes (EKS/AKS/GKE), GitHub Enterprise, AWS + Azure + GCP, Vault, Keycloak (OIDC/SAML/SCIM), Crowdstrike, JAMF.
- **Demo focus:** live K8s + GitHub connectors, custom controls, auditor portal, FedRAMP package generation, regulated-industry reference customer call.
- **Pilot:** 4-week pilot on `mgmt-use1` + 1 AWS sandbox account post-selection.
- **Decision authority:** CISO + Head of Compliance + CFO co-sign.

**Timeline**
- 2026-01-11 — RFP issued (today).
- 2026-01-23 — Responses due.
- 2026-01-26 to 2026-01-30 — Demos + reference calls.
- 2026-02-06 — Selection decision; pilot starts.

Please confirm receipt and a primary point of contact for our procurement team.

Best,
<Compliance PM name>
ACME Compliance · compliance@acme.io · <phone>

---

## Email 2 — Vanta

**To:** <vanta-account-exec@vanta.com>
**Cc:** ciso@acme.io, compliance@acme.io
**Subject:** ACME — Compliance Automation RFP — response requested by 2026-01-23

(Same body as Email 1 — substitute "Vanta" for "Drata".)

---

## Email 3 — Secureframe

**To:** <secureframe-account-exec@secureframe.com>
**Cc:** ciso@acme.io, compliance@acme.io
**Subject:** ACME — Compliance Automation RFP — response requested by 2026-01-23

(Same body as Email 1 — substitute "Secureframe" for "Drata".)

---

## Attachments to include
1. `drata-vanta-secureframe-eval.md` (rendered to PDF as `ACME-Compliance-RFP-v1.0.pdf`)
2. ACME standard MSA + DPA + SCC templates
3. Stack inventory (1-pager) — `compliance-stack-inventory.md` (to be drafted by Compliance PM Day 4)
4. Reference architecture diagram extract (C2 view) from `docs/platform/c4/c4-diagrams.md`

## Internal log entry to file in SharePoint / GRC tool
- RFP ID: `RFP-2026-001-COMPLIANCE`
- Sent to: Drata, Vanta, Secureframe
- Confidentiality: Confidential — Restricted
- Records retention: 7 years
