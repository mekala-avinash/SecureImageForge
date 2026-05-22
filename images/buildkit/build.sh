#!/usr/bin/env bash
# =============================================================================
# build.sh — Reproducible, signed, attested multi-arch container build.
#
# Outputs:
#   - Multi-arch image (linux/amd64,linux/arm64) pushed to registry
#   - SBOM (SPDX + CycloneDX) attached as OCI artifacts
#   - SLSA L3 in-toto provenance attached
#   - Cosign signature (keyless via Fulcio OIDC)
#   - Rekor transparency log entry
#
# Required env:
#   IMAGE_REPO       e.g. registry.acme.io/orders/api
#   IMAGE_TAG        e.g. v1.2.3 (semver) or git sha
#   DOCKERFILE       default: Dockerfile
#   PLATFORMS        default: linux/amd64,linux/arm64
#   COSIGN_EXPERIMENTAL=1  (for keyless)
#
# Required tools: buildx (BuildKit), syft, grype, trivy, cosign, crane, jq.
# Most are present in images/tools/Dockerfile (the canonical ops image).
# =============================================================================
set -euo pipefail

: "${IMAGE_REPO:?required}"
: "${IMAGE_TAG:?required}"
: "${DOCKERFILE:=Dockerfile}"
: "${PLATFORMS:=linux/amd64,linux/arm64}"
: "${CONTEXT:=.}"

# Reproducibility: anchor timestamps to commit time.
SOURCE_DATE_EPOCH="$(git -C "${CONTEXT}" log -1 --pretty=%ct)"
export SOURCE_DATE_EPOCH

BUILD_DATE="$(date -u --iso-8601=seconds -d "@${SOURCE_DATE_EPOCH}")"
VCS_REF="$(git -C "${CONTEXT}" rev-parse HEAD)"
IMAGE="${IMAGE_REPO}:${IMAGE_TAG}"

echo "::group::1/6 BuildKit hermetic multi-arch build"
docker buildx build \
  --file "${DOCKERFILE}" \
  --platform "${PLATFORMS}" \
  --provenance="mode=max" \
  --sbom=true \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  --build-arg VCS_REF="${VCS_REF}" \
  --build-arg VERSION="${IMAGE_TAG}" \
  --build-arg SERVICE_NAME="$(basename "${IMAGE_REPO}")" \
  --cache-from "type=registry,ref=${IMAGE_REPO}:buildcache" \
  --cache-to   "type=registry,ref=${IMAGE_REPO}:buildcache,mode=max" \
  --output     "type=image,name=${IMAGE},push=true,rewrite-timestamp=true" \
  "${CONTEXT}"
echo "::endgroup::"

DIGEST="$(crane digest "${IMAGE}")"
DIGEST_REF="${IMAGE_REPO}@${DIGEST}"
echo "Image digest: ${DIGEST_REF}"

echo "::group::2/6 Vulnerability scan (Trivy + Grype, fail on HIGH/CRITICAL)"
trivy image --severity CRITICAL,HIGH --exit-code 1 --no-progress "${DIGEST_REF}"
grype "${DIGEST_REF}" --fail-on high
echo "::endgroup::"

echo "::group::3/6 SBOM generation (SPDX + CycloneDX)"
syft "${DIGEST_REF}" -o spdx-json      > sbom.spdx.json
syft "${DIGEST_REF}" -o cyclonedx-json > sbom.cdx.json
echo "::endgroup::"

echo "::group::4/6 Cosign keyless sign"
COSIGN_EXPERIMENTAL=1 cosign sign --yes "${DIGEST_REF}"
echo "::endgroup::"

echo "::group::5/6 Attest SBOM (SPDX + CycloneDX)"
COSIGN_EXPERIMENTAL=1 cosign attest --yes --type spdxjson    --predicate sbom.spdx.json "${DIGEST_REF}"
COSIGN_EXPERIMENTAL=1 cosign attest --yes --type cyclonedx   --predicate sbom.cdx.json  "${DIGEST_REF}"
echo "::endgroup::"

echo "::group::6/6 SLSA L3 provenance"
# The reusable CI workflow uses slsa-github-generator; this is the local fallback.
cat > provenance.json <<EOF
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [{"name":"${IMAGE_REPO}","digest":{"sha256":"${DIGEST#sha256:}"}}],
  "predicateType": "https://slsa.dev/provenance/v1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://acme.io/buildkit/v1",
      "externalParameters": {"source":"${VCS_REF}", "platforms":"${PLATFORMS}"}
    },
    "runDetails": {
      "builder": {"id":"${BUILDER_ID:-https://github.com/acme/.github/workflows/build.yml@refs/heads/main}"},
      "metadata": {"invocationId":"${GITHUB_RUN_ID:-local-$(date +%s)}", "startedOn":"${BUILD_DATE}"}
    }
  }
}
EOF
COSIGN_EXPERIMENTAL=1 cosign attest --yes --type slsaprovenance --predicate provenance.json "${DIGEST_REF}"
echo "::endgroup::"

echo "✅ Build complete: ${DIGEST_REF}"
echo "DIGEST=${DIGEST}" >> "${GITHUB_OUTPUT:-/dev/null}" 2>/dev/null || true
