# IAM GitHub OIDC module

Federates GitHub Actions workflows in `var.allowed_repos` to assume the
`gha-build` IAM role via short-lived STS tokens. **Replaces** long-lived
`ECR_PASS` + `GITOPS_TOKEN` secrets in the reusable CI workflow.

## Usage

```hcl
module "gha_oidc" {
  source        = "../../modules/iam-github-oidc"
  region        = "us-east-1"
  account_id    = "000000000000"
  allowed_repos = ["acme/platform", "acme/gitops", "acme/orders-api"]
  kms_key_arn   = module.kms.arn   # optional
}

output "role_arn" { value = module.gha_oidc.role_arn }
```

## Cutover

1. `terraform apply` this module.
2. In `acme/platform`, set the repo variable `AWS_ACCOUNT` (account_id) — the OIDC role assumption is referenced by the reusable workflow's `role-to-assume`.
3. Delete `ECR_PASS` from platform secrets (`gh secret delete ECR_PASS -R acme/platform`).
4. Re-run a service build; confirm the GitOps PR still opens.

## Caveats

- `thumbprint_list` must be refreshed periodically — GitHub rotates Sigstore certs once or twice a year. Run `aws iam list-open-id-connect-providers` and compare.
- `max_session_duration = 3600` is the cap for OIDC roles by default; bump only if your buildx step legitimately takes longer.
