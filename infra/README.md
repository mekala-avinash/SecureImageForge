# Infrastructure Modules

Production Terraform / OpenTofu modules. Each is self-contained, documented, and used as a building block by the per-environment root modules under `acme/gitops/clusters/`.

## Layout

```
infra/modules/
├── aws-eks-cluster/         # Hardened EKS (private endpoint, KMS, audit logs, no aws-auth)
├── aws-irsa/                # Generic IAM Roles for Service Accounts
├── aws-kms-bucket/          # KMS-encrypted S3 bucket with optional Object Lock + lifecycle
├── vault-bootstrap/         # Vault HA install + KMS auto-unseal (placeholder; uses official Helm)
└── argocd-bootstrap/        # Argo CD HA + OIDC + root App-of-Apps
```

## Architecture rationale

- **Composition.** Each module solves one thing; root modules wire them together.
- **No upstream-module re-implementations.** We wrap `terraform-aws-modules/*` with ACME-opinionated defaults rather than fork.
- **Compatible with OpenTofu 1.8+.** No HCL2-incompatible features.
- **OIDC throughout.** Cloud creds are short-lived; no static IAM users.

## Example usage (root module — lives in acme/gitops, not here)

```hcl
module "prod_use1" {
  source = "git::https://github.com/acme/platform.git//infra/modules/aws-eks-cluster?ref=v1.0.0"
  cluster_name       = "prod-use1"
  kubernetes_version = "1.31"
  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets
  tags = { Program = "EPT", Env = "prod" }
}

module "loki_bucket" {
  source = "git::https://github.com/acme/platform.git//infra/modules/aws-kms-bucket?ref=v1.0.0"
  name   = "acme-loki-chunks-use1"
  lifecycle_transitions = [
    { days = 30,  storage_class = "STANDARD_IA" },
    { days = 90,  storage_class = "GLACIER_IR" },
  ]
}

module "loki_irsa" {
  source = "git::https://github.com/acme/platform.git//infra/modules/aws-irsa?ref=v1.0.0"
  role_name           = "loki-s3-prod-use1"
  oidc_provider_arn   = module.prod_use1.oidc_provider_arn
  oidc_provider_url   = module.prod_use1.oidc_provider_url
  namespace           = "observability"
  service_account     = "loki"
  inline_policy_json  = data.aws_iam_policy_document.loki_s3.json
}

module "argocd" {
  source = "git::https://github.com/acme/platform.git//infra/modules/argocd-bootstrap?ref=v1.0.0"
  gitops_repo_url   = "https://github.com/acme/gitops"
  gitops_repo_branch = "main"
  root_app_path     = "clusters/${module.prod_use1.cluster_name}"
}
```

## Security considerations

- **EKS** endpoint is private-only; access via VPN or SSM session manager bastion.
- **KMS keys** have rotation enabled, 30-day deletion window, separate per bucket/cluster.
- **IRSA** module pins both `sub` and `aud` conditions on the trust policy.
- **S3 bucket** module enables Object Lock (Compliance mode, 7 years) when used for audit/compliance use cases.
- **Argo CD** is OIDC-only; admin access via Keycloak group `acme:platform-admins`.

## Versioning

Each module follows semver via Git tags (`v1.0.0`, etc.). Renovate keeps consumers current.
