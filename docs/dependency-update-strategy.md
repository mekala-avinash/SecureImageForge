# Dependency update strategy

Reproducible, auditable, and automated. Every paved-road service inherits these
defaults via the Backstage scaffolder.

## Principles

1. **Pin everything.** Direct + transitive deps must be locked to specific versions/digests.
2. **One source of truth per language.** Lockfile committed to the repo.
3. **Bots, not humans, propose bumps.** Renovate Bot (or Dependabot) runs hourly.
4. **Bots, not humans, merge security bumps.** Renovate rules below auto-merge CVE patches that pass CI.
5. **Provenance over trust.** Every release is rebuilt from a pinned lockfile and signed.

## Per-language conventions

| Language | Manifest          | Lockfile                | Bot rule                                 |
|----------|-------------------|-------------------------|------------------------------------------|
| Python   | `requirements.txt`| `requirements.lock`     | `pip-compile --generate-hashes` on PR    |
| Node.js  | `package.json`    | `package-lock.json`     | `npm ci` enforced in CI                  |
| Go       | `go.mod`          | `go.sum`                | `go mod tidy && go mod verify`           |
| Java     | `pom.xml`         | (Maven resolves locked) | `mvn -B dependency:resolve --offline`    |
| Container| `Dockerfile`      | digest-pinned `FROM`    | renovate auto-PR on base-image bumps     |
| Helm     | `Chart.yaml`      | `Chart.lock`            | `helm dependency update` in CI           |

## Regenerating a lockfile

```bash
# Python
pip-compile --generate-hashes -o requirements.lock requirements.txt

# Node.js
npm install --package-lock-only

# Go
go mod tidy && go mod download && go mod verify

# Java (Maven)
mvn -B dependency:go-offline

# Helm
helm dependency update helm
```

## Renovate config (committed at repo root)

```json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:recommended", ":semanticCommits"],
  "schedule": ["before 6am on monday"],
  "lockFileMaintenance": { "enabled": true, "schedule": ["before 4am on monday"] },
  "vulnerabilityAlerts": { "enabled": true, "labels": ["security"] },
  "packageRules": [
    {
      "description": "Auto-merge CVE patches that pass CI",
      "matchUpdateTypes": ["patch"],
      "matchPackagePatterns": ["*"],
      "vulnerabilityAlerts": true,
      "automerge": true,
      "automergeStrategy": "squash"
    },
    {
      "description": "Group OTel ecosystem bumps",
      "matchPackagePatterns": ["^opentelemetry", "^@opentelemetry"],
      "groupName": "opentelemetry"
    },
    {
      "description": "Pin all Docker tags to digests",
      "pinDigests": true
    }
  ]
}
```

## Vulnerability scanning hooks

Every CI pipeline (GitHub Actions / GitLab CI / Azure DevOps) runs three scanners
on every PR; merging is blocked on any HIGH or CRITICAL finding:

| Scanner | Target           | Failure condition          |
|---------|------------------|----------------------------|
| Trivy   | container image  | `--severity CRITICAL,HIGH` |
| Grype   | container image  | `--fail-on high`           |
| OSV     | source SBOM      | `--fail-on-vuln`           |

## SBOM generation

A SPDX-JSON SBOM is produced and attested (`cosign attest --type spdxjson`) for
every image. The SBOM is published as a CI artifact and uploaded to the
internal SBOM store at `https://sbom.acme.io/<service>/<digest>.spdx.json`.

## Audit + retention

- Renovate PR history retained in Git (no force-pushes on `main`).
- Cosign signatures + Rekor transparency log entries are immutable and queryable:
  ```bash
  cosign tree <image>@<digest>
  ```
- Vulnerability scanner SARIF output flows to GitHub Code Scanning (Security tab).
