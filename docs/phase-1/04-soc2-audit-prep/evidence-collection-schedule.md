# Evidence Collection Schedule — Phase 1 Weeks 1–12

> Compliance PM updates this weekly. Drata = compliance automation platform (selected Week 4 of Phase 0).

## Connector Onboarding

| Week | Connector | Owner | Status |
|---|---|---|---|
| 1 | Kubernetes (`mgmt-use1`) — agent-based | Compliance PM | ☐ |
| 1 | GitHub Org (`acme/*`) — App | Compliance PM | ☐ |
| 1 | Keycloak (SAML/OIDC audit feed) | Compliance PM + Identity | ☐ |
| 1 | Vault (audit log → Splunk → Drata) | Compliance PM + DevSecOps | ☐ |
| 1 | AWS Org (org-wide CloudTrail + Config) | Compliance PM + Cloud Infra | ☐ |
| 1 | JAMF (macOS endpoints) | Compliance PM + IT | ☐ |
| 2 | Splunk SIEM (alerts feed) | Compliance PM + SRE | ☐ |
| 2 | Intune (Windows endpoints) | Compliance PM + IT | ☐ |
| 2 | Harbor + ECR (image registry events) | Compliance PM + Platform | ☐ |
| 2 | Crowdstrike EDR | Compliance PM + Security | ☐ |
| 3 | Kubernetes (`prod-use1`) | Compliance PM + Platform | ☐ |
| 3 | Atlassian (Jira + Confluence) | Compliance PM + IT | ☐ |
| 4 | Slack (audit log) | Compliance PM + IT | ☐ |
| 4 | LMS (training records) | Compliance PM + People | ☐ |

## Custom Controls (built in Drata as policies-as-code)

| # | Control | Built? | Author |
|---|---|---|---|
| 1 | Kyverno admission denials weekly count + sample | ☐ | DevSecOps |
| 2 | Cosign verifyImages pass rate (prod) | ☐ | DevSecOps |
| 3 | Vault dynamic credential TTL median ≤ 1h | ☐ | DevSecOps |
| 4 | Argo CD sync events with CODEOWNERS approval evidence | ☐ | Platform |
| 5 | Gatekeeper SoD constraint hit rate | ☐ | DevSecOps |
| 6 | Pod Security "restricted" coverage % | ☐ | Platform |
| 7 | Cilium default-deny coverage % | ☐ | Platform |
| 8 | OTel collector PII redaction sampling | ☐ | SRE |
| 9 | Loki retention enforcement + S3 Object Lock proof | ☐ | SRE |
| 10 | Argo CD self-heal events (drift remediation) | ☐ | Platform |

## Workflows (Drata-managed)

| # | Workflow | Frequency | Owner |
|---|---|---|---|
| W1 | Quarterly access review (SCIM-driven) | Quarterly | Compliance PM |
| W2 | Onboarding security training | Per-hire | People |
| W3 | Annual security training | Annual | People |
| W4 | Vendor security review | Per-vendor + annual | Compliance PM |
| W5 | DSAR (GDPR Art 15-22) | Per-request, 30d SLA | Compliance PM |
| W6 | Incident response post-mortem evidence | Per-incident | SRE + Compliance |
| W7 | DR drill evidence | Quarterly | SRE |
| W8 | Backup restore evidence | Quarterly | SRE |

## Mock Auditor Walkthroughs

| # | Date | Lead | Outcome (post-meeting) |
|---|---|---|---|
| 1 | 2026-02-26 (Week 4) | Compliance PM | Gap list |
| 2 | 2026-03-20 (Week 6) | Compliance PM + selected auditor (informal) | Pre-window readiness |
| 3 | 2026-04-03 (Week 7) | Selected auditor | Engagement-letter pre-check |

## Bridge Letters (sub-processor SOC 2 reports)

| Provider | Letter received? | Period covered | Next renewal |
|---|---|---|---|
| AWS | ☐ | Year (TBD) | (TBD) |
| GitHub Enterprise | ☐ | | |
| Atlassian | ☐ | | |
| Splunk | ☐ | | |
| PagerDuty | ☐ | | |
| Cloudflare | ☐ | | |
| Drata | ☐ | | |

## Weekly Status Format (post in #platform-transformation)

```
SOC2 Week N:
- Connectors live: X/14
- Controls auto-evidenced: X/47 (target ≥ 80%)
- Custom controls built: X/10
- Mock walkthrough findings open: X (HIGH: x, MED: x, LOW: x)
- Bridge letters received: X/7
- Risks: <list>
- Asks for SteerCo: <list>
```
