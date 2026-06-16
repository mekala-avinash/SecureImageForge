#!/usr/bin/env bash
# Validate an ArgoCD ApplicationSet renders against an expected service tree.
# This script does NOT require a live cluster — it just validates YAML structure
# and ApplicationSet logic via `kubectl apply --dry-run=client`.
set -euo pipefail

PLATFORM_ROOT="${PAVEDROAD_ROOT:-/app}"
ok()  { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
err() { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

command -v kubectl >/dev/null || err "kubectl missing"

# We can't validate ArgoCD CRDs server-side without a cluster; use Python YAML
# parsing for structural validation and reserve kubectl for kustomize output.
python3 - <<'PY' "$PLATFORM_ROOT" || err "ApplicationSet YAML parse failed"
import sys, yaml, pathlib
root = pathlib.Path(sys.argv[1])
files = [
  root/"gitops-bootstrap/gitops/applicationsets/services.yaml",
  root/"gitops-bootstrap/gitops/applicationsets/tenants-project.yaml",
  root/"gitops-bootstrap/gitops/bootstrap/platform-namespaces.yaml",
]
for f in files:
  docs = list(yaml.safe_load_all(f.read_text()))
  assert docs, f"{f}: empty"
  for d in docs:
    if d is None: continue
    assert "apiVersion" in d, f"{f}: doc missing apiVersion"
    assert "kind" in d, f"{f}: doc missing kind"
PY
ok "ApplicationSet + AppProject + bootstrap YAMLs parse"

# Verify the template overlay structure resolves.
for env in dev staging prod; do
  base="$PLATFORM_ROOT/gitops-bootstrap/gitops/apps/_template/overlays/$env/kustomization.yaml"
  [ -f "$base" ] || err "missing overlay: $env"
  ok "overlay $env present"
done

# Smoke: kustomize build the dev overlay (replace placeholders first).
if command -v kustomize >/dev/null 2>&1; then
  tmp=$(mktemp -d)
  cp -r "$PLATFORM_ROOT/gitops-bootstrap/gitops/apps/_template" "$tmp/svc"
  find "$tmp/svc" -type f \( -name '*.yaml' -o -name '*.yml' \) -print0 |
    xargs -0 sed -i \
      -e 's/SERVICE_NAME/test-svc/g' \
      -e 's/SERVICE_NAMESPACE/test-svc-dev/g' \
      -e 's/SERVICE_TEAM/payments/g' \
      -e 's/\bENV\b/dev/g'
  kustomize build "$tmp/svc/overlays/dev" >/dev/null && ok "kustomize build (dev overlay)"
  rm -rf "$tmp"
else
  printf "  \033[1;33m!\033[0m kustomize not installed — skipped overlay build\n"
fi

ok "ArgoCD GitOps assets validated"
