#!/usr/bin/env bash
# =============================================================================
# push-all.sh — Day 5 GitOps Repo Bootstrap (Platform Lead execution)
#
# Pushes /app/gitops-bootstrap/{gitops,platform-repo,tenants-repo} to:
#   - github.com/<ORG>/gitops
#   - github.com/<ORG>/platform
#   - github.com/<ORG>/tenants
#
# Requires:
#   - gh CLI authenticated as Org admin (gh auth login)
#   - git configured with signing key (gitsign or GPG/SSH)
#   - terraform >=1.5 for branch protection module
#   - ORG env var set (default: acme)
# =============================================================================
set -euo pipefail

ORG="${ORG:-acme}"
ROOT="$(cd "$(dirname "$0")" && pwd)"
TF_DIR="${ROOT}/platform-repo/modules/github-repo-protection"

declare -a REPOS=(gitops platform tenants)
declare -A SRC_DIR=(
  [gitops]="${ROOT}/gitops"
  [platform]="${ROOT}/platform-repo"
  [tenants]="${ROOT}/tenants-repo"
)

# ----------------------------------------------------------------------------
# 1. Create repos (idempotent)
# ----------------------------------------------------------------------------
for r in "${REPOS[@]}"; do
  echo ">> [1/4] Ensure ${ORG}/${r} exists"
  if ! gh repo view "${ORG}/${r}" >/dev/null 2>&1; then
    gh repo create "${ORG}/${r}" --private --confirm \
      --description "Enterprise Platform Transformation — ${r}"
  else
    echo "   already exists, skipping create"
  fi
done

# ----------------------------------------------------------------------------
# 2. Initialize local repos, commit (signed), push
# ----------------------------------------------------------------------------
for r in "${REPOS[@]}"; do
  src="${SRC_DIR[$r]}"
  echo ">> [2/4] Init + push ${src} → ${ORG}/${r}"
  pushd "$src" >/dev/null
  if [ ! -d .git ]; then
    git init -b main
  fi
  git add -A
  if ! git diff --cached --quiet; then
    git commit -S -m "feat: initial bootstrap

Enterprise Platform Transformation — Phase 0 Day 5.
Source-of-record bootstrap. Branch protection applied via Terraform.

Refs: docs/leadership-review/day-0-5/04-gitops-bootstrap/README.md"
  else
    echo "   nothing to commit"
  fi
  git remote remove origin 2>/dev/null || true
  git remote add origin "git@github.com:${ORG}/${r}.git"
  git push -u origin main
  popd >/dev/null
done

# ----------------------------------------------------------------------------
# 3. Apply branch protection + CODEOWNERS via Terraform
# ----------------------------------------------------------------------------
echo ">> [3/4] Apply branch protection (Terraform)"
pushd "$TF_DIR" >/dev/null
terraform init -input=false
terraform apply -input=false -auto-approve \
  -var "org=${ORG}" \
  -var 'repos=["gitops","platform","tenants"]'
popd >/dev/null

# ----------------------------------------------------------------------------
# 4. Verify
# ----------------------------------------------------------------------------
echo ">> [4/4] Verifying"
for r in "${REPOS[@]}"; do
  echo "   - ${ORG}/${r}:"
  gh api "repos/${ORG}/${r}/branches/main/protection" \
    --jq '{required_reviews:.required_pull_request_reviews.required_approving_review_count, signed:.required_signatures.enabled, linear:.required_linear_history.enabled, force_push:.allow_force_pushes.enabled, status_checks:.required_status_checks.contexts}'
done

echo
echo "✅ All three repos bootstrapped, pushed, and protected."
echo "   Apply Argo CD root App next:"
echo "   kubectl apply -n argocd -f gitops/platform/argocd/root-app.yaml"
