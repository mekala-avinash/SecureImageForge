# Managed Istio RFP — Vendor Cover Emails

> Send Day 5 (2026-01-11). Owner: Platform Lead. CC: VP Platform, Head of SRE.
> Each email links to the full RFP at `docs/leadership-review/day-0-5/06-managed-istio-rfp/rfp.md`.

---

## Email 1 — Tetrate

**To:** <tetrate-account-exec@tetrate.io>
**Cc:** vp-platform@acme.io, head-of-sre@acme.io
**Subject:** ACME — Managed Istio RFP — response requested by 2026-01-23

Hi <name>,

ACME is standing up a multi-region (us-east-1, eu-west-1, ap-south-1) Kubernetes platform with Istio service mesh as a core dependency. We are evaluating Tetrate Service Bridge, Google Anthos Service Mesh, and Solo.io Gloo Mesh to select a managed-Istio offering that will materially reduce our operational burden.

The full RFP is attached. Key requirements:

- **Topology:** Istio multi-primary across 3 regions, SPIFFE-federated trust domains.
- **Identity:** AuthorizationPolicy + RequestAuthentication with JWKS from Keycloak; cross-cluster identity without shared secrets.
- **Progressive delivery:** first-class Argo Rollouts integration (VirtualService traffic split).
- **Lifecycle:** Istio ≥ 1.23 within 60 days of upstream release; zero-downtime control-plane upgrades; CVE SLA ≤ 72h on HIGH/CRITICAL.
- **Compliance:** FIPS-validated build available (FedRAMP path); air-gapped install variant.
- **Operability:** GitOps-friendly (Helm + CRDs only; no UI-only configuration).
- **Observability:** OTel-native (no proprietary agents required).

**Required vendor deliverables**
1. Reference architecture for our 3-region topology.
2. 4–6 week POC plan covering multi-primary install, AuthorizationPolicy, Argo Rollouts canary, OTel pipeline.
3. CVE response process documentation.
4. Pricing for 50 / 200 / 1000 workloads (Y1/Y2/Y3).
5. Three reference customers, at least one in regulated industry.
6. Air-gapped install documentation.

**Timeline**
- 2026-01-11 — RFP issued (today).
- 2026-01-23 — Responses due.
- 2026-01-26 to 2026-02-06 — Scoring + reference calls.
- 2026-02-09 — POC kickoff with top 1–2 vendors.
- 2026-02-20 — Recommendation to CTO + VP Platform.

Please confirm receipt and a primary point of contact.

Best,
<Platform Lead name>
ACME Platform Engineering · platform@acme.io · <phone>

---

## Email 2 — Google Cloud (Anthos Service Mesh)

**To:** <google-anthos-account-exec@google.com>
**Cc:** vp-platform@acme.io, head-of-sre@acme.io
**Subject:** ACME — Managed Istio RFP — response requested by 2026-01-23

(Same body — substitute Anthos. Add explicit note: "We run primarily on AWS with secondary Azure; please clarify Anthos Attached Clusters maturity for non-GKE environments in your response.")

---

## Email 3 — Solo.io (Gloo Mesh)

**To:** <solo-account-exec@solo.io>
**Cc:** vp-platform@acme.io, head-of-sre@acme.io
**Subject:** ACME — Managed Istio RFP — response requested by 2026-01-23

(Same body — substitute Solo.io / Gloo Mesh. Add explicit note: "Please clarify which Gloo Mesh capabilities require Gloo Gateway adoption, and which are mesh-only.")

---

## Internal log entry
- RFP ID: `RFP-2026-002-ISTIO`
- Sent to: Tetrate, Google Cloud, Solo.io
- Confidentiality: Confidential — Restricted
- Self-managed baseline: separately scoped by Platform Lead for comparator
- Records retention: 7 years
