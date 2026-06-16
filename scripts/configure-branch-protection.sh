#!/usr/bin/env bash
# Idempotently configure GitHub branch protection on the platform repo so that
# the e2e and required CI checks must pass before merging to main.
#
# Usage:
#   ./scripts/configure-branch-protection.sh acme/platform [branch]
#   DRY_RUN=1 ./scripts/configure-branch-protection.sh acme/platform
#
# What it sets on the target branch:
#   - Required status checks (strict, all listed below)
#   - 2 required approving reviews + dismiss stale + require CODEOWNERS
#   - Signed-commits required
#   - Linear history
#   - No force-push / no deletion
#   - Conversation resolution required
set -euo pipefail

REPO="${1:-${PLATFORM_REPO:-acme/platform}}"
BRANCH="${2:-main}"
DRY_RUN="${DRY_RUN:-0}"

ok()   { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
warn() { printf "  \033[1;33m!\033[0m %s\n" "$*"; }
die()  { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

command -v gh >/dev/null 2>&1 || die "gh CLI missing"
gh auth status >/dev/null 2>&1 || die "gh not logged in"

# Required check names — these must match the workflow `jobs.<id>.name` (or id).
read -r -d '' REQUIRED_CHECKS <<'JSON' || true
[
  "e2e-paved-road / validate-static",
  "e2e-paved-road / validate-live",
  "reusable-build / pre-flight",
  "reusable-build / build",
  "reusable-build / scan",
  "reusable-build / sign-attest",
  "reusable-build / helm-validate",
  "reusable-build / grype-scan"
]
JSON

# Build PATCH payload for the GitHub branch-protection API.
PAYLOAD=$(cat <<JSON
{
  "required_status_checks": {
    "strict": true,
    "checks": $(echo "$REQUIRED_CHECKS" | python3 -c 'import json,sys; print(json.dumps([{"context": c} for c in json.load(sys.stdin)]))')
  },
  "enforce_admins": true,
  "required_pull_request_reviews": {
    "required_approving_review_count": 2,
    "dismiss_stale_reviews": true,
    "require_code_owner_reviews": true,
    "require_last_push_approval": true
  },
  "restrictions": null,
  "required_linear_history": true,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "required_conversation_resolution": true,
  "lock_branch": false,
  "allow_fork_syncing": false,
  "block_creations": false,
  "required_signatures": true
}
JSON
)

echo "→ Configuring branch protection: $REPO@$BRANCH"
echo "$PAYLOAD" | python3 -m json.tool >/dev/null || die "invalid JSON payload"

if [ "$DRY_RUN" = "1" ]; then
  warn "DRY_RUN=1 — printing payload only, NOT calling GitHub API."
  echo "$PAYLOAD" | python3 -m json.tool
  exit 0
fi

# GitHub branch-protection requires PUT, not PATCH, and the v3 API.
echo "$PAYLOAD" | gh api -X PUT \
  -H "Accept: application/vnd.github+json" \
  "/repos/$REPO/branches/$BRANCH/protection" --input - >/dev/null

ok "branch protection applied"

# Also enable required_signatures via dedicated endpoint (some orgs need both).
gh api -X POST \
  -H "Accept: application/vnd.github+json" \
  "/repos/$REPO/branches/$BRANCH/protection/required_signatures" >/dev/null 2>&1 \
  && ok "required_signatures enabled" || warn "required_signatures already set or not permitted"

echo
echo "  Verify in the UI:  https://github.com/$REPO/settings/branches"
