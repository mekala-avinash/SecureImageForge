#!/usr/bin/env bash
# Validate the pavedroad CLI bootstrap workflow across all 4 supported languages.
# Pure file-system test: no cluster, no network.
set -euo pipefail

OUT=$(mktemp -d)
ok()  { printf "  \033[1;32m✓\033[0m %s\n" "$*"; }
err() { printf "  \033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

command -v pavedroad >/dev/null 2>&1 || err "pavedroad CLI not on PATH (pip install -e developer-experience/pavedroad-cli)"

for lang in python go nodejs java rust dotnet; do
  rm -rf "$OUT/$lang"; mkdir -p "$OUT/$lang"
  pavedroad new service --name "test-$lang" --language "$lang" --team payments --out "$OUT/$lang" >/dev/null
  for required in Dockerfile Makefile README.md catalog-info.yaml helm/Chart.yaml helm/values.yaml helm/templates/all.yaml; do
    [ -e "$OUT/$lang/test-$lang/$required" ] || err "$lang: missing $required"
  done
  # Pipeline coverage check
  [ -e "$OUT/$lang/test-$lang/.github/workflows/build.yml" ] || err "$lang: missing GitHub Actions workflow"
  case "$lang" in
    python) [ -e "$OUT/$lang/test-$lang/src/main.py" ]            || err "$lang: missing src/main.py" ;;
    go)     [ -e "$OUT/$lang/test-$lang/cmd/server/main.go" ]      || err "$lang: missing cmd/server/main.go" ;;
    nodejs) [ -e "$OUT/$lang/test-$lang/src/server.ts" ]           || err "$lang: missing src/server.ts" ;;
    java)   [ -e "$OUT/$lang/test-$lang/src/main/java/io/acme/Application.java" ] || err "$lang: missing Application.java" ;;
    rust)   [ -e "$OUT/$lang/test-$lang/src/main.rs" ]             || err "$lang: missing src/main.rs" ;;
    dotnet) [ -e "$OUT/$lang/test-$lang/Program.cs" ]              || err "$lang: missing Program.cs" ;;
  esac
  # Templating substitution check — no literal placeholders should remain
  if grep -rEl '\{\{service_name\}\}|\{\{team\}\}|\{\{language\}\}' "$OUT/$lang/test-$lang" >/dev/null; then
    err "$lang: untemplated {{...}} placeholders remain"
  fi
  ok "$lang scaffold complete"
done

# doctor check against the python scaffold
( cd "$OUT/python/test-python" && pavedroad doctor . >/dev/null ) && ok "doctor passes on rendered python scaffold" || err "doctor failed"
ok "CLI bootstrap workflow OK"
rm -rf "$OUT"
