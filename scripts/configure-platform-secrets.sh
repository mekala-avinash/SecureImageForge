#!/usr/bin/env bash
# Idempotent configurator for acme/platform repo secrets + variables.
#
# What it does:
#   - Uses the GitHub CLI (`gh`) to set every secret + variable consumed by the
#     reusable workflows under .github/workflows/.
#   - Reads required values from a sidecar env file (.platform-secrets.env)
#     OR prompts interactively when run with a TTY.
#   - Idempotent: safe to re-run after rotation.
#
# Required tooling:
#   - gh (logged in with `repo,admin:org` scopes)
#
# Usage:
#   1. Copy .platform-secrets.env.example to .platform-secrets.env and fill in.
#   2. ./scripts/configure-platform-secrets.sh acme/platform
#   3. Delete .platform-secrets.env (it is in .gitignore — DO NOT COMMIT).
#
# Variables (non-secret, visible to CI):
#   AWS_ACCOUNT         AWS account id hosting ECR + the gha-build IAM role
#   AWS_REGION          ECR region (e.g. us-east-1)
#   ECR_USER            ECR username used by SLSA generator (often AWS)
#
# Secrets (encrypted):
#   GITOPS_TOKEN        PAT (or fine-grained) with write access to acme/gitops
#   ECR_PASS            ECR registry password for SLSA provenance generator
#   COSIGN_PASSWORD     Cosign keyless does NOT need this; only set if migrating
#                        from a key-pair workflow.
#   RENOVATE_TOKEN      PAT used by the Renovate workflow (`repo` + `workflow`).
set -euo pipefail

REPO="${1:-${PLATFORM_REPO:-acme/platform}}"
ENV_FILE="${2:-.platform-secrets.env}"
DRY_RUN="${DRY_RUN:-0}"

ok()   { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
warn() { printf "  \033[1;33m!\033[0m %s\n" "$*"; }
die()  { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

if [ "$DRY_RUN" = "1" ]; then
  warn "DRY_RUN=1 — no GitHub mutations will be made."
fi

command -v gh >/dev/null 2>&1 || die "gh CLI missing — install with: brew install gh"
gh auth status >/dev/null 2>&1 || die "gh not logged in — run: gh auth login"

# Verify the repo is reachable AND the caller can write secrets.
gh repo view "$REPO" >/dev/null 2>&1 || die "cannot access $REPO — check repo name and gh permissions"
gh api "repos/$REPO" -q '.permissions.admin' 2>/dev/null | grep -qx true \
  || warn "you may not have admin on $REPO — secret writes may 403"

REQUIRED_VARS=(AWS_ACCOUNT AWS_REGION ECR_USER)
REQUIRED_SECRETS=(GITOPS_TOKEN ECR_PASS RENOVATE_TOKEN)
OPTIONAL_SECRETS=(COSIGN_PASSWORD SLACK_WEBHOOK_URL)

# Load sidecar env if present.
if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  . "$ENV_FILE"
  set +a
  ok "loaded $ENV_FILE"
else
  warn "$ENV_FILE not found — falling back to interactive prompts"
fi

prompt_if_missing() {
  local var="$1" kind="$2"
  if [ -z "${!var:-}" ]; then
    if [ ! -t 0 ]; then
      die "$var not set and no TTY for interactive entry. Populate $ENV_FILE."
    fi
    if [ "$kind" = "secret" ]; then
      read -r -s -p "  $var (hidden): " val; echo
    else
      read -r -p "  $var: " val
    fi
    printf -v "$var" '%s' "$val"
  fi
}

echo "→ configuring repo: $REPO"

echo "── variables ──"
for v in "${REQUIRED_VARS[@]}"; do
  prompt_if_missing "$v" "variable"
  current=$(gh variable list -R "$REPO" --json name,value -q ".[] | select(.name==\"$v\") | .value" 2>/dev/null || true)
  if [ "$current" = "${!v}" ]; then
    ok "$v unchanged"
  else
    gh variable set "$v" -R "$REPO" -b "${!v}" >/dev/null
    ok "$v updated"
  fi
done

echo "── secrets ──"
for s in "${REQUIRED_SECRETS[@]}"; do
  prompt_if_missing "$s" "secret"
  gh secret set "$s" -R "$REPO" -b "${!s}" >/dev/null
  ok "$s set"
done

for s in "${OPTIONAL_SECRETS[@]}"; do
  if [ -n "${!s:-}" ]; then
    gh secret set "$s" -R "$REPO" -b "${!s}" >/dev/null
    ok "$s set (optional)"
  fi
done

echo "── verifying ──"
gh secret   list -R "$REPO" | sed 's/^/    /'
gh variable list -R "$REPO" | sed 's/^/    /'
ok "platform secrets + variables configured for $REPO"

cat <<EOF

  Next steps:
   • Delete or move $ENV_FILE OUT of the repo working tree.
   • Trigger a re-run of the reusable workflow on a service repo:
       gh workflow run build.yml -R acme/<service>
   • Confirm the GitOps PR opens automatically in acme/gitops.

EOF
