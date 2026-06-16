# Required secrets & variables on `acme/platform`

The reusable CI workflows in `ci/github-actions/`, `ci/gitlab-ci/`, and
`ci/azure-devops/` consume the following secrets/variables. Configure them
once on the platform repo; service repos inherit them via `secrets: inherit`
on `workflow_call`.

## Variables (non-secret — shown in CI logs)

| Variable      | Example         | Used by                                            |
|---------------|-----------------|----------------------------------------------------|
| `AWS_ACCOUNT` | `000000000000`  | OIDC role assumption + ECR registry path.          |
| `AWS_REGION`  | `us-east-1`     | ECR login + Karpenter Terraform.                   |
| `ECR_USER`    | `AWS`           | slsa-github-generator login.                       |

## Secrets (encrypted, not in logs)

| Secret             | Where to get it                                                                                                | Used by                              |
|--------------------|---------------------------------------------------------------------------------------------------------------|--------------------------------------|
| `GITOPS_TOKEN`     | Fine-grained PAT on `acme/gitops` (contents=R/W, pull_requests=W).                                            | `gitops-bump` stage (all 3 CIs).     |
| `ECR_PASS`         | `aws ecr get-login-password --region $AWS_REGION` (rotates every 12h — use OIDC role chaining in production). | `slsa-provenance` job.               |
| `RENOVATE_TOKEN`   | PAT (or GitHub App token) with `repo` + `workflow` on `acme/*`.                                               | `.github/workflows/renovate.yml`.    |
| `COSIGN_PASSWORD`  | (Optional) Only when migrating from key-pair signing. Sigstore keyless does not need it.                       | n/a by default.                      |

## Provisioning

```bash
cp .platform-secrets.env.example .platform-secrets.env  # populate the values
./scripts/configure-platform-secrets.sh acme/platform
rm .platform-secrets.env                                  # then delete the populated copy
```

The script is idempotent — re-run after rotation to update values in place.

## OIDC (recommended over long-lived PATs/passwords)

For both `GITOPS_TOKEN` and `ECR_PASS` you should migrate to **GitHub OIDC
federation** to AWS IAM roles. The CI snippet under
`.github/workflows/reusable-build.yml` already includes `id-token: write` and
configures the `aws-actions/configure-aws-credentials@v4` action with a
`role-to-assume`. The trust policy must restrict the role to
`repo:acme/<service>:*` and `actions:read` claims.

Once OIDC is in place the only secret left is `RENOVATE_TOKEN` (which the GH
App can replace too).
