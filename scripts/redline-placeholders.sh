#!/usr/bin/env bash
# =============================================================================
# redline-placeholders.sh — Program PM helper for minute-redline pass
#
# Surveys the leadership-review docs and produces:
#   1. A single CSV of every placeholder occurrence (file:line:placeholder)
#   2. A redline form (markdown) the PM fills in once
#   3. An apply mode that substitutes from the filled-in form across all docs
#
# Usage:
#   ./redline-placeholders.sh survey         # produce placeholders.csv + redline-form.md
#   ./redline-placeholders.sh apply          # read redline-form.md → substitute everywhere
# =============================================================================
set -euo pipefail

ROOT="${REPO_ROOT:-/app}"
TARGET="${ROOT}/docs/leadership-review"
OUT_CSV="${ROOT}/scripts/placeholders.csv"
OUT_FORM="${ROOT}/scripts/redline-form.md"

cmd="${1:-survey}"

if [[ "$cmd" == "survey" ]]; then
  echo "file,line,placeholder" > "$OUT_CSV"
  # Find every <token>, <name>, <date>, <phone>, <link> style placeholder
  grep -RnoE '<(name|date|phone|link|Procurement lead|VP Platform name|Platform Lead name|Compliance PM name|year \(TBD\)|TBD)>' "$TARGET" 2>/dev/null \
    | awk -F: '{ printf "%s,%s,%s\n", $1, $2, substr($0, index($0,$3)) }' >> "$OUT_CSV"

  unique_count=$(awk -F, 'NR>1{print $3}' "$OUT_CSV" | sort -u | wc -l)
  occurrences=$(($(wc -l < "$OUT_CSV") - 1))

  cat > "$OUT_FORM" <<EOF
# Redline form (fill in once, then run \`./redline-placeholders.sh apply\`)

> ${occurrences} placeholder occurrences across ${unique_count} unique tokens.
> Open \`${OUT_CSV}\` to see where each appears.

Fill in REAL values below. The script will substitute everywhere on apply.

\`\`\`yaml
# Names
vp_platform:        "REPLACE_ME (e.g., Alice Chen)"
cto:                "REPLACE_ME"
ciso:               "REPLACE_ME"
cfo_delegate:       "REPLACE_ME"
head_of_sre:        "REPLACE_ME"
head_of_compliance: "REPLACE_ME"
head_of_devex:      "REPLACE_ME"
finops_lead:        "REPLACE_ME"
vp_revenue:         "REPLACE_ME"
platform_lead:      "REPLACE_ME"
compliance_pm:      "REPLACE_ME"
program_pm:         "REPLACE_ME"
procurement_lead:   "REPLACE_ME"

# Contact details (used in vendor cover emails)
platform_lead_phone:    "+1-555-XXX-XXXX"
compliance_pm_phone:    "+1-555-XXX-XXXX"

# Dates (override defaults only if your kickoff was not on the assumed date)
day_0:        "2026-01-06"
day_3_steerco: "2026-01-09"
day_5:        "2026-01-11"
week_4_gate:  "2026-02-06"
retrospective: "2026-02-13"
phase_1_kickoff: "2026-02-09"
phase_1_mid_point: "2026-03-20"
phase_1_gate: "2026-05-08"
\`\`\`

When done, run:
\`\`\`bash
./redline-placeholders.sh apply
\`\`\`
EOF

  echo "✅ Survey complete:"
  echo "  - ${OUT_CSV}  (${occurrences} occurrences)"
  echo "  - ${OUT_FORM} (fill in REAL values, then run: $0 apply)"

elif [[ "$cmd" == "apply" ]]; then
  if [[ ! -f "$OUT_FORM" ]]; then
    echo "ERROR: ${OUT_FORM} not found. Run \`$0 survey\` first." >&2
    exit 1
  fi
  if grep -q "REPLACE_ME" "$OUT_FORM"; then
    echo "ERROR: ${OUT_FORM} still contains REPLACE_ME tokens. Edit the form first." >&2
    grep -n "REPLACE_ME" "$OUT_FORM" >&2
    exit 1
  fi

  # Parse YAML-ish form
  declare -A map
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*([a-z_]+):[[:space:]]*\"(.*)\"[[:space:]]*$ ]] || continue
    map["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
  done < "$OUT_FORM"

  # Apply substitutions across the leadership-review docs
  for f in $(find "$TARGET" -name "*.md"); do
    cp "$f" "$f.bak"
    # Generic placeholders → leave individual <name> alone (those are per-context),
    # but substitute role-bearing tokens.
    sed -i \
      -e "s|<VP Platform name>|${map[vp_platform]}|g" \
      -e "s|<VP Platform>|${map[vp_platform]}|g" \
      -e "s|<Platform Lead name>|${map[platform_lead]}|g" \
      -e "s|<Compliance PM name>|${map[compliance_pm]}|g" \
      -e "s|<Procurement lead>|${map[procurement_lead]}|g" \
      -e "s|<phone>|${map[platform_lead_phone]}|g" \
      "$f"
  done
  echo "✅ Applied. Backups in *.bak. Diff to review:"
  cd "$TARGET" && find . -name "*.bak" | head -5
  echo "Run: diff -u <file>.bak <file>   to inspect each change."
  echo "When happy:  find ${TARGET} -name '*.bak' -delete"
fi
