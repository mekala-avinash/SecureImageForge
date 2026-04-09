# SecureImage Forge - Product Requirements Document

## Original Problem Statement
SecureImage Forge is an automated pipeline tool designed to build, harden, and verify Docker images for enterprise runtimes (Java, Node.js, .NET, Go). The product has multiple phases:
- Phase 1 (MVP): Basic pipelines, Trivy scanning, CLI & Web Dashboard
- Phase 2 (Enterprise): Advanced analytics, health scores, registry integrations
- Phase 3 (Advanced Security): Multi-architecture builds, keyless signing, custom policies
- Phase 4 (Enterprise Ecosystem): OPA integration, SLSA Level 3/4 compliance, automated VEX, Evergreen pipeline auto-updates
- Phase 4.5 (Granular Controls): Simple vs. Advanced UI toggle for dynamic versioning, binary stripping whitelists, FIPS mode, and custom Docker labels

## Architecture
- **Frontend**: React with Tailwind CSS, React Router, Axios
- **Backend**: FastAPI with Motor (async MongoDB)
- **Database**: MongoDB
- **CLI**: Click-based CLI (`/app/forge`)

### Key Files
- `/app/backend/server.py` - Main FastAPI application (1372 lines)
- `/app/frontend/src/App.js` - React router and main components
- `/app/frontend/src/components/EnhancedNewBuild.js` - Advanced build config form
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

### Policies & Analytics
- `GET /api/policies` - List policies
- `POST /api/policies` - Create policy
- `GET /api/analytics/*` - Various analytics endpoints

## Prioritized Backlog

### P0 - Critical
- None currently

### P1 - High Priority
- Wire Phase 4 "Exception Management Workflow" into UI
- Wire Phase 4 "Global Drift Detection" Dashboard
- Implement actual SLSA Level 3/4 provenance generation (not mocked)
- Implement actual VEX document generation (not mocked)

### P2 - Medium Priority
- Refactor server.py (1372 lines) into separate routers
- Implement actual Docker builds (currently simulated)
- Implement actual Trivy scanning
- Implement actual Cosign signing

### P3 - Nice to Have
- IDE Extensions (VS Code plugin)
- Cloud-Native Registry Shims
- Slack/Teams ChatOps Webhooks

## Technical Notes

### Mocked Services
The following features return hardcoded/simulated data:
- Docker image building
- Trivy vulnerability scanning
- Cosign image signing
- OPA policy evaluation
- SLSA attestation
- VEX generation
- Drift detection

### Test Results
- Backend: 100% (26/26 tests passed)
- Frontend: 100% (all UI tests passed)
- Test file: `/app/backend/tests/test_secureimage_forge.py`

## Last Updated
2026-04-09 - Completed Phase 4.5 Granular Controls implementation and testing
