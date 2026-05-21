#!/usr/bin/env bash
# =============================================================================
# buildkit-build.sh — Standard hardened build wrapper
# Produces: multi-arch OCI image + SBOM + SLSA provenance + Cosign signature
# Requires: buildkit, syft, cosign, jq, OIDC env (GITHUB_TOKEN or similar)
# =============================================================================
set -euo pipefail

: "${IMAGE_REPO:?registry.acme.io/team/svc}"
: "${IMAGE_TAG:?semver+gitsha}"
: "${DOCKERFILE:=Dockerfile}"
: "${PLATFORMS:=linux/amd64,linux/arm64}"
: "${CONTEXT:=.}"
: "${SOURCE_DATE_EPOCH:=$(git log -1 --pretty=%ct)}"
export SOURCE_DATE_EPOCH

IMAGE="${IMAGE_REPO}:${IMAGE_TAG}"

echo ">> [1/6] BuildKit hermetic multi-arch build"
buildctl build \
  --frontend dockerfile.v0 \
  --local context="${CONTEXT}" \
  --local dockerfile="$(dirname "${DOCKERFILE}")" \
  --opt filename="$(basename "${DOCKERFILE}")" \
  --opt platform="${PLATFORMS}" \
  --opt build-arg:BUILD_DATE="$(date -u --iso-8601=seconds -d @"${SOURCE_DATE_EPOCH}")" \
  --opt build-arg:VCS_REF="$(git rev-parse HEAD)" \
  --opt build-arg:VERSION="${IMAGE_TAG}" \
  --output type=image,name="${IMAGE}",push=true,rewrite-timestamp=true \
  --export-cache type=registry,ref="${IMAGE_REPO}:buildcache",mode=max \
  --import-cache type=registry,ref="${IMAGE_REPO}:buildcache"

DIGEST=$(crane digest "${IMAGE}")
echo "Image digest: ${DIGEST}"

echo ">> [2/6] Generate SBOM (SPDX)"
syft "${IMAGE_REPO}@${DIGEST}" -o spdx-json > sbom.spdx.json
syft "${IMAGE_REPO}@${DIGEST}" -o cyclonedx-json > sbom.cdx.json

echo ">> [3/6] Vulnerability scan (Trivy + Grype)"
trivy image --severity CRITICAL,HIGH --exit-code 1 --no-progress "${IMAGE_REPO}@${DIGEST}"
grype "${IMAGE_REPO}@${DIGEST}" --fail-on high

echo ">> [4/6] Cosign keyless sign (Fulcio OIDC → Rekor)"
COSIGN_EXPERIMENTAL=1 cosign sign --yes "${IMAGE_REPO}@${DIGEST}"

echo ">> [5/6] Attest SBOM"
COSIGN_EXPERIMENTAL=1 cosign attest --yes \
  --predicate sbom.spdx.json --type spdxjson \
  "${IMAGE_REPO}@${DIGEST}"
COSIGN_EXPERIMENTAL=1 cosign attest --yes \
  --predicate sbom.cdx.json --type cyclonedx \
  "${IMAGE_REPO}@${DIGEST}"

echo ">> [6/6] SLSA L3 provenance"
slsa-generator generate --artifact-digest "${DIGEST}" --artifact-name "${IMAGE_REPO}" > provenance.json
COSIGN_EXPERIMENTAL=1 cosign attest --yes \
  --predicate provenance.json --type slsaprovenance \
  "${IMAGE_REPO}@${DIGEST}"

echo "Build complete: ${IMAGE_REPO}@${DIGEST}"
