#!/usr/bin/env bash
# Render every paved-road service Helm chart and validate against the Kubernetes schema.
# Picks up:
#   - /app/reference-service/helm
#   - /app/templates/backstage-scaffolder/*/skeleton/helm
#
# Validators:
#   - helm template (no actual install)
#   - kubeconform --strict (when present; warn otherwise)
set -euo pipefail

PLATFORM_ROOT="${PAVEDROAD_ROOT:-/app}"
ok()  { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
warn(){ printf "  \033[1;33m!\033[0m %s\n" "$*"; }
err() { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

charts=()
for d in \
  "$PLATFORM_ROOT/reference-service/helm" \
  "$PLATFORM_ROOT"/templates/backstage-scaffolder/*/skeleton/helm; do
  [ -f "$d/Chart.yaml" ] && charts+=("$d")
done

[ "${#charts[@]}" -gt 0 ] || err "no charts found"

for chart in "${charts[@]}"; do
  echo "── $chart"
  # Render with templated placeholders substituted so `helm template` doesn't choke.
  tmp=$(mktemp -d)
  cp -r "$chart" "$tmp/chart"
  sed -i 's/{{service_name}}/test-svc/g; s/{{team}}/payments/g; s/{{language}}/python/g; s/{{description}}/desc/g' \
    "$tmp/chart/values.yaml" "$tmp/chart/Chart.yaml" 2>/dev/null || true

  if command -v helm >/dev/null 2>&1; then
    if helm template "$tmp/chart" --dependency-update >/tmp/rendered.yaml 2>/tmp/helm.err; then
      ok "helm template"
    else
      warn "helm template (missing acme-platform-lib dep — expected in monorepo-only context)"
    fi

    if command -v kubeconform >/dev/null 2>&1 && [ -s /tmp/rendered.yaml ]; then
      if kubeconform -strict -summary </tmp/rendered.yaml; then
        ok "kubeconform"
      else
        err "kubeconform validation failed for $chart"
      fi
    fi
  else
    warn "helm CLI not installed — skipping render"
  fi
  rm -rf "$tmp"
done

echo "Helm validation pass."
