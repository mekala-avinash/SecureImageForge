# License Register

The project ships under **Apache-2.0** and only depends on permissively-licensed code.

## Allowed dependency licenses

Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, MPL-2.0, Zlib, Unicode-3.0 / Unicode-DFS-2016, CC0-1.0.

## Denied

GPL-2.0, GPL-3.0, AGPL-3.0, LGPL-2.1, LGPL-3.0, SSPL-1.0, BUSL-1.1, anything copyleft.

Enforced in CI via `cargo deny check` (config at `forge/deny.toml`).

## Bundled third-party tools

| Tool | Version pin | License | Source of truth |
|---|---|---|---|
| BuildKit (`buildkitd`, `buildctl`) | TBD (Phase 1) | Apache-2.0 | https://github.com/moby/buildkit |
| Trivy | TBD | Apache-2.0 | https://github.com/aquasecurity/trivy |
| Syft | TBD | Apache-2.0 | https://github.com/anchore/syft |
| Grype | TBD | Apache-2.0 | https://github.com/anchore/grype |
| Cosign | TBD | Apache-2.0 | https://github.com/sigstore/cosign |
| OPA | TBD | Apache-2.0 | https://github.com/open-policy-agent/opa |
| in-toto | TBD | Apache-2.0 | https://github.com/in-toto/in-toto |

`xtask bundle-buildkit` (Phase 1) records exact SHA-256 digests in `forge/vendor/manifest.json` so bundles are reproducible.

## Self-SBOM

Each release attaches a CycloneDX SBOM generated with `cargo cyclonedx` and signed with `cosign`. Phase 5 wires this into the release pipeline.
