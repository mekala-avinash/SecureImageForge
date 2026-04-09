# SecureImage Forge - Product Requirements Document

## Original Problem Statement
SecureImage Forge is an automated pipeline tool designed to build, harden, and verify Docker images for enterprise runtimes (Java, Node.js, .NET, Go). The product has multiple phases:
- Phase 1 (MVP): Basic pipelines, Trivy scanning, CLI & Web Dashboard
- Phase 2 (Enterprise): Advanced analytics, health scores, registry integrations
- Phase 3 (Advanced Security): Multi-architecture builds, keyless signing, custom policies
- Phase 4 (Enterprise Ecosystem): OPA integration, SLSA Level 3/4 compliance, automated VEX, Evergreen pipeline auto-updates
- Phase 4.5 (Granular Controls): Simple vs. Advanced UI toggle for dynamic versioning, binary stripping whitelists, FIPS mode, and custom Docker labels
- Phase 5 (Auto-Remediation): Automatic vulnerability remediation with one-click fixes

## Architecture
- **Frontend**: React with Tailwind CSS, React Router, Axios
- **Backend**: FastAPI with Motor (async MongoDB)
- **Database**: MongoDB
- **CLI**: Click-based CLI (`/app/forge`)

### Key Files
- `/app/backend/server.py` - Main FastAPI application (~1640 lines)
- `/app/frontend/src/App.js` - React router and main components
- `/app/frontend/src/components/EnhancedNewBuild.js` - Advanced build config form
- `/app/frontend/src/components/BuildDetail.js` - Build details with remediation UI
- `/app/backend/services/vulnerability_remediation.py` - CVE database and auto-fix logic
- `/app/backend/services/version_matrix.py` - Runtime version matrices
- `/app/backend/cli/forge_cli.py` - CLI interface

## What's Been Implemented

### Phase 1 - MVP (COMPLETE)
- [x] Basic build pipeline UI with runtime/base image selection
- [x] Trivy vulnerability scanning (MOCKED)
- [x] CycloneDX SBOM generation (MOCKED)
- [x] CLI tool (`forge build`, `forge list`, `forge scan`)
- [x] Web Dashboard with stats and recent builds
- [x] Build detail view with logs and scan results

### Phase 2 - Enterprise Expansion (COMPLETE)
- [x] Health score calculation per build
- [x] Analytics dashboard with charts
- [x] Node.js and Go runtime templates
- [x] Build history pagination

### Phase 3 - Advanced Security (COMPLETE)
- [x] Multi-architecture builds (amd64, arm64)
- [x] Keyless signing with Cosign (MOCKED)
- [x] OPA Policies UI page
- [x] Policy templates system

### Phase 4 - Enterprise Ecosystem (PARTIAL - MOCKED)
- [x] Service files created for OPA, SLSA, VEX, Drift Detection
- [ ] Actual implementation needed (currently returns hardcoded data)
- [ ] UI integration for Phase 4 features

### Phase 4.5 - Granular Runtime & OS Controls (COMPLETE - 2026-04-09)
- [x] EnhancedNewBuild component with Simple/Advanced toggle
- [x] Dynamic runtime version picker (Java 8/11/17/21, .NET 6/7/8, Go 1.20-1.22, Node 18-21)
- [x] Runtime distribution selector (Temurin, Corretto, Microsoft)
- [x] Base image tag pinning (Alpine 3.18-3.20, Debian 11-12, Distroless)
- [x] FIPS 140-2 mode toggle (when distribution supports it)
- [x] Binary whitelist ("Break Glass" feature)
- [x] Environment variable sanitization rules
- [x] CIS Benchmark Level selection (Level 1/Level 2)
- [x] Custom Docker labels for asset tracking
- [x] SBOM format selection (CycloneDX 1.4, SPDX 2.3)
- [x] SBOM scan depth configuration

### Phase 5 - Automatic Vulnerability Remediation (COMPLETE - 2026-04-09)
- [x] CVE Remediation Database (15+ known CVEs with auto-fix mappings)
  - Log4Shell, Spring4Shell, Commons Text RCE, curl/glibc/OpenSSL CVEs
  - Node.js, Java, Go, .NET specific vulnerabilities
- [x] Vulnerability Analysis API with remediation status per CVE
  - Auto-fixable, Patch Available, Manual Required classifications
  - Estimated remediation time calculation
- [x] One-Click Auto-Remediation
  - "Auto-Remediate All" button in vulnerability tab
  - Individual "Fix This" button per CVE
- [x] Generated Remediated Dockerfile
  - Base image upgrades
  - OS package security patches
  - Security metadata labels
  - Copy/Download functionality
- [x] Delta Scan Verification (simulated)
  - Shows original vs new vulnerability counts
  - Verification pass/fail status
- [x] Remediation Audit Trail
  - Full history of remediation actions
  - Compliance-ready logging
- [x] UI Enhancements
  - Remediation Summary (Total, Auto-Fixable, Patch Available, Manual Required)
  - AUTO-FIXABLE / PATCH AVAILABLE / BREAKING badges
  - Expandable CVE rows with descriptions and fix commands

## API Endpoints

### Build Management
- `POST /api/builds` - Create new build (accepts advanced config)
- `GET /api/builds` - List all builds
- `GET /api/builds/{id}` - Get build details
- `GET /api/stats` - Dashboard statistics

### Configuration Matrices
- `GET /api/runtime-versions` - All runtime version matrices
- `GET /api/runtime-versions/{runtime}` - Specific runtime versions
- `GET /api/base-image-tags` - All base image tags
- `GET /api/base-image-tags/{base}` - Specific base image tags
- `GET /api/cis-levels` - CIS benchmark level configurations
- `GET /api/sbom-formats` - SBOM format and scan depth options
- `POST /api/validate-runtime-config` - Validate runtime/version/distribution combo

### Vulnerability Remediation (NEW)
- `GET /api/builds/{id}/vulnerabilities/analysis` - Detailed analysis with remediation status
- `POST /api/builds/{id}/remediate` - Auto-remediate all fixable vulnerabilities
- `POST /api/builds/{id}/remediate/{cve_id}` - Remediate single CVE
- `GET /api/builds/{id}/remediation-history` - Remediation audit trail
- `GET /api/remediation/cve-database` - List all known CVEs with fixes
- `GET /api/remediation/stats` - Overall remediation statistics

### Policies & Analytics
- `GET /api/policies` - List policies
- `POST /api/policies` - Create policy
- `GET /api/analytics/*` - Various analytics endpoints

## Prioritized Backlog

### P0 - Critical
- None currently

### P1 - High Priority
- Phase 2 Policy-Based Auto-Fix (auto-remediate during CI/CD builds)
- Wire Phase 4 "Exception Management Workflow" into UI
- Wire Phase 4 "Global Drift Detection" Dashboard
- Implement actual SLSA Level 3/4 provenance generation (not mocked)

### P2 - Medium Priority
- Refactor server.py (~1640 lines) into separate routers
- Phase 3 "Proactive Evergreen" (upstream monitoring, auto-PRs)
- Implement actual Docker builds (currently simulated)
- Implement actual Trivy scanning
- Webhook notifications (Slack/Teams) for remediation events

### P3 - Nice to Have
- IDE Extensions (VS Code plugin for Dockerfile linting)
- Cloud-Native Registry Shims
- Real-time CVE database integration (NVD, OSV.dev)

## Technical Notes

### Mocked Services
The following features return hardcoded/simulated data:
- Docker image building
- Trivy vulnerability scanning (returns realistic CVE IDs)
- Cosign image signing
- OPA policy evaluation
- SLSA attestation
- VEX generation
- Drift detection
- Delta scan verification

### Test Results (Latest)
- Backend: 100% (41/41 tests passed)
- Frontend: 100% (all UI tests passed)
- Test files: 
  - `/app/backend/tests/test_secureimage_forge.py`
  - `/app/backend/tests/test_vulnerability_remediation.py`

## Last Updated
2026-04-09 - Completed Phase 5 Automatic Vulnerability Remediation implementation and testing
