# Operator runbook: 3-action handover

Execute these in order. Each step is idempotent — re-running is safe after
rotation or partial failure. Estimated wall-clock: **20 minutes** start to finish.

> Why you (not the agent): each action requires credentials only you control:
> a GitHub PAT with admin scope on `acme/*`, AWS account credentials, and an
> active SSH/HTTPS connection to push from your laptop.

---

## Action 1 — Populate `.platform-secrets.env` and run the configurator

```bash
cd /app                                     # or wherever you clone acme/platform
cp .platform-secrets.env.example .platform-secrets.env
$EDITOR .platform-secrets.env               # fill in 6 values (see below)
```

| Value          | How to obtain                                                                 |
|----------------|-------------------------------------------------------------------------------|
| `AWS_ACCOUNT`  | `aws sts get-caller-identity --query Account --output text`                   |
| `AWS_REGION`   | The region where your ECR registry lives (e.g. `us-east-1`).                  |
| `ECR_USER`     | Always `AWS` for ECR.                                                         |
| `GITOPS_TOKEN` | https://github.com/settings/personal-access-tokens/new → fine-grained PAT, expiry 90d, repository access `acme/gitops`, **Contents: R/W**, **Pull requests: R/W**. |
| `ECR_PASS`     | `aws ecr get-login-password --region $AWS_REGION` (note: 12h TTL — see OIDC note below). |
| `RENOVATE_TOKEN` | New fine-grained PAT, scope: **all `acme/*` repos**, Contents R/W, Pull Requests R/W, Workflows R/W. |

Then **dry-run** first to confirm:

```bash
DRY_RUN=1 ./scripts/configure-platform-secrets.sh acme/platform
```

Apply for real:

```bash
./scripts/configure-platform-secrets.sh acme/platform
rm .platform-secrets.env                    # IMPORTANT: do not commit
```

Expected output ends with `platform secrets + variables configured for acme/platform`
and a printed list of secret names + their last-updated timestamps.

---

## Action 2 — Push and watch the e2e workflow run

This repo is already prepared. From within the Emergent chat, click the
**"Save to GitHub"** button at the bottom of the chat to push every change you
see in this session up to `acme/platform`.

After the push:

```bash
# Tail the latest run of the e2e workflow
gh run watch -R acme/platform $(gh run list -R acme/platform -w e2e-paved-road -L 1 --json databaseId -q '.[0].databaseId')
```

You should see:
- `validate-static` job complete in ~2 min (pytest + validate-cli + validate-argocd).
- `validate-live` job complete in ~5-7 min (kind boot → ArgoCD core install → scaffold → apply → rollout assertion).

If `validate-live` fails the workflow uploads a `kubectl get all -A` dump as an
artifact — download from the failed-run page.

---

## Action 3 — Install Renovate App + lock down branch protection

Two sub-steps:

### 3a. Renovate

Pick one (both produce identical PRs):

| Option | What to do                                                                                                   |
|--------|--------------------------------------------------------------------------------------------------------------|
| **Renovate GitHub App (recommended)** | https://github.com/apps/renovate → Configure → choose `acme` org → "All repositories" or just `platform` + `gitops`. The committed `/app/renovate.json` is picked up automatically. |
| **Self-hosted Action**                | Already wired in `/app/.github/workflows/renovate.yml` — runs hourly on its own once `RENOVATE_TOKEN` (set in Action 1) is in place. No further setup. |

Verify by opening: https://github.com/acme/platform/issues/labels — you should
see `dependencies`, `security`, `automerge` labels appear on the first Renovate
PR within 1 hour.

### 3b. Branch protection

Dry-run to see exactly what will be set:

```bash
DRY_RUN=1 ./scripts/configure-branch-protection.sh acme/platform main
```

Apply:

```bash
./scripts/configure-branch-protection.sh acme/platform main
./scripts/configure-branch-protection.sh acme/gitops   main      # also protect the GitOps repo
```

This enforces (verify in the UI link printed at the end):

- ✅ `e2e-paved-road / validate-static` and `validate-live` required to pass.
- ✅ `reusable-build / *` (pre-flight, build, scan, sign-attest, helm-validate, grype-scan) required.
- ✅ 2 approving reviews + dismiss stale + CODEOWNERS required.
- ✅ Signed commits required.
- ✅ Linear history; no force-push; no deletion.
- ✅ Required conversation resolution.

---

## Sanity check (after all three)

Open a no-op PR to `acme/platform` — bump a README typo. You should see:

1. The e2e workflow run automatically.
2. The merge button stays disabled until `validate-static`, `validate-live`, and
   all `reusable-build / *` checks pass.
3. Renovate opens its first batch of dependency PRs within an hour (or use
   the manual `workflow_dispatch` of the renovate workflow to trigger now).

## OIDC migration (recommended follow-on)

`GITOPS_TOKEN` and `ECR_PASS` are long-lived. Migrate to OIDC federation:

```
.github/workflows/reusable-build.yml already requests `id-token: write`.
Create an AWS IAM role with trust policy keyed on
  token.actions.githubusercontent.com / sub == "repo:acme/<service>:*"
and grant it `ecr:PushImage` + `sts:AssumeRole`.
Then drop `ECR_PASS` from the platform secrets.
```

Full IAM trust policy template: see `/app/infra/modules/iam/oidc-github.tf`.
