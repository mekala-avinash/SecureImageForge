# ACME Paved Road — Implementation Monorepo

> Production-grade implementation of the enterprise platform's paved road. New services land on this road and within ≤ 60 minutes get: hardened multi-arch image → signed + SBOM + SLSA L3 → GitOps deploy → OTel-instrumented → SLOs as code → policy-verified at admission.
>
> This tree is **code**. Governance/PM documents live under `/app/docs/`.

## Top-level layout

| Path | Purpose |
|---|---|
| [`platform/`](./platform/) | Cross-cutting code (Helm library chart, root manifests, ApplicationSet generators) |
| [`golden-path/`](./golden-path/) | Definition of the paved road, principles, adoption workflow |
| [`templates/`](./templates/) | Backstage scaffolders + copy-paste service skeleton |
| [`ci/`](./ci/) | Reusable CI workflows (GitHub Actions + GitLab CI) |
| [`images/`](./images/) | Hardened base + runtime + tooling Dockerfiles, BuildKit pipeline |
| [`infra/`](./infra/) | Terraform / OpenTofu modules |
| [`observability/`](./observability/) | OTel collector, Prometheus, Loki, Tempo, dashboards |
| [`security/`](./security/) | Kyverno, OPA, Falco, Cilium, supply-chain (Cosign/SBOM/SLSA) |
| [`developer-experience/`](./developer-experience/) | Devcontainer, docker-compose, `pavedroad` CLI, quickstart |
| [`reference-service/`](./reference-service/) | Fully working FastAPI service using every component |

## Quickstart

```bash
# One-time: install the CLI
pipx install -e developer-experience/pavedroad-cli

# Scaffold a new service onto the paved road
pavedroad new service --name orders-api --language python --team orders

# Local dev loop
cd orders-api && make dev   # devcontainer + docker-compose (Postgres/Redis/OTel)

# Push → reusable CI does build/scan/sign/SBOM/SLSA/push/GitOps-PR
git push
```

See `developer-experience/docs/QUICKSTART.md` for the full walkthrough.

## Pinned versions

| Layer | Version |
|---|---|
| Kubernetes | 1.31 |
| Helm | 3.16 |
| Argo CD / Rollouts | 2.13 / 1.7 |
| Istio | 1.23 |
| Cilium | 1.16 |
| Kyverno | 1.13 |
| OTel Collector Contrib | 0.110 |
| Cosign / Syft / Trivy | 2.4 / 1.16 / 0.56 |
| Terraform (OpenTofu) | ≥ 1.5 (1.8) |
| Backstage | 1.32 |
