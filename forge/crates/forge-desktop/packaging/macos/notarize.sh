#!/usr/bin/env bash
# macOS code-sign + notarize helper.
#
# Inputs (env):
#   APP_BUNDLE         path to SecureImageForge.app
#   SIGNING_IDENTITY   "Developer ID Application: <name> (<TEAMID>)"
#   APPLE_ID           Apple ID for notarytool
#   APPLE_TEAM_ID      Apple developer Team ID
#   APPLE_PASSWORD     app-specific password
#
# Run: ./notarize.sh
set -euo pipefail

: "${APP_BUNDLE:?APP_BUNDLE is required}"
: "${SIGNING_IDENTITY:?SIGNING_IDENTITY is required}"
: "${APPLE_ID:?APPLE_ID is required}"
: "${APPLE_TEAM_ID:?APPLE_TEAM_ID is required}"
: "${APPLE_PASSWORD:?APPLE_PASSWORD is required}"

ENTITLEMENTS="$(dirname "$0")/entitlements.plist"

echo "[notarize] codesign --deep --force --options runtime"
codesign --deep --force --timestamp \
    --options runtime \
    --entitlements "$ENTITLEMENTS" \
    --sign "$SIGNING_IDENTITY" \
    "$APP_BUNDLE"

echo "[notarize] verifying signature"
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"
spctl --assess --verbose=4 --type execute "$APP_BUNDLE" || true

ZIP="$(dirname "$APP_BUNDLE")/SecureImageForge.zip"
echo "[notarize] zipping → $ZIP"
ditto -c -k --keepParent "$APP_BUNDLE" "$ZIP"

echo "[notarize] submitting to Apple notary service"
xcrun notarytool submit "$ZIP" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_PASSWORD" \
    --wait

echo "[notarize] stapling ticket"
xcrun stapler staple "$APP_BUNDLE"

echo "[notarize] done"
