# SecureImage Forge

**SecureImage Forge** is an automated pipeline tool designed to build, harden, and verify Docker images for enterprise runtimes. It ensures that every container image produced is "secure by default," meeting rigorous compliance standards.

> ⚠️ **Active rewrite in progress** — the project is being ported from the
> Python/FastAPI MVP to a multi-platform Rust desktop application using only
> Apache-2.0 / MIT-licensed components (BuildKit, Trivy, Syft, Cosign, OPA,
> Dioxus). The Rust workspace lives in [`forge/`](./forge); see
> [`forge/docs/ARCHITECTURE.md`](./forge/docs/ARCHITECTURE.md) and
> [`forge/docs/DEVELOPMENT.md`](./forge/docs/DEVELOPMENT.md). The legacy
> Python implementation under `backend/` and `frontend/` will be removed once
> Phases 1–3 reach parity.

## Features

### Phase 1 MVP (Current)
- ✅ Multi-runtime support (Java, .NET)
- ✅ Base image selection (Alpine, Debian, Distroless)
- ✅ Compliance hardening (HIPAA, SOC2/PCI-DSS, CIS Benchmarks)
- ✅ Vulnerability scanning simulation
- ✅ SBOM generation (CycloneDX format)
- ✅ Web Dashboard
- ✅ CLI Tool
- ✅ Real-time build monitoring
- ✅ Compliance reporting

## Architecture

The system follows a **Base → Harden → Verify** workflow:

1. **Ingestion**: Pulls verified upstream images
2. **Stripping**: Removes shells, package managers if configured
3. **Hardening**: Applies non-root user, security configurations
4. **Scanning**: Vulnerability analysis
5. **Attestation**: Image signing (simulated)

## Web Dashboard

Access the web dashboard at: `http://localhost:3000` (or your deployment URL)

Features:
- Real-time build monitoring
- Vulnerability scan results
- Compliance health scores
- SBOM viewer
- Build history

## CLI Usage

The CLI tool provides command-line access to SecureImage Forge.

### Installation

The CLI is located at `/app/backend/cli/forge_cli.py`. Make it executable:

```bash
chmod +x /app/backend/cli/forge_cli.py
```

### Commands

#### 1. Build a New Image

```bash
python /app/backend/cli/forge_cli.py build \
  --name my-secure-app \
  --runtime java \
  --base alpine \
  --compliance hipaa soc2 cis \
  --sbom \
  --sign
```

**Options:**
- `--name`: Build configuration name (required)
- `--runtime`: Runtime environment (`java`, `dotnet`) (required)
- `--base`: Base image type (`alpine`, `debian`, `distroless`) (required)
- `--compliance`: Compliance profiles (multiple: `hipaa`, `soc2`, `cis`)
- `--no-shell`: Remove shell binaries (default: true)
- `--no-pkg-mgr`: Remove package managers (default: true)
- `--sbom/--no-sbom`: Generate SBOM (default: true)
- `--sign/--no-sign`: Sign image (default: true)

**Example:**
```bash
# Build a Java app on Alpine with all compliance profiles
python /app/backend/cli/forge_cli.py build \
  --name java-app-v1 \
  --runtime java \
  --base alpine \
  --compliance cis hipaa

# Build a .NET app on Debian with minimal hardening
python /app/backend/cli/forge_cli.py build \
  --name dotnet-api \
  --runtime dotnet \
  --base debian \
  --compliance soc2
```

#### 2. View Vulnerability Scan Results

```bash
python /app/backend/cli/forge_cli.py scan <build-id>
```

**Example:**
```bash
python /app/backend/cli/forge_cli.py scan abc123-def456-ghi789
```

#### 3. List All Builds

```bash
python /app/backend/cli/forge_cli.py list
```

#### 4. View Build Logs

```bash
python /app/backend/cli/forge_cli.py logs <build-id>
```

**Example:**
```bash
python /app/backend/cli/forge_cli.py logs abc123-def456-ghi789
```

#### 5. View Statistics

```bash
python /app/backend/cli/forge_cli.py stats
```

### API Endpoints

The backend API runs on port 8001 with `/api` prefix:

- `POST /api/builds` - Create and start a new build
- `GET /api/builds` - List all builds
- `GET /api/builds/{id}` - Get build details
- `GET /api/builds/{id}/scan` - Get vulnerability scan results
- `GET /api/builds/{id}/compliance` - Get compliance report
- `GET /api/builds/{id}/sbom` - Get SBOM
- `GET /api/stats` - Get dashboard statistics

## Compliance Standards

### HIPAA
- Non-root user enforcement
- No SSH server
- Audit logging hooks

### SOC2/PCI-DSS
- Non-root user enforcement
- FIPS-compliant cryptography
- Controlled entry points

### CIS Benchmarks
- Non-root user enforcement
- Shell binary removal
- Package manager removal
- Read-only filesystem recommendations

## Target Security Controls

- **Vulnerability Management**: 0 Critical/High CVEs policy
- **Least Privilege**: Mandatory USER 1000:1000
- **Immutability**: Read-only root filesystem
- **Transparency**: SBOM generation in CycloneDX format

## Technology Stack

### Backend
- FastAPI
- MongoDB
- Docker SDK for Python
- Python Click (CLI)

### Frontend
- React 19
- Tailwind CSS
- Phosphor Icons
- Axios

### Design System
- **Theme**: Light, Swiss & High-Contrast
- **Typography**: Chivo (headings), IBM Plex Mono (body)
- **Colors**: Klein Blue (#002FA7), Signal Red (#FF3B30), Minimal grayscale
- **Style**: "Control Room" aesthetic with functional clarity

## Development

### Backend
```bash
cd /app/backend
pip install -r requirements.txt
python server.py
```

### Frontend
```bash
cd /app/frontend
yarn install
yarn start
```

### Environment Variables

**Backend** (`/app/backend/.env`):
```
MONGO_URL=mongodb://localhost:27017
DB_NAME=secureforge
CORS_ORIGINS=*
```

**Frontend** (`/app/frontend/.env`):
```
REACT_APP_BACKEND_URL=http://localhost:8001
```

## Roadmap

### Phase 2: Enterprise Expansion
- .NET and Go runtime support expansion
- CIS Benchmark automated remediation
- Centralized dashboard for "Health Scores"
- Integration with private registries (JFrog, Azure ACR)

### Phase 3: Advanced Security
- Multi-arch support (ARM64/AMD64)
- Automatic base image updates
- Custom policy engine (Rego/OPA)
- Real Trivy integration
- Real Cosign signing

## License

Proprietary - Enterprise Security Tool

## Support

For issues and support, please contact the security team.
