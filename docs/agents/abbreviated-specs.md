# Abbreviated Specs — Agents 2, 4, 5, 6, 8, 11, 12, 13, 14

## 02 Integration Testing Agent
- **Responsibilities:** spin up ephemeral preview env, run Pact contract tests, Playwright e2e, k6 smoke. Block deploy on fail.
- **Triggers:** pre-deploy gate (Argo CD pre-sync hook).
- **Approval:** block on fail; auto pass on green.
- **Metrics:** suite_duration, flake_rate, gate_block_total.

## 04 Documentation Agent
- **Responsibilities:** detect API changes (OpenAPI diff), generate TechDocs, scaffold ADRs, update README sections marked with `<!-- agent:auto -->`. Translate to required languages.
- **Triggers:** code merge to main, openapi spec change.
- **Approval:** tech writer review for non-trivial diffs.
- **Outputs:** PR with docs updates.

## 05 Refactoring Agent
- **Responsibilities:** weekly complexity scan (CodeClimate/Sonar metrics), propose refactors with semantic-preservation proofs (regression test runs + property-based tests).
- **Approval:** senior engineer review; blast_radius = medium minimum.
- **Limits:** max files touched per PR = 5; max LOC delta = 400.

## 06 CI/CD Optimization Agent
- **Responsibilities:** mine pipeline metrics; propose changes (cache keys, parallelism, matrix pruning, runner sizing). Open PRs against pipeline files.
- **Triggers:** weekly, on pipeline-duration regression (>15% over 14d baseline).
- **Approval:** platform team review.

## 08 Monitoring & Observability Agent
- **Responsibilities:** when a new service is registered in Backstage, generate dashboards (Grafana JSON), alerts (Prom rules), SLOs (Sloth), runbook scaffold. Detect drift between dashboards-as-code and live state.
- **Approval:** auto for additive; human for removals.

## 11 Cost Optimization Agent
- **Responsibilities:** pull billing daily (AWS CUR / Azure Cost / GCP BQ), correlate with K8s usage (Kubecost), open right-sizing PRs, identify idle nonprod, recommend Savings Plans.
- **Approval:** FinOps + owner.
- **Metrics:** projected_savings_usd, realized_savings_usd, recommendations_accepted_ratio.

## 12 Infrastructure Drift Agent
- **Responsibilities:** hourly `terraform plan` / `pulumi preview` against state; if drift detected, file report + remediation PR (revert manual change). Optionally auto-revert in nonprod.
- **Approval:** IaC owner.

## 13 Kubernetes Optimization Agent
- **Responsibilities:** use KRR / VPA recommendations + Prom metrics to propose CPU/memory request/limit changes; HPA tuning; node affinity adjustments.
- **Approval:** service owner.
- **Outputs:** PR against Helm values / Kustomize overlays.

## 14 API Contract Validation Agent
- **Responsibilities:** diff OpenAPI/AsyncAPI specs across versions, classify changes (additive/breaking), notify registered consumers, gate breaking changes on consumer ack or major version bump.
- **Triggers:** spec change in repo; consumer subscribed.
- **Approval:** API Council on breaking.
