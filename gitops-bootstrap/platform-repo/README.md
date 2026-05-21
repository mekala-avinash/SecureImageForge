# acme/platform — Reusable Modules, Charts, Compositions

This repository is the **toolbox** the platform team uses to build the gitops repo. It does **not** deploy anything directly. It publishes:

- **Terraform / OpenTofu modules** under `modules/` (VPC, EKS, IAM/IRSA, KMS, GitHub repo protection, etc.).
- **Crossplane XRDs + Compositions** under `crossplane/` for cloud control planes managed declaratively from K8s.
- **Reusable Helm charts** under `helm-charts/` (`microservice`, `worker`, `job`, `cronjob`, `kafka-consumer`, `grpc-service`).

## Versioning

- All modules are semver-tagged (`v1.2.3`).
- Breaking changes require a major bump + CHANGELOG entry + 2-week deprecation notice.
- Consumers pin to a major + minor (e.g., `~> 1.2`).

## Quality gates

- Terratest / OpenTofu test suite per module.
- Helm chart unit tests (`helm unittest`).
- Conftest policy bundle applied to chart-rendered YAML.
- Renovate keeps base-chart and provider versions current.

## Layout

```
platform-repo/
├── modules/           # Terraform / OpenTofu modules
├── crossplane/        # XRDs + Compositions
└── helm-charts/       # Reusable Helm charts
```
