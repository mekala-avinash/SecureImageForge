#!/usr/bin/env bash
# Build a Debian package for SecureImage Forge using fpm.
# Requires: fpm (https://github.com/jordansissel/fpm).
set -euo pipefail

VERSION="${1:-0.1.0}"
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
STAGE="$(mktemp -d)"

mkdir -p "$STAGE/usr/bin" "$STAGE/usr/share/applications"
cp "$ROOT/target/release/forge-desktop" "$STAGE/usr/bin/forge-desktop"
cp "$ROOT/target/release/forge"         "$STAGE/usr/bin/forge"
cp "$(dirname "$0")/forge-desktop.desktop" "$STAGE/usr/share/applications/"

fpm -s dir -t deb \
    --name secureimage-forge \
    --version "$VERSION" \
    --license Apache-2.0 \
    --vendor SecureImage \
    --description "Build, harden, and verify container images" \
    --url "https://github.com/secureimage/forge" \
    -C "$STAGE" \
    --depends "buildkit | docker.io" \
    --depends "trivy | aquasecurity-trivy" \
    .
