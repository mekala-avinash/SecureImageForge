# GitHub Repo Protection — Terraform Module

Applied automatically by `/app/gitops-bootstrap/push-all.sh` immediately after repos are created.

## What it does
- Enforces `main` branch protection across `acme/gitops`, `acme/platform`, `acme/tenants`.
- Requires ≥ 2 reviewers, CODEOWNERS approval, signed commits, linear history.
- Requires 5 status checks green: `lint`, `build`, `kubeconform`, `policy`, `secret-scan`.
- Disables force-push and branch deletion.
- Enables secret scanning + push protection.
- Restricts direct pushes to `main` to:
  - `/<org>/platform-admins` team
  - `<org>-argocd-bot` machine user (used by image-bump automation)

## Usage
```bash
cd /app/gitops-bootstrap/platform-repo/modules/github-repo-protection
terraform init
terraform apply -var "org=acme" -var 'repos=["gitops","platform","tenants"]'
```

## Required environment
- `GITHUB_TOKEN` with `admin:org` + `repo` scopes.

## Inputs

| Name | Type | Default | Description |
|---|---|---|---|
| `org` | string | — | GitHub organization (e.g. `acme`) |
| `repos` | list(string) | `["gitops","platform","tenants"]` | Target repos |
| `required_reviewers` | number | 2 | Min PR approvals |
| `required_status_checks` | list(string) | see main.tf | Required green checks |

## Outputs
- `protected_repos` — list of fully-qualified protected repos
- `required_status_checks` — echo of enforced checks
