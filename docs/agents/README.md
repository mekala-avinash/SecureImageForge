# Multi-Agent Autonomous Engineering System

This directory specifies the 15 production agents that comprise the platform's autonomous engineering fleet. Every agent follows a common contract (see `agent-template.yaml`) and is governed by Temporal workflows, SPIFFE-issued identities, gVisor sandboxes, and immutable WORM audit logs.

## Catalog

| # | Agent | Spec File |
|---|---|---|
| 1 | Unit Testing | `01-unit-testing-agent.yaml` |
| 2 | Integration Testing | `02-integration-testing-agent.yaml` |
| 3 | Security Scanning | `03-security-scanning-agent.yaml` |
| 4 | Documentation | `04-documentation-agent.yaml` |
| 5 | Refactoring | `05-refactoring-agent.yaml` |
| 6 | CI/CD Optimization | `06-cicd-optimization-agent.yaml` |
| 7 | Deployment Automation | `07-deployment-automation-agent.yaml` |
| 8 | Monitoring & Observability | `08-observability-agent.yaml` |
| 9 | Dependency Management | `09-dependency-management-agent.yaml` |
| 10 | Incident Analysis | `10-incident-analysis-agent.yaml` |
| 11 | Cost Optimization | `11-cost-optimization-agent.yaml` |
| 12 | Infrastructure Drift | `12-infra-drift-agent.yaml` |
| 13 | Kubernetes Optimization | `13-k8s-optimization-agent.yaml` |
| 14 | API Contract Validation | `14-api-contract-agent.yaml` |
| 15 | Supply Chain Security | `15-supply-chain-agent.yaml` |

## Common Contract

See `agent-template.yaml` for the full schema. Highlights:

- **Identity:** SPIFFE SVID + cloud workload identity (IRSA/WIF).
- **Sandboxing:** gVisor (runsc) or Kata.
- **State:** Redis (hot), Postgres+pgvector (long-term), S3 (artifacts).
- **Coordination:** Temporal workflows.
- **Approval:** blast-radius tiered (auto / human-in-loop / CAB).
- **Audit:** every decision signed + logged to S3 Object Lock (7 years).
- **Rollback:** every mutating action registers a compensating inverse.

## Sequence: Cross-Agent Collaboration Example

```text
PR opened
   │
   ├─▶ Security Scan Agent  ── findings ──▶ Jira/Slack
   ├─▶ Unit Testing Agent   ── tests/cov ──▶ PR status check
   ├─▶ API Contract Agent   ── breaking?  ──▶ Consumer notify
   └─▶ Refactor Agent       ── complexity ──▶ suggestion comment
            │
            ▼ all green
   ┌─────────────────────────┐
   │ Supply Chain Sec Agent  │
   │ verifies signatures+SBOM│
   └────────────┬────────────┘
                │
   ┌────────────▼────────────┐         SLO breach?       ┌──────────────────┐
   │ Deployment Agent        │────────────────────────▶│ Incident Analysis │
   │ canary 5%→25%→50%→100%  │                          │ Agent              │
   └────────────┬────────────┘                          └──────────────────┘
                │                                              │
                ▼                                              ▼
        ┌──────────────────┐                          ┌──────────────────┐
        │ Observability    │                          │ Auto-rollback +  │
        │ Agent (dashboards)│                         │ blameless RCA    │
        └──────────────────┘                          └──────────────────┘
```
