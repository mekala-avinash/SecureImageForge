"""
SecureImage Forge API - Main Server Module

This is the main FastAPI application for SecureImage Forge.
The codebase is being progressively modularized into:
- /routes/ - API endpoint routers (analytics, policies, webhooks, etc.)
- /models/ - Pydantic data models
- /services/ - Business logic services
- database.py - MongoDB connection

For now, most routes remain in this file for stability.
See /routes/__init__.py for the modular architecture plan.
"""

from fastapi import FastAPI, APIRouter, HTTPException, BackgroundTasks
from dotenv import load_dotenv
from starlette.middleware.cors import CORSMiddleware
from motor.motor_asyncio import AsyncIOMotorClient
import os
import logging
from pathlib import Path
from pydantic import BaseModel, Field, ConfigDict
from typing import List, Optional, Dict, Any
import uuid
from datetime import datetime, timezone, timedelta
import docker
import json
import subprocess
import tempfile
import shutil
import sys

# Import Phase 2 services
sys.path.append(str(Path(__file__).parent))
from services.health_score import calculate_health_score, get_health_grade, get_health_status
from services.remediation_engine import generate_remediation_suggestions, get_cis_benchmark_score
# Import Phase 3 services
from services.policy_engine import evaluate_all_policies, POLICY_TEMPLATES, get_policy_recommendation
from services.image_updater import check_for_updates, generate_update_recommendation, simulate_cve_in_old_version
from services.signing_service import sign_image, verify_signature, generate_attestation
# Import Phase 4 services
from services.opa_engine import get_enterprise_policies, evaluate_rego_policy
from services.exception_manager import ExceptionManager, ExceptionRequest, ExceptionStatus, get_exception_templates
from services.drift_detector import DriftDetector, simulate_k8s_runtime_images
from services.slsa_attestor import (
    generate_slsa_provenance,
    verify_slsa_provenance,
    generate_attestation_bundle,
    SLSALevel
)
from services.vex_generator import (
    generate_vex_document,
    get_vex_summary,
    VEXStatus,
    VEXJustification
)
from services.evergreen_pipeline import EvergreenPipeline, get_evergreen_stats
from services.lifecycle_manager import LifecycleManager, DeprecationPolicy, ImageLifecycleStage, run_garbage_collection
from services.webhook_manager import WebhookManager, WebhookConfig, WebhookEventType, WebhookDestination, DeliveryStatus, set_db_interface
# Import Vulnerability Remediation service
from services.vulnerability_remediation import (
    analyze_vulnerabilities,
    generate_remediated_dockerfile,
    create_remediation_record,
    simulate_delta_scan,
    get_remediation_status,
    CVE_REMEDIATION_DATABASE,
    RemediationStatus
)

ROOT_DIR = Path(__file__).parent
load_dotenv(ROOT_DIR / '.env')

# MongoDB connection
mongo_url = os.environ['MONGO_URL']
client = AsyncIOMotorClient(mongo_url)
db = client[os.environ['DB_NAME']]

# Docker client
try:
    docker_client = docker.from_env()
except Exception as e:
    logging.warning(f"Docker client initialization failed: {e}")
    docker_client = None

# Phase 4 service instances
exception_manager = ExceptionManager()
drift_detector = DriftDetector()
evergreen_pipeline = EvergreenPipeline()
lifecycle_manager = LifecycleManager()
webhook_manager = WebhookManager()

# Create the main app without a prefix
app = FastAPI(title="SecureImage Forge API")

# Create a router with the /api prefix
api_router = APIRouter(prefix="/api")

# Import modular route files
from routes.analytics import router as analytics_router
from routes.config import router as config_router
from routes.policies import router as policies_router
from routes.registries import router as registries_router
from routes.webhooks import router as webhooks_router, set_webhook_manager

# Include modular routers (these replace inline routes below)
api_router.include_router(analytics_router)
api_router.include_router(config_router)
api_router.include_router(policies_router)
api_router.include_router(registries_router)
api_router.include_router(webhooks_router)

# Set the webhook manager for the webhooks router
set_webhook_manager(webhook_manager)

# Models
class BuildConfig(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str
    runtime: str  # java, dotnet, go, nodejs
    base_image: str  # alpine, debian, distroless
    compliance_profiles: List[str]  # hipaa, soc2, cis
    remove_shell: bool = True
    remove_package_manager: bool = True
    enable_sbom: bool = True
    enable_signing: bool = True
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class BuildConfigCreate(BaseModel):
    name: str
    runtime: str
    base_image: str
    compliance_profiles: List[str]
    remove_shell: bool = True
    remove_package_manager: bool = True
    enable_sbom: bool = True
    enable_signing: bool = True
    architecture: List[str] = Field(default_factory=lambda: ["amd64"])  # Phase 3: Multi-arch support
    # Phase 4.5: Granular Controls
    runtime_version: Optional[str] = None  # e.g., "17", "8.0", "1.22"
    runtime_distribution: Optional[str] = None  # e.g., "temurin", "corretto", "microsoft"
    base_image_tag: Optional[str] = None  # e.g., "3.19.1", "12-slim"
    binary_whitelist: List[str] = Field(default_factory=list)  # Binaries to keep (e.g., ["/usr/bin/curl"])
    env_sanitization_rules: List[str] = Field(default_factory=list)  # Env vars to strip/mask
    cis_level: int = 1  # 1 or 2
    fips_mode_enabled: bool = False
    custom_labels: Dict[str, str] = Field(default_factory=dict)  # Docker labels
    sbom_format: str = "cyclonedx"  # cyclonedx or spdx
    sbom_scan_depth: str = "os_and_runtime"  # os_only, os_and_runtime, full

class BuildHistory(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    config_id: str
    config_name: str
    status: str  # building, scanning, hardening, completed, failed
    image_tag: Optional[str] = None
    logs: List[str] = Field(default_factory=list)
    vulnerability_count: Optional[Dict[str, int]] = None
    compliance_score: Optional[int] = None
    sbom_path: Optional[str] = None
    is_signed: bool = False  # Phase 3
    signature_id: Optional[str] = None  # Phase 3
    architecture: List[str] = Field(default_factory=lambda: ["amd64"])  # Phase 3
    started_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    completed_at: Optional[datetime] = None

class ScanResult(BaseModel):
    model_config = ConfigDict(extra="ignore")
    build_id: str
    vulnerabilities: Dict[str, List[Dict[str, Any]]]
    total_count: Dict[str, int]
    scan_time: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class ComplianceReport(BaseModel):
    model_config = ConfigDict(extra="ignore")
    build_id: str
    profiles: List[str]
    checks: List[Dict[str, Any]]
    overall_score: int
    passed: int
    failed: int
    warnings: int

# Phase 2 Models
class Registry(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str
    type: str  # jfrog, acr, dockerhub
    url: str
    username: str
    password: str  # In production, encrypt this
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class RegistryCreate(BaseModel):
    name: str
    type: str
    url: str
    username: str
    password: str

class HealthScoreHistory(BaseModel):
    model_config = ConfigDict(extra="ignore")
    build_id: str
    score: int
    grade: str
    status: str
    timestamp: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class BuildAnalytics(BaseModel):
    total_builds: int
    completed_builds: int
    failed_builds: int
    success_rate: float
    avg_health_score: int
    avg_compliance_score: int
    total_vulnerabilities: Dict[str, int]
    trend_data: List[Dict[str, Any]]

# Phase 3 Models
class Policy(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str
    description: str
    type: str  # vulnerability, compliance, configuration, security, freshness
    enforcement: str  # block, warn, info
    rule: Dict[str, Any]
    enabled: bool = True
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

class PolicyCreate(BaseModel):
    name: str
    description: str
    type: str
    enforcement: str
    rule: Dict[str, Any]
    enabled: bool = True

class BuildConfigExtended(BuildConfigCreate):
    architecture: List[str] = Field(default_factory=lambda: ["amd64"])  # amd64, arm64
    
class ImageSignature(BaseModel):
    model_config = ConfigDict(extra="ignore")
    signature_id: str
    build_id: str
    image_tag: str
    digest: str
    signing_method: str
    signed_at: datetime
    verified: bool = False

# Dockerfile Templates
DOCKERFILE_TEMPLATES = {
    "java": {
        "alpine": """FROM eclipse-temurin:17-jre-alpine
RUN addgroup -g 1000 appuser && adduser -D -u 1000 -G appuser appuser
WORKDIR /app
USER 1000:1000
COPY --chown=1000:1000 app.jar /app/app.jar
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "/app/app.jar"]""",
        "debian": """FROM eclipse-temurin:17-jre-jammy
RUN groupadd -g 1000 appuser && useradd -r -u 1000 -g appuser appuser
WORKDIR /app
USER 1000:1000
COPY --chown=1000:1000 app.jar /app/app.jar
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "/app/app.jar"]""",
        "distroless": """FROM gcr.io/distroless/java17-debian12
COPY app.jar /app/app.jar
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "/app/app.jar"]"""
    },
    "dotnet": {
        "alpine": """FROM mcr.microsoft.com/dotnet/aspnet:8.0-alpine
RUN addgroup -g 1000 appuser && adduser -D -u 1000 -G appuser appuser
WORKDIR /app
USER 1000:1000
COPY --chown=1000:1000 . /app
EXPOSE 8080
ENTRYPOINT ["dotnet", "app.dll"]""",
        "debian": """FROM mcr.microsoft.com/dotnet/aspnet:8.0
RUN groupadd -g 1000 appuser && useradd -r -u 1000 -g appuser appuser
WORKDIR /app
USER 1000:1000
COPY --chown=1000:1000 . /app
EXPOSE 8080
ENTRYPOINT ["dotnet", "app.dll"]""",
        "distroless": """FROM mcr.microsoft.com/dotnet/runtime:8.0
WORKDIR /app
COPY . /app
EXPOSE 8080
ENTRYPOINT ["dotnet", "app.dll"]"""
    },
    "go": {
        "alpine": """FROM golang:1.21-alpine AS builder
WORKDIR /build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-w -s" -o app .

FROM alpine:latest
RUN addgroup -g 1000 appuser && adduser -D -u 1000 -G appuser appuser
WORKDIR /app
USER 1000:1000
COPY --from=builder --chown=1000:1000 /build/app /app/app
EXPOSE 8080
ENTRYPOINT ["/app/app"]""",
        "debian": """FROM golang:1.21 AS builder
WORKDIR /build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-w -s" -o app .

FROM debian:bookworm-slim
RUN groupadd -g 1000 appuser && useradd -r -u 1000 -g appuser appuser
WORKDIR /app
USER 1000:1000
COPY --from=builder --chown=1000:1000 /build/app /app/app
EXPOSE 8080
ENTRYPOINT ["/app/app"]""",
        "distroless": """FROM golang:1.21 AS builder
WORKDIR /build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-w -s" -o app .

FROM gcr.io/distroless/static-debian12
COPY --from=builder /build/app /app
EXPOSE 8080
ENTRYPOINT ["/app"]"""
    },
    "nodejs": {
        "alpine": """FROM node:20-alpine
RUN addgroup -g 1000 appuser && adduser -D -u 1000 -G appuser appuser
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force
USER 1000:1000
COPY --chown=1000:1000 . .
EXPOSE 8080
ENTRYPOINT ["node", "index.js"]""",
        "debian": """FROM node:20-slim
RUN groupadd -g 1000 appuser && useradd -r -u 1000 -g appuser appuser
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force
USER 1000:1000
COPY --chown=1000:1000 . .
EXPOSE 8080
ENTRYPOINT ["node", "index.js"]""",
        "distroless": """FROM node:20-slim AS builder
WORKDIR /build
COPY package*.json ./
RUN npm ci --only=production

FROM gcr.io/distroless/nodejs20-debian12
WORKDIR /app
COPY --from=builder /build/node_modules ./node_modules
COPY . .
EXPOSE 8080
CMD ["index.js"]"""
    }
}

# Compliance hardening rules
COMPLIANCE_RULES = {
    "hipaa": [
        {"check": "non_root_user", "description": "Application runs as non-root user", "severity": "critical"},
        {"check": "no_ssh", "description": "SSH server is not installed", "severity": "high"},
        {"check": "audit_logging", "description": "Audit logging hooks configured", "severity": "medium"}
    ],
    "soc2": [
        {"check": "non_root_user", "description": "Application runs as non-root user", "severity": "critical"},
        {"check": "fips_crypto", "description": "FIPS-compliant cryptography", "severity": "high"},
        {"check": "controlled_entrypoints", "description": "Strictly controlled entry points", "severity": "medium"}
    ],
    "cis": [
        {"check": "non_root_user", "description": "Application runs as non-root user", "severity": "critical"},
        {"check": "no_shell", "description": "Shell binaries removed", "severity": "high"},
        {"check": "no_package_manager", "description": "Package managers removed", "severity": "high"},
        {"check": "read_only_fs", "description": "Read-only root filesystem", "severity": "medium"}
    ]
}

# Helper functions
def generate_dockerfile(runtime: str, base_image: str, config: BuildConfigCreate) -> str:
    """Generate Dockerfile based on runtime and base image"""
    template = DOCKERFILE_TEMPLATES.get(runtime, {}).get(base_image, "")
    
    if not template:
        raise ValueError(f"Unsupported runtime/base combination: {runtime}/{base_image}")
    
    # Add hardening steps
    hardening_steps = []
    
    if config.remove_shell and base_image != "distroless":
        if "alpine" in base_image:
            hardening_steps.append("RUN rm -rf /bin/sh /bin/bash /usr/bin/sh /usr/bin/bash 2>/dev/null || true")
        else:
            hardening_steps.append("RUN rm -rf /bin/sh /bin/bash /usr/bin/sh /usr/bin/bash /bin/dash 2>/dev/null || true")
    
    if config.remove_package_manager and base_image != "distroless":
        if "alpine" in base_image:
            hardening_steps.append("RUN rm -rf /sbin/apk /usr/bin/apk /etc/apk 2>/dev/null || true")
        else:
            hardening_steps.append("RUN rm -rf /usr/bin/apt* /usr/bin/dpkg* /var/lib/apt /var/lib/dpkg 2>/dev/null || true")
    
    # Insert hardening steps before USER directive
    lines = template.split('\n')
    user_index = next((i for i, line in enumerate(lines) if line.startswith('USER')), len(lines))
    
    for step in reversed(hardening_steps):
        lines.insert(user_index, step)
    
    return '\n'.join(lines)

def simulate_vulnerability_scan(image_tag: str) -> Dict[str, Any]:
    """Simulate vulnerability scanning (Trivy would be used in production)"""
    # For demo purposes, return simulated results with some real CVEs from our database
    import random
    
    # Real CVEs from our remediation database
    KNOWN_CVES = {
        "CRITICAL": [
            {"id": "CVE-2021-44228", "package": "log4j-core", "description": "Apache Log4j2 Remote Code Execution (Log4Shell)"},
            {"id": "CVE-2022-22965", "package": "spring-beans", "description": "Spring Framework RCE via Data Binding (Spring4Shell)"},
            {"id": "CVE-2022-42889", "package": "commons-text", "description": "Apache Commons Text RCE via StringSubstitutor"}
        ],
        "HIGH": [
            {"id": "CVE-2023-38545", "package": "curl", "description": "curl SOCKS5 heap buffer overflow"},
            {"id": "CVE-2023-4911", "package": "glibc", "description": "glibc buffer overflow in ld.so"},
            {"id": "CVE-2023-32002", "package": "nodejs", "description": "Node.js experimental permission model bypass"},
            {"id": "CVE-2023-34035", "package": "spring-security", "description": "Spring Security authorization rule bypass"},
            {"id": "CVE-2023-39325", "package": "golang", "description": "Go net/http HTTP/2 rapid reset attack"}
        ],
        "MEDIUM": [
            {"id": "CVE-2023-5678", "package": "openssl", "description": "OpenSSL Excessive time spent checking DH keys"},
            {"id": "CVE-2023-45143", "package": "undici", "description": "undici CRLF injection in request headers"}
        ],
        "LOW": []
    }
    
    vuln_types = ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']
    vulnerabilities = {}
    
    for vtype in vuln_types:
        # Mix known CVEs with random ones
        known = KNOWN_CVES.get(vtype, [])
        selected_known = random.sample(known, min(len(known), random.randint(1, len(known)))) if known else []
        
        # Add some random unknown CVEs too
        random_count = random.randint(0, 2) if vtype in ['CRITICAL', 'HIGH'] else random.randint(1, 5)
        random_vulns = [
            {
                "id": f"CVE-2024-{random.randint(10000, 99999)}",
                "package": f"pkg-{random.choice(['openssl', 'libcurl', 'zlib', 'glibc'])}",
                "severity": vtype,
                "description": f"Sample {vtype.lower()} vulnerability in base image"
            }
            for _ in range(random_count)
        ]
        
        # Combine known and random
        all_vulns = selected_known + random_vulns
        for v in all_vulns:
            v["severity"] = vtype
        
        vulnerabilities[vtype] = all_vulns
    
    total_count = {k: len(v) for k, v in vulnerabilities.items()}
    
    return {
        "vulnerabilities": vulnerabilities,
        "total_count": total_count
    }

def generate_compliance_report(config: BuildConfigCreate, build_id: str) -> ComplianceReport:
    """Generate compliance report based on selected profiles"""
    all_checks = []
    passed = 0
    failed = 0
    warnings = 0
    
    for profile in config.compliance_profiles:
        rules = COMPLIANCE_RULES.get(profile, [])
        for rule in rules:
            check_result = {
                "profile": profile,
                "check": rule["check"],
                "description": rule["description"],
                "severity": rule["severity"],
                "status": "passed"  # Simulated - would be actual check in production
            }
            
            # Simulate some failures for demo
            if rule["check"] in ["fips_crypto", "audit_logging"]:
                check_result["status"] = "warning"
                warnings += 1
            else:
                passed += 1
            
            all_checks.append(check_result)
    
    total = passed + failed + warnings
    overall_score = int((passed / total * 100)) if total > 0 else 100
    
    return ComplianceReport(
        build_id=build_id,
        profiles=config.compliance_profiles,
        checks=all_checks,
        overall_score=overall_score,
        passed=passed,
        failed=failed,
        warnings=warnings
    )

def generate_sbom(image_tag: str, build_id: str) -> str:
    """Generate SBOM in CycloneDX format"""
    sbom = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.4",
        "version": 1,
        "metadata": {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "component": {
                "type": "container",
                "name": image_tag,
                "version": "1.0.0"
            }
        },
        "components": [
            {
                "type": "library",
                "name": "base-image",
                "version": "latest",
                "description": "Base container image"
            }
        ]
    }
    
    sbom_path = f"/tmp/sbom_{build_id}.json"
    with open(sbom_path, 'w') as f:
        json.dump(sbom, f, indent=2)
    
    return sbom_path

async def build_image_task(config_dict: dict, build_id: str):
    """Background task to build and harden image"""
    try:
        # Update status to building
        await db.build_history.update_one(
            {"id": build_id},
            {"$set": {"status": "building"}, "$push": {"logs": f"Starting build process at {datetime.now(timezone.utc).isoformat()}"}}
        )
        
        config = BuildConfigCreate(**config_dict)
        image_tag = f"secureforge/{config.name}:latest"
        
        # Generate Dockerfile
        dockerfile_content = generate_dockerfile(config.runtime, config.base_image, config)
        
        await db.build_history.update_one(
            {"id": build_id},
            {"$push": {"logs": "Dockerfile generated successfully"}}
        )
        
        # Simulate building (in production, would use docker_client.images.build)
        await db.build_history.update_one(
            {"id": build_id},
            {"$set": {"status": "hardening", "image_tag": image_tag}, "$push": {"logs": f"Building image: {image_tag}"}}
        )
        
        # Simulate hardening phase
        await db.build_history.update_one(
            {"id": build_id},
            {"$push": {"logs": "Applying hardening profiles..."}}
        )
        
        # Update status to scanning
        await db.build_history.update_one(
            {"id": build_id},
            {"$set": {"status": "scanning"}, "$push": {"logs": "Scanning for vulnerabilities..."}}
        )
        
        # Vulnerability scan
        scan_results = simulate_vulnerability_scan(image_tag)
        await db.scan_results.insert_one({
            "build_id": build_id,
            "vulnerabilities": scan_results["vulnerabilities"],
            "total_count": scan_results["total_count"],
            "scan_time": datetime.now(timezone.utc).isoformat()
        })
        
        # Compliance report
        compliance_report = generate_compliance_report(config, build_id)
        await db.compliance_reports.insert_one(compliance_report.model_dump())
        
        await db.build_history.update_one(
            {"id": build_id},
            {"$push": {"logs": f"Compliance score: {compliance_report.overall_score}%"}}
        )
        
        # Generate SBOM
        if config.enable_sbom:
            sbom_path = generate_sbom(image_tag, build_id)
            await db.build_history.update_one(
                {"id": build_id},
                {"$set": {"sbom_path": sbom_path}, "$push": {"logs": f"SBOM generated: {sbom_path}"}}
            )
        
        # Complete
        completed_build_data = {
            "status": "completed",
            "vulnerability_count": scan_results["total_count"],
            "compliance_score": compliance_report.overall_score,
            "completed_at": datetime.now(timezone.utc)
        }
        
        await db.build_history.update_one(
            {"id": build_id},
            {
                "$set": {
                    "status": "completed",
                    "vulnerability_count": scan_results["total_count"],
                    "compliance_score": compliance_report.overall_score,
                    "completed_at": completed_build_data["completed_at"].isoformat()
                },
                "$push": {"logs": "Build completed successfully"}
            }
        )
        
        # Calculate and store health score (Phase 2)
        final_build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
        if final_build:
            if isinstance(final_build.get('started_at'), str):
                final_build['started_at'] = datetime.fromisoformat(final_build['started_at'])
            if isinstance(final_build.get('completed_at'), str):
                final_build['completed_at'] = datetime.fromisoformat(final_build['completed_at'])
            
            health_score = calculate_health_score(final_build)
            health_grade = get_health_grade(health_score)
            health_status = get_health_status(health_score)
            
            await db.health_scores.insert_one({
                "build_id": build_id,
                "score": health_score,
                "grade": health_grade,
                "status": health_status,
                "timestamp": datetime.now(timezone.utc).isoformat()
            })
            
            await db.build_history.update_one(
                {"id": build_id},
                {"$push": {"logs": f"Health score: {health_score} ({health_grade})"}}
            )
            
            # Phase 3: Sign the image if enabled
            if config.enable_signing:
                try:
                    signature = sign_image(image_tag, build_id, final_build.get('sbom_path'))
                    await db.signatures.insert_one(signature)
                    
                    await db.build_history.update_one(
                        {"id": build_id},
                        {
                            "$set": {
                                "is_signed": True,
                                "signature_id": signature['signature_id']
                            },
                            "$push": {"logs": f"Image signed: {signature['signature_id'][:8]}..."}
                        }
                    )
                except Exception as e:
                    await db.build_history.update_one(
                        {"id": build_id},
                        {"$push": {"logs": f"Warning: Signing failed: {str(e)}"}}
                    )
        
    except Exception as e:
        await db.build_history.update_one(
            {"id": build_id},
            {
                "$set": {"status": "failed"},
                "$push": {"logs": f"Build failed: {str(e)}"}
            }
        )

# API Routes
@api_router.get("/")
async def root():
    return {"message": "SecureImage Forge API", "version": "1.0.0"}

@api_router.post("/builds", response_model=BuildHistory)
async def create_build(config: BuildConfigCreate, background_tasks: BackgroundTasks):
    """Create and start a new image build"""
    # Save config
    config_obj = BuildConfig(**config.model_dump())
    config_dict = config_obj.model_dump()
    config_dict['created_at'] = config_dict['created_at'].isoformat()
    await db.build_configs.insert_one(config_dict)
    
    # Create build history entry
    build_history = BuildHistory(
        config_id=config_obj.id,
        config_name=config.name,
        status="queued"
    )
    build_dict = build_history.model_dump()
    build_dict['started_at'] = build_dict['started_at'].isoformat()
    await db.build_history.insert_one(build_dict)
    
    # Start background build task
    background_tasks.add_task(build_image_task, config.model_dump(), build_history.id)
    
    return build_history

@api_router.get("/builds", response_model=List[BuildHistory])
async def get_builds():
    """Get all build history"""
    builds = await db.build_history.find({}, {"_id": 0}).sort("started_at", -1).to_list(100)
    
    for build in builds:
        if isinstance(build.get('started_at'), str):
            build['started_at'] = datetime.fromisoformat(build['started_at'])
        if build.get('completed_at') and isinstance(build['completed_at'], str):
            build['completed_at'] = datetime.fromisoformat(build['completed_at'])
    
    return builds

@api_router.get("/builds/{build_id}", response_model=BuildHistory)
async def get_build(build_id: str):
    """Get specific build details"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    if isinstance(build.get('started_at'), str):
        build['started_at'] = datetime.fromisoformat(build['started_at'])
    if build.get('completed_at') and isinstance(build['completed_at'], str):
        build['completed_at'] = datetime.fromisoformat(build['completed_at'])
    
    return build

@api_router.get("/builds/{build_id}/scan")
async def get_scan_results(build_id: str):
    """Get vulnerability scan results for a build"""
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    return scan

@api_router.get("/builds/{build_id}/compliance")
async def get_compliance_report(build_id: str):
    """Get compliance report for a build"""
    report = await db.compliance_reports.find_one({"build_id": build_id}, {"_id": 0})
    
    if not report:
        raise HTTPException(status_code=404, detail="Compliance report not found")
    
    return report

@api_router.get("/builds/{build_id}/sbom")
async def get_sbom(build_id: str):
    """Get SBOM for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build or not build.get('sbom_path'):
        raise HTTPException(status_code=404, detail="SBOM not found")
    
    try:
        with open(build['sbom_path'], 'r') as f:
            sbom = json.load(f)
        return sbom
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error reading SBOM: {str(e)}")

@api_router.get("/configs", response_model=List[BuildConfig])
async def get_configs():
    """Get all build configurations"""
    configs = await db.build_configs.find({}, {"_id": 0}).to_list(100)
    
    for config in configs:
        if isinstance(config.get('created_at'), str):
            config['created_at'] = datetime.fromisoformat(config['created_at'])
    
    return configs

@api_router.get("/stats")
async def get_stats():
    """Get dashboard statistics"""
    total_builds = await db.build_history.count_documents({})
    completed_builds = await db.build_history.count_documents({"status": "completed"})
    failed_builds = await db.build_history.count_documents({"status": "failed"})
    in_progress = await db.build_history.count_documents({"status": {"$in": ["queued", "building", "scanning", "hardening"]}})
    
    # Average compliance score
    pipeline = [
        {"$group": {"_id": None, "avg_score": {"$avg": "$compliance_score"}}}
    ]
    result = await db.build_history.aggregate(pipeline).to_list(1)
    avg_compliance = int(result[0]['avg_score']) if result and result[0].get('avg_score') else 0
    
    return {
        "total_builds": total_builds,
        "completed_builds": completed_builds,
        "failed_builds": failed_builds,
        "in_progress": in_progress,
        "avg_compliance_score": avg_compliance
    }

# ============ PHASE 2 ENDPOINTS ============

@api_router.get("/builds/{build_id}/health")
async def get_build_health(build_id: str):
    """Get health score for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Convert datetime strings if needed
    if isinstance(build.get('started_at'), str):
        build['started_at'] = datetime.fromisoformat(build['started_at'])
    if build.get('completed_at') and isinstance(build['completed_at'], str):
        build['completed_at'] = datetime.fromisoformat(build['completed_at'])
    
    score = calculate_health_score(build)
    grade = get_health_grade(score)
    status = get_health_status(score)
    
    # Store in health score history
    health_record = {
        "build_id": build_id,
        "score": score,
        "grade": grade,
        "status": status,
        "timestamp": datetime.now(timezone.utc).isoformat()
    }
    await db.health_scores.insert_one(health_record)
    
    return {
        "build_id": build_id,
        "score": score,
        "grade": grade,
        "status": status,
        "timestamp": health_record["timestamp"]
    }

@api_router.get("/builds/{build_id}/remediation")
async def get_remediation_suggestions(build_id: str):
    """Get remediation suggestions for compliance failures"""
    report = await db.compliance_reports.find_one({"build_id": build_id}, {"_id": 0})
    
    if not report:
        raise HTTPException(status_code=404, detail="Compliance report not found")
    
    suggestions = generate_remediation_suggestions(report['checks'])
    cis_score = get_cis_benchmark_score(report['checks'])
    
    return {
        "build_id": build_id,
        "remediation_suggestions": suggestions,
        "cis_benchmark": cis_score
    }

# Registry Management - NOW SERVED FROM routes/registries.py

@api_router.post("/builds/{build_id}/push/{registry_id}")
async def push_to_registry(build_id: str, registry_id: str):
    """Push build image to a registry"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    registry = await db.registries.find_one({"id": registry_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    if not registry:
        raise HTTPException(status_code=404, detail="Registry not found")
    
    if build['status'] != 'completed':
        raise HTTPException(status_code=400, detail="Build not completed")
    
    # Simulate push
    pushed_tag = f"{registry['url']}/{build['image_tag']}"
    
    return {
        "build_id": build_id,
        "registry_id": registry_id,
        "pushed_tag": pushed_tag,
        "status": "success",
        "message": f"Image pushed to {registry['name']}"
    }

# Analytics Endpoints - NOW SERVED FROM routes/analytics.py

# ============ PHASE 3 ENDPOINTS ============

# Policy Management - NOW SERVED FROM routes/policies.py

@api_router.post("/builds/{build_id}/evaluate-policies")
async def evaluate_build_policies(build_id: str):
    """Evaluate all active policies against a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    config = await db.build_configs.find_one({"id": build['config_id']}, {"_id": 0})
    
    # Get all enabled policies
    policies = await db.policies.find({"enabled": True}, {"_id": 0}).to_list(1000)
    
    if not policies:
        return {
            "build_id": build_id,
            "total_policies": 0,
            "message": "No active policies to evaluate"
        }
    
    # Convert datetime strings
    if isinstance(build.get('started_at'), str):
        build['started_at'] = datetime.fromisoformat(build['started_at'])
    if build.get('completed_at') and isinstance(build['completed_at'], str):
        build['completed_at'] = datetime.fromisoformat(build['completed_at'])
    
    # Evaluate policies
    evaluation = evaluate_all_policies(policies, build, config)
    evaluation['build_id'] = build_id
    
    # Store evaluation results
    await db.policy_evaluations.insert_one({
        "build_id": build_id,
        "evaluation": evaluation,
        "evaluated_at": datetime.now(timezone.utc).isoformat()
    })
    
    return evaluation

@api_router.get("/builds/{build_id}/policy-recommendations")
async def get_build_policy_recommendations(build_id: str):
    """Get policy recommendations for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Convert datetime strings
    if isinstance(build.get('started_at'), str):
        build['started_at'] = datetime.fromisoformat(build['started_at'])
    if build.get('completed_at') and isinstance(build['completed_at'], str):
        build['completed_at'] = datetime.fromisoformat(build['completed_at'])
    
    recommendations = get_policy_recommendation(build)
    recommended_policies = [POLICY_TEMPLATES[rec] for rec in recommendations if rec in POLICY_TEMPLATES]
    
    return {
        "build_id": build_id,
        "recommended_policies": recommended_policies,
        "recommendation_count": len(recommended_policies)
    }

# Base Image Updates
@api_router.get("/builds/{build_id}/check-updates")
async def check_build_updates(build_id: str):
    """Check for available updates for a build's base image and runtime"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    config = await db.build_configs.find_one({"id": build['config_id']}, {"_id": 0})
    if not config:
        raise HTTPException(status_code=404, detail="Build configuration not found")
    
    update_info = check_for_updates(config['base_image'], config['runtime'])
    recommendation = generate_update_recommendation(update_info)
    cves_fixed = simulate_cve_in_old_version(config['base_image'], config['runtime'])
    
    # Store update check
    await db.update_checks.insert_one({
        "build_id": build_id,
        "update_info": update_info,
        "recommendation": recommendation,
        "cves_fixed_by_update": cves_fixed,
        "checked_at": datetime.now(timezone.utc).isoformat()
    })
    
    return {
        "build_id": build_id,
        "update_info": update_info,
        "recommendation": recommendation,
        "cves_fixed_by_update": cves_fixed
    }

@api_router.get("/updates/summary")
async def get_updates_summary():
    """Get summary of available updates across all builds"""
    configs = await db.build_configs.find({}, {"_id": 0}).to_list(1000)
    
    updates_available = []
    for config in configs:
        update_info = check_for_updates(config['base_image'], config['runtime'])
        if update_info['has_updates']:
            recommendation = generate_update_recommendation(update_info)
            updates_available.append({
                "config_id": config['id'],
                "config_name": config['name'],
                "base_image": config['base_image'],
                "runtime": config['runtime'],
                "update_info": update_info,
                "priority": recommendation.get('priority', 3)
            })
    
    # Sort by priority
    updates_available.sort(key=lambda x: x['priority'])
    
    return {
        "total_configs": len(configs),
        "updates_available": len(updates_available),
        "high_priority": sum(1 for u in updates_available if u['priority'] <= 2),
        "updates": updates_available
    }

# Image Signing
@api_router.post("/builds/{build_id}/sign")
async def sign_build_image(build_id: str):
    """Sign a completed build's image"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    if build['status'] != 'completed':
        raise HTTPException(status_code=400, detail="Only completed builds can be signed")
    
    # Sign the image
    signature = sign_image(build['image_tag'], build_id, build.get('sbom_path'))
    
    # Store signature
    await db.signatures.insert_one(signature)
    
    # Update build with signature info
    await db.build_history.update_one(
        {"id": build_id},
        {
            "$set": {
                "is_signed": True,
                "signature_id": signature['signature_id']
            },
            "$push": {"logs": f"Image signed with signature ID: {signature['signature_id']}"}
        }
    )
    
    return signature

@api_router.get("/builds/{build_id}/signature")
async def get_build_signature(build_id: str):
    """Get signature information for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    if not build.get('is_signed'):
        raise HTTPException(status_code=404, detail="Build is not signed")
    
    signature = await db.signatures.find_one({"build_id": build_id}, {"_id": 0})
    
    if not signature:
        raise HTTPException(status_code=404, detail="Signature not found")
    
    # Verify signature
    verification = verify_signature(signature)
    
    return {
        "signature": signature,
        "verification": verification
    }

@api_router.get("/builds/{build_id}/attestation")
async def get_build_attestation(build_id: str):
    """Get SLSA provenance attestation for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    attestation = generate_attestation(build_id, build)
    
    return attestation

@api_router.get("/signatures/verify/{image_tag}")
async def verify_image_signature(image_tag: str):
    """Verify signature for an image tag"""
    signature = await db.signatures.find_one({"image_tag": image_tag}, {"_id": 0})
    
    if not signature:
        raise HTTPException(status_code=404, detail="No signature found for this image")
    
    verification = verify_signature(signature)
    
    return verification

# ============ PHASE 4.5 GRANULAR CONTROLS ============
# NOTE: These routes are now served from routes/config.py
# Keeping this comment for reference

# =============================================================================
# VULNERABILITY REMEDIATION ENDPOINTS (Phase 5 - Auto-Remediation)
# =============================================================================

@api_router.get("/builds/{build_id}/vulnerabilities/analysis")
async def get_vulnerability_analysis(build_id: str):
    """Get detailed vulnerability analysis with remediation status for each CVE"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Get scan results
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    # Get build config for base image info
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    base_image = config.get("base_image", "alpine") if config else "alpine"
    
    # Analyze vulnerabilities
    analysis = analyze_vulnerabilities(scan.get("vulnerabilities", {}), base_image)
    
    return {
        "build_id": build_id,
        "build_name": build.get("config_name"),
        "base_image": base_image,
        "analysis": analysis
    }


@api_router.post("/builds/{build_id}/remediate")
async def trigger_auto_remediation(build_id: str, cve_ids: List[str] = None):
    """Trigger automatic remediation for all or specific CVEs"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Get scan results
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    # Get build config
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    if not config:
        raise HTTPException(status_code=404, detail="Build config not found")
    
    base_image = config.get("base_image", "alpine")
    runtime = config.get("runtime", "java")
    
    # Analyze vulnerabilities
    analysis = analyze_vulnerabilities(scan.get("vulnerabilities", {}), base_image)
    
    # Filter to specific CVEs if provided
    if cve_ids:
        analysis["vulnerabilities_with_remediation"] = [
            v for v in analysis["vulnerabilities_with_remediation"]
            if v.get("id") in cve_ids
        ]
        analysis["generated_dockerfile_fixes"] = [
            f for f in analysis["generated_dockerfile_fixes"]
            if f.get("cve_id") in cve_ids
        ]
    
    # Generate remediated Dockerfile
    original_dockerfile = f"FROM {base_image}\n# Original application layers..."
    remediated = generate_remediated_dockerfile(
        original_dockerfile,
        analysis,
        base_image,
        runtime
    )
    
    # Create remediation record
    remediation_id = str(uuid.uuid4())
    remediation_record = {
        "id": remediation_id,
        "build_id": build_id,
        "build_name": build.get("config_name"),
        "status": "completed",
        "original_vulnerability_count": scan.get("total_count", {}),
        "remediated_dockerfile": remediated["dockerfile"],
        "applied_fixes": remediated["applied_fixes"],
        "fixes_count": remediated["fixes_applied_count"],
        "created_at": datetime.now(timezone.utc).isoformat(),
        "audit_trail": []
    }
    
    # Add audit entries
    for fix in remediated["applied_fixes"]:
        audit_entry = create_remediation_record(
            build_id=build_id,
            cve_id=fix.get("cve_id", "UNKNOWN"),
            action="auto_remediate",
            status="applied",
            details=fix
        )
        remediation_record["audit_trail"].append(audit_entry)
    
    # Store remediation record
    await db.remediation_records.insert_one(remediation_record)
    
    # Simulate delta scan
    delta_scan = simulate_delta_scan(
        scan.get("total_count", {}),
        remediated["applied_fixes"]
    )
    
    return {
        "remediation_id": remediation_id,
        "build_id": build_id,
        "status": "completed",
        "dockerfile": remediated["dockerfile"],
        "applied_fixes": remediated["applied_fixes"],
        "fixes_count": remediated["fixes_applied_count"],
        "delta_scan": delta_scan,
        "message": f"Successfully generated remediated Dockerfile with {remediated['fixes_applied_count']} fixes applied."
    }


@api_router.post("/builds/{build_id}/remediate/{cve_id}")
async def remediate_single_cve(build_id: str, cve_id: str):
    """Trigger remediation for a single CVE"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Get remediation status for this CVE
    remediation_info = get_remediation_status(cve_id)
    
    if remediation_info["status"] == RemediationStatus.NO_FIX:
        raise HTTPException(status_code=400, detail=f"No fix available for {cve_id}")
    
    if not remediation_info["auto_fixable"]:
        return {
            "cve_id": cve_id,
            "status": "manual_required",
            "message": "This CVE requires manual intervention",
            "remediation_info": remediation_info
        }
    
    # Get config for base image
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    base_image = config.get("base_image", "alpine") if config else "alpine"
    base_type = "alpine" if "alpine" in base_image.lower() else "debian"
    
    # Get the fix command
    fix_command = None
    if remediation_info["fix_commands"]:
        if base_type in remediation_info["fix_commands"]:
            fix_command = remediation_info["fix_commands"][base_type]
        elif "docker" in remediation_info["fix_commands"]:
            fix_command = remediation_info["fix_commands"]["docker"]
    
    # Create audit record
    audit_entry = create_remediation_record(
        build_id=build_id,
        cve_id=cve_id,
        action="single_cve_remediate",
        status="fix_generated",
        details={
            "fix_command": fix_command,
            "fixed_version": remediation_info["fixed_version"],
            "remediation_type": remediation_info["remediation_type"]
        }
    )
    
    # Store audit record
    await db.remediation_audit.insert_one(audit_entry)
    
    return {
        "cve_id": cve_id,
        "status": "fix_generated",
        "fix_command": fix_command,
        "fixed_version": remediation_info["fixed_version"],
        "remediation_type": remediation_info["remediation_type"],
        "breaking_changes": remediation_info["breaking_changes"],
        "message": f"Fix generated for {cve_id}. Apply the command to your Dockerfile.",
        "audit_id": audit_entry["id"]
    }


@api_router.get("/builds/{build_id}/remediation-history")
async def get_remediation_history(build_id: str):
    """Get remediation audit trail for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Get all remediation records
    records = await db.remediation_records.find(
        {"build_id": build_id},
        {"_id": 0}
    ).sort("created_at", -1).to_list(100)
    
    # Get audit entries
    audit_entries = await db.remediation_audit.find(
        {"build_id": build_id},
        {"_id": 0}
    ).sort("timestamp", -1).to_list(100)
    
    return {
        "build_id": build_id,
        "remediation_records": records,
        "audit_trail": audit_entries,
        "total_remediations": len(records),
        "total_audit_entries": len(audit_entries)
    }


@api_router.get("/remediation/cve-database")
async def get_cve_database():
    """Get the list of known CVEs with automated fixes"""
    cve_list = []
    for cve_id, info in CVE_REMEDIATION_DATABASE.items():
        cve_list.append({
            "cve_id": cve_id,
            "severity": info["severity"],
            "package": info["package"],
            "fixed_version": info["fixed_version"],
            "auto_fixable": info["auto_fixable"],
            "remediation_type": info["remediation_type"],
            "description": info["description"],
            "breaking_changes": info.get("breaking_changes", False)
        })
    
    return {
        "total_cves": len(cve_list),
        "auto_fixable_count": sum(1 for c in cve_list if c["auto_fixable"]),
        "cves": sorted(cve_list, key=lambda x: (
            {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}.get(x["severity"], 4),
            x["cve_id"]
        ))
    }


@api_router.get("/remediation/stats")
async def get_remediation_stats():
    """Get overall remediation statistics"""
    total_remediations = await db.remediation_records.count_documents({})
    total_fixes = 0
    
    async for record in db.remediation_records.find({}, {"fixes_count": 1}):
        total_fixes += record.get("fixes_count", 0)
    
    return {
        "total_remediations_performed": total_remediations,
        "total_fixes_applied": total_fixes,
        "cve_database_size": len(CVE_REMEDIATION_DATABASE),
        "auto_fixable_cves": sum(1 for c in CVE_REMEDIATION_DATABASE.values() if c["auto_fixable"]),
        "supported_remediation_types": [
            "os_package_upgrade",
            "base_image_upgrade",
            "dependency_upgrade",
            "configuration"
        ]
    }


# =============================================================================
# EXCEPTION MANAGEMENT ENDPOINTS (Phase 4 - Request for Deviation)
# =============================================================================

@api_router.get("/exceptions")
async def list_exceptions(status: Optional[str] = None):
    """List all exception requests"""
    query = {}
    if status:
        query["status"] = status
    
    exceptions = await db.exceptions.find(query, {"_id": 0}).sort("created_at", -1).to_list(100)
    
    # Count by status
    pending = await db.exceptions.count_documents({"status": "pending"})
    approved = await db.exceptions.count_documents({"status": "approved"})
    rejected = await db.exceptions.count_documents({"status": "rejected"})
    
    return {
        "exceptions": exceptions,
        "counts": {
            "pending": pending,
            "approved": approved,
            "rejected": rejected,
            "total": len(exceptions)
        }
    }


@api_router.get("/exceptions/templates")
async def get_exception_request_templates():
    """Get available exception request templates"""
    return {
        "templates": get_exception_templates()
    }


@api_router.post("/exceptions")
async def create_exception_request(request_data: Dict[str, Any]):
    """Create a new exception request"""
    required_fields = ["build_id", "policy_id", "requestor", "justification"]
    for field in required_fields:
        if field not in request_data:
            raise HTTPException(status_code=400, detail=f"Missing required field: {field}")
    
    exception = {
        "id": str(uuid.uuid4()),
        "build_id": request_data["build_id"],
        "policy_id": request_data["policy_id"],
        "requestor": request_data["requestor"],
        "justification": request_data["justification"],
        "template_type": request_data.get("template_type"),
        "duration_days": request_data.get("duration_days", 30),
        "status": "pending",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "expires_at": None,
        "approver": None,
        "approved_at": None,
        "rejection_reason": None
    }
    
    await db.exceptions.insert_one(exception)
    
    # Send webhook notification and persist to DB
    await send_webhook_and_persist(WebhookEventType.EXCEPTION_REQUESTED, {
        "exception_id": exception["id"],
        "build_id": exception["build_id"],
        "requestor": exception["requestor"],
        "policy_id": exception["policy_id"]
    })
    
    return {"id": exception["id"], "status": "pending", "message": "Exception request created successfully"}


@api_router.get("/exceptions/{exception_id}")
async def get_exception(exception_id: str):
    """Get exception request details"""
    exception = await db.exceptions.find_one({"id": exception_id}, {"_id": 0})
    if not exception:
        raise HTTPException(status_code=404, detail="Exception not found")
    return exception


@api_router.post("/exceptions/{exception_id}/approve")
async def approve_exception(exception_id: str, approval_data: Dict[str, Any]):
    """Approve an exception request"""
    exception = await db.exceptions.find_one({"id": exception_id})
    if not exception:
        raise HTTPException(status_code=404, detail="Exception not found")
    
    if exception["status"] != "pending":
        raise HTTPException(status_code=400, detail=f"Exception is already {exception['status']}")
    
    approver = approval_data.get("approver", "security_admin")
    notes = approval_data.get("notes", "")
    duration_days = exception.get("duration_days", 30)
    
    expires_at = datetime.now(timezone.utc) + timedelta(days=duration_days)
    
    await db.exceptions.update_one(
        {"id": exception_id},
        {"$set": {
            "status": "approved",
            "approver": approver,
            "approved_at": datetime.now(timezone.utc).isoformat(),
            "approval_notes": notes,
            "expires_at": expires_at.isoformat()
        }}
    )
    
    # Send webhook notification and persist to DB
    await send_webhook_and_persist(WebhookEventType.EXCEPTION_APPROVED, {
        "exception_id": exception_id,
        "build_id": exception["build_id"],
        "approver": approver
    })
    
    return {"status": "approved", "expires_at": expires_at.isoformat(), "message": "Exception approved"}


@api_router.post("/exceptions/{exception_id}/reject")
async def reject_exception(exception_id: str, rejection_data: Dict[str, Any]):
    """Reject an exception request"""
    exception = await db.exceptions.find_one({"id": exception_id})
    if not exception:
        raise HTTPException(status_code=404, detail="Exception not found")
    
    if exception["status"] != "pending":
        raise HTTPException(status_code=400, detail=f"Exception is already {exception['status']}")
    
    approver = rejection_data.get("approver", "security_admin")
    reason = rejection_data.get("reason", "No reason provided")
    
    await db.exceptions.update_one(
        {"id": exception_id},
        {"$set": {
            "status": "rejected",
            "approver": approver,
            "approved_at": datetime.now(timezone.utc).isoformat(),
            "rejection_reason": reason
        }}
    )
    
    return {"status": "rejected", "message": "Exception rejected", "reason": reason}


# =============================================================================
# DRIFT DETECTION ENDPOINTS (Phase 4 - Global Drift Dashboard)
# =============================================================================

@api_router.get("/drift/runtime-images")
async def get_runtime_images():
    """Get all registered runtime images from Kubernetes clusters"""
    # In production, this would connect to K8s API
    # For now, return simulated data plus any DB-stored images
    simulated = simulate_k8s_runtime_images()
    
    # Also fetch from DB
    db_images = await db.runtime_images.find({}, {"_id": 0}).to_list(100)
    
    all_images = simulated + db_images
    
    return {
        "images": all_images,
        "total_count": len(all_images),
        "clusters": ["production", "staging"],  # Simulated clusters
        "last_scan": datetime.now(timezone.utc).isoformat()
    }


@api_router.post("/drift/register-image")
async def register_runtime_image(image_data: Dict[str, Any]):
    """Register a runtime image for drift detection"""
    image_record = {
        "image_id": image_data.get("image_id", str(uuid.uuid4())),
        "namespace": image_data.get("namespace", "default"),
        "pod_name": image_data.get("pod_name"),
        "image_tag": image_data["image_tag"],
        "digest": image_data.get("digest", ""),
        "template_id": image_data.get("template_id"),
        "has_shell": image_data.get("has_shell", False),
        "running_as_root": image_data.get("running_as_root", False),
        "registered_at": datetime.now(timezone.utc).isoformat()
    }
    
    await db.runtime_images.insert_one(image_record)
    
    # Also register with in-memory drift detector
    drift_detector.register_runtime_image(image_record["image_id"], image_record)
    
    return {"image_id": image_record["image_id"], "message": "Image registered for drift detection"}


@api_router.get("/drift/scan")
async def scan_for_drift():
    """Scan all runtime images for configuration drift"""
    # Get simulated K8s images
    runtime_images = simulate_k8s_runtime_images()
    
    drift_results = []
    total_drifted = 0
    critical_drifts = 0
    
    for image in runtime_images:
        # Register and scan each image
        drift_detector.register_runtime_image(image["image_id"], image)
        
        if image.get("template_id"):
            # Register a simulated template
            drift_detector.register_template(image["template_id"], {
                "has_shell": False,
                "running_as_root": False,
                "digest": "sha256:expected_secure_digest"
            })
            
            # Detect drift
            result = drift_detector.detect_drift(image["image_id"], image["template_id"])
            result["image_tag"] = image.get("image_tag")
            result["namespace"] = image.get("namespace")
            result["pod_name"] = image.get("pod_name")
            
            if result["has_drift"]:
                total_drifted += 1
                if result["risk_level"] == "critical":
                    critical_drifts += 1
            
            drift_results.append(result)
    
    # Store scan results
    scan_record = {
        "id": str(uuid.uuid4()),
        "scanned_at": datetime.now(timezone.utc).isoformat(),
        "total_images": len(runtime_images),
        "drifted_count": total_drifted,
        "critical_drifts": critical_drifts,
        "results": drift_results
    }
    
    await db.drift_scans.insert_one(scan_record)
    
    # Send webhook if critical drifts found
    if critical_drifts > 0:
        await send_webhook_and_persist(WebhookEventType.DRIFT_DETECTED, {
            "scan_id": scan_record["id"],
            "critical_drifts": critical_drifts,
            "total_drifted": total_drifted
        })
    
    return {
        "scan_id": scan_record["id"],
        "summary": {
            "total_images": len(runtime_images),
            "compliant": len(runtime_images) - total_drifted,
            "drifted": total_drifted,
            "critical": critical_drifts
        },
        "results": drift_results,
        "scanned_at": scan_record["scanned_at"]
    }


@api_router.get("/drift/history")
async def get_drift_scan_history():
    """Get drift scan history"""
    scans = await db.drift_scans.find({}, {"_id": 0, "results": 0}).sort("scanned_at", -1).to_list(50)
    
    return {
        "scans": scans,
        "total_scans": len(scans)
    }


@api_router.get("/drift/stats")
async def get_drift_stats():
    """Get drift detection statistics"""
    # Get latest scan
    latest_scan = await db.drift_scans.find_one({}, {"_id": 0}, sort=[("scanned_at", -1)])
    
    # Calculate averages from history
    total_scans = await db.drift_scans.count_documents({})
    
    return {
        "latest_scan": latest_scan,
        "total_scans": total_scans,
        "monitored_clusters": 2,
        "monitored_namespaces": ["production", "staging", "development"]
    }


# =============================================================================
# REMEDIATION POLICY ENDPOINTS (Phase 5.2 - Policy-Based Auto-Fix)
# =============================================================================

@api_router.get("/remediation/policies")
async def get_remediation_policies():
    """Get all remediation policies"""
    policies = await db.remediation_policies.find({}, {"_id": 0}).to_list(100)
    
    if not policies:
        # Return default policies
        policies = [
            {
                "id": "default-strict",
                "name": "Strict Mode",
                "description": "Fail build if auto-remediation fails",
                "mode": "strict",
                "auto_remediate_critical": True,
                "auto_remediate_high": True,
                "auto_remediate_medium": False,
                "fail_on_unfixable_critical": True,
                "notify_on_remediation": True,
                "enabled": False
            },
            {
                "id": "default-graceful",
                "name": "Graceful Mode",
                "description": "Apply fixes but allow builds to pass if some CVEs remain",
                "mode": "graceful",
                "auto_remediate_critical": True,
                "auto_remediate_high": True,
                "auto_remediate_medium": True,
                "fail_on_unfixable_critical": False,
                "notify_on_remediation": True,
                "enabled": True
            },
            {
                "id": "default-notify-only",
                "name": "Notify Only",
                "description": "Detect vulnerabilities but don't auto-fix (notification only)",
                "mode": "notify_only",
                "auto_remediate_critical": False,
                "auto_remediate_high": False,
                "auto_remediate_medium": False,
                "fail_on_unfixable_critical": False,
                "notify_on_remediation": True,
                "enabled": False
            }
        ]
    
    return {
        "policies": policies,
        "active_policy": next((p for p in policies if p.get("enabled")), policies[1] if len(policies) > 1 else None)
    }


@api_router.post("/remediation/policies")
async def create_remediation_policy(policy_data: Dict[str, Any]):
    """Create or update a remediation policy"""
    policy = {
        "id": policy_data.get("id", str(uuid.uuid4())),
        "name": policy_data["name"],
        "description": policy_data.get("description", ""),
        "mode": policy_data.get("mode", "graceful"),
        "auto_remediate_critical": policy_data.get("auto_remediate_critical", True),
        "auto_remediate_high": policy_data.get("auto_remediate_high", True),
        "auto_remediate_medium": policy_data.get("auto_remediate_medium", False),
        "fail_on_unfixable_critical": policy_data.get("fail_on_unfixable_critical", False),
        "notify_on_remediation": policy_data.get("notify_on_remediation", True),
        "enabled": policy_data.get("enabled", False),
        "created_at": datetime.now(timezone.utc).isoformat()
    }
    
    # If enabling this policy, disable others
    if policy["enabled"]:
        await db.remediation_policies.update_many({}, {"$set": {"enabled": False}})
    
    await db.remediation_policies.update_one(
        {"id": policy["id"]},
        {"$set": policy},
        upsert=True
    )
    
    return {"id": policy["id"], "message": "Policy saved successfully"}


@api_router.post("/remediation/policies/{policy_id}/activate")
async def activate_remediation_policy(policy_id: str):
    """Activate a specific remediation policy"""
    policy = await db.remediation_policies.find_one({"id": policy_id})
    if not policy:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    # Disable all other policies
    await db.remediation_policies.update_many({}, {"$set": {"enabled": False}})
    
    # Enable this policy
    await db.remediation_policies.update_one({"id": policy_id}, {"$set": {"enabled": True}})
    
    return {"message": f"Policy '{policy.get('name')}' activated"}


@api_router.post("/builds/{build_id}/auto-remediate-with-policy")
async def auto_remediate_with_policy(build_id: str):
    """Auto-remediate vulnerabilities according to active policy"""
    # Get active policy
    policy = await db.remediation_policies.find_one({"enabled": True})
    if not policy:
        # Use default graceful policy
        policy = {
            "mode": "graceful",
            "auto_remediate_critical": True,
            "auto_remediate_high": True,
            "auto_remediate_medium": False,
            "fail_on_unfixable_critical": False,
            "notify_on_remediation": True
        }
    
    # Get build
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Get scan results
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    # Get build config
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    base_image = config.get("base_image", "alpine") if config else "alpine"
    runtime = config.get("runtime", "java") if config else "java"
    
    # Analyze and filter vulnerabilities by policy
    analysis = analyze_vulnerabilities(scan.get("vulnerabilities", {}), base_image)
    
    # Filter based on policy settings
    filtered_vulns = []
    for vuln in analysis["vulnerabilities_with_remediation"]:
        severity = vuln.get("severity", "LOW")
        if severity == "CRITICAL" and policy.get("auto_remediate_critical"):
            filtered_vulns.append(vuln)
        elif severity == "HIGH" and policy.get("auto_remediate_high"):
            filtered_vulns.append(vuln)
        elif severity == "MEDIUM" and policy.get("auto_remediate_medium"):
            filtered_vulns.append(vuln)
    
    # Filter to only auto-fixable
    fixable_vulns = [v for v in filtered_vulns if v.get("auto_fixable")]
    
    if not fixable_vulns:
        # Check if we should fail
        unfixable_critical = sum(1 for v in analysis["vulnerabilities_with_remediation"] 
                                 if v.get("severity") == "CRITICAL" and not v.get("auto_fixable"))
        
        if policy.get("fail_on_unfixable_critical") and unfixable_critical > 0:
            return {
                "status": "failed",
                "mode": policy.get("mode"),
                "message": f"Build blocked: {unfixable_critical} unfixable critical vulnerabilities",
                "unfixable_critical": unfixable_critical
            }
        
        return {
            "status": "skipped",
            "mode": policy.get("mode"),
            "message": "No auto-fixable vulnerabilities matching policy criteria",
            "policy_applied": policy.get("name", "default")
        }
    
    # Generate remediation
    analysis["vulnerabilities_with_remediation"] = fixable_vulns
    analysis["generated_dockerfile_fixes"] = [
        f for f in analysis.get("generated_dockerfile_fixes", [])
        if any(v.get("id") == f.get("cve_id") for v in fixable_vulns)
    ]
    
    original_dockerfile = f"FROM {base_image}\n# Original application layers..."
    remediated = generate_remediated_dockerfile(
        original_dockerfile,
        analysis,
        base_image,
        runtime
    )
    
    # Create remediation record
    remediation_id = str(uuid.uuid4())
    remediation_record = {
        "id": remediation_id,
        "build_id": build_id,
        "build_name": build.get("config_name"),
        "policy_applied": policy.get("name", "default"),
        "policy_mode": policy.get("mode"),
        "status": "completed",
        "remediated_dockerfile": remediated["dockerfile"],
        "applied_fixes": remediated["applied_fixes"],
        "fixes_count": remediated["fixes_applied_count"],
        "created_at": datetime.now(timezone.utc).isoformat()
    }
    
    await db.remediation_records.insert_one(remediation_record)
    
    # Send notification if enabled
    if policy.get("notify_on_remediation"):
        await send_webhook_and_persist(WebhookEventType.BUILD_COMPLETED, {
            "build_id": build_id,
            "image_tag": build.get("image_tag"),
            "remediation_applied": True,
            "fixes_count": remediated["fixes_applied_count"]
        })
    
    return {
        "remediation_id": remediation_id,
        "status": "completed",
        "mode": policy.get("mode"),
        "policy_applied": policy.get("name", "default"),
        "dockerfile": remediated["dockerfile"],
        "applied_fixes": remediated["applied_fixes"],
        "fixes_count": remediated["fixes_applied_count"],
        "message": f"Auto-remediation completed with {remediated['fixes_applied_count']} fixes"
    }


# =============================================================================
# SLSA ATTESTATION ENDPOINTS (Phase 4 - Supply Chain Security)
# =============================================================================

@api_router.get("/builds/{build_id}/slsa")
async def get_slsa_attestation(build_id: str, level: int = 3):
    """Generate SLSA attestation for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    if not config:
        config = {"runtime": "java", "base_image": "alpine"}
    
    # Generate SLSA provenance
    build_data = {
        "id": build_id,
        "image_tag": build.get("image_tag", f"secureforge/{config.get('runtime')}:latest"),
        "started_at": build.get("started_at"),
        "completed_at": build.get("completed_at")
    }
    
    attestation_bundle = generate_attestation_bundle(build_data, config, slsa_level=level)
    
    # Store attestation
    await db.slsa_attestations.update_one(
        {"build_id": build_id},
        {"$set": {
            "build_id": build_id,
            "slsa_level": level,
            "attestation": attestation_bundle,
            "created_at": datetime.now(timezone.utc).isoformat()
        }},
        upsert=True
    )
    
    # Send webhook notification and persist to DB
    await send_webhook_and_persist(WebhookEventType.SLSA_ATTESTATION_GENERATED, {
        "build_id": build_id,
        "image_tag": build_data["image_tag"],
        "slsa_level": level,
        "detail_url": f"https://secureforge.enterprise/builds/{build_id}"
    })
    
    return attestation_bundle


@api_router.post("/builds/{build_id}/slsa/verify")
async def verify_build_slsa(build_id: str):
    """Verify SLSA attestation for a build"""
    attestation = await db.slsa_attestations.find_one({"build_id": build_id}, {"_id": 0})
    if not attestation:
        raise HTTPException(status_code=404, detail="SLSA attestation not found. Generate one first.")
    
    provenance = attestation.get("attestation", {}).get("provenance", {})
    verification = verify_slsa_provenance(provenance)
    
    return {
        "build_id": build_id,
        "verification": verification,
        "slsa_level": attestation.get("slsa_level"),
        "verified_at": datetime.now(timezone.utc).isoformat()
    }


@api_router.get("/slsa/levels")
async def get_slsa_level_info():
    """Get information about SLSA levels and requirements"""
    return {
        "levels": {
            1: {
                "name": "Build L1",
                "description": "Provenance exists and build is scripted",
                "requirements": ["Scripted build", "Provenance generated"],
                "trust_level": "Low"
            },
            2: {
                "name": "Build L2",
                "description": "Hosted build platform with signed provenance",
                "requirements": ["Version controlled source", "Hosted build service", "Authenticated provenance"],
                "trust_level": "Medium"
            },
            3: {
                "name": "Build L3",
                "description": "Hardened build platform with non-falsifiable provenance",
                "requirements": ["Hardened platform", "Isolated execution", "Non-falsifiable provenance"],
                "trust_level": "High"
            },
            4: {
                "name": "Build L4",
                "description": "Hermetic, reproducible builds with two-party review",
                "requirements": ["Two-party review", "Hermetic builds", "Reproducible builds"],
                "trust_level": "Very High"
            }
        },
        "default_level": 3,
        "recommended_for_production": 3
    }


# =============================================================================
# VEX DOCUMENT ENDPOINTS (Phase 4 - Vulnerability Exploitability Exchange)
# =============================================================================

@api_router.get("/builds/{build_id}/vex")
async def get_vex_document(build_id: str, format: str = "openvex"):
    """Generate VEX document for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    
    # Generate VEX document
    vex_doc = generate_vex_document(
        build_id=build_id,
        vulnerabilities=scan.get("vulnerabilities", {}),
        image_tag=build.get("image_tag", f"secureforge/{config.get('runtime', 'app')}:latest"),
        runtime=config.get("runtime", "java") if config else "java",
        config=config,
        output_format=format
    )
    
    # Store VEX document
    await db.vex_documents.update_one(
        {"build_id": build_id},
        {"$set": {
            "build_id": build_id,
            "format": format,
            "document": vex_doc,
            "created_at": datetime.now(timezone.utc).isoformat()
        }},
        upsert=True
    )
    
    # Send webhook notification and persist to DB
    summary = vex_doc.get("summary", {})
    await send_webhook_and_persist(WebhookEventType.VEX_DOCUMENT_GENERATED, {
        "build_id": build_id,
        "image_tag": build.get("image_tag"),
        "false_positive_rate": summary.get("false_positive_rate", 0),
        "not_affected": summary.get("not_affected", 0),
        "affected": summary.get("affected", 0),
        "detail_url": f"https://secureforge.enterprise/builds/{build_id}"
    })
    
    return vex_doc


@api_router.get("/builds/{build_id}/vex/summary")
async def get_vex_summary_endpoint(build_id: str):
    """Get executive summary of VEX analysis"""
    vex_stored = await db.vex_documents.find_one({"build_id": build_id}, {"_id": 0})
    
    if not vex_stored:
        # Generate VEX first
        build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
        if not build:
            raise HTTPException(status_code=404, detail="Build not found")
        
        scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
        if not scan:
            raise HTTPException(status_code=404, detail="Scan results not found")
        
        config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
        
        vex_doc = generate_vex_document(
            build_id=build_id,
            vulnerabilities=scan.get("vulnerabilities", {}),
            image_tag=build.get("image_tag", "unknown"),
            runtime=config.get("runtime", "java") if config else "java",
            config=config
        )
    else:
        vex_doc = vex_stored.get("document", {})
    
    summary = get_vex_summary(vex_doc)
    
    return {
        "build_id": build_id,
        "summary": summary
    }


@api_router.get("/vex/formats")
async def get_vex_formats():
    """Get supported VEX formats"""
    return {
        "formats": [
            {
                "id": "openvex",
                "name": "OpenVEX",
                "description": "OpenVEX specification for vulnerability exploitability",
                "spec_url": "https://openvex.dev/",
                "default": True
            },
            {
                "id": "csaf",
                "name": "CSAF VEX",
                "description": "OASIS CSAF 2.0 VEX profile",
                "spec_url": "https://docs.oasis-open.org/csaf/csaf/v2.0/csaf-v2.0.html",
                "default": False
            }
        ],
        "statuses": [
            {"id": "not_affected", "description": "Vulnerability does not affect this product"},
            {"id": "affected", "description": "Vulnerability affects this product"},
            {"id": "fixed", "description": "Vulnerability has been fixed"},
            {"id": "under_investigation", "description": "Status is being investigated"}
        ],
        "justifications": [
            {"id": "component_not_present", "description": "Vulnerable component is not in the product"},
            {"id": "vulnerable_code_not_present", "description": "Vulnerable code is not present"},
            {"id": "vulnerable_code_not_in_execute_path", "description": "Vulnerable code is not executed"},
            {"id": "vulnerable_code_cannot_be_controlled_by_adversary", "description": "Code cannot be exploited"},
            {"id": "inline_mitigations_already_exist", "description": "Existing mitigations prevent exploitation"}
        ]
    }


# =============================================================================
# WEBHOOK MANAGEMENT ENDPOINTS - NOW SERVED FROM routes/webhooks.py
# =============================================================================


# Include the router in the main app
app.include_router(api_router)

app.add_middleware(
    CORSMiddleware,
    allow_credentials=True,
    allow_origins=os.environ.get('CORS_ORIGINS', '*').split(','),
    allow_methods=["*"],
    allow_headers=["*"],
)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


# Database interface functions for webhook delivery persistence
async def save_webhook_delivery(delivery: dict):
    """Save webhook delivery record to database"""
    # Make a copy to avoid MongoDB mutating the original dict with _id
    delivery_copy = {**delivery}
    await db.webhook_deliveries.insert_one(delivery_copy)


async def get_webhook_deliveries(limit: int = 50):
    """Get webhook delivery history from database"""
    deliveries = await db.webhook_deliveries.find(
        {},
        {"_id": 0}
    ).sort("created_at", -1).limit(limit).to_list(limit)
    return deliveries


async def get_webhook_delivery_stats():
    """Get webhook delivery statistics from database"""
    # Count total deliveries
    total = await db.webhook_deliveries.count_documents({})
    success = await db.webhook_deliveries.count_documents({"status": "success"})
    failed = await db.webhook_deliveries.count_documents({"status": "failed"})
    
    # Get webhook counts from manager (still in-memory)
    registered = len(webhook_manager.webhooks)
    enabled = sum(1 for w in webhook_manager.webhooks.values() if w.enabled)
    
    return {
        "total_deliveries": total,
        "successful": success,
        "failed": failed,
        "success_rate": round((success / total * 100) if total > 0 else 0, 2),
        "registered_webhooks": registered,
        "enabled_webhooks": enabled
    }


async def send_webhook_and_persist(event_type: WebhookEventType, payload: dict):
    """Send webhook event and persist deliveries to database"""
    # Send the webhook (synchronous HTTP call)
    deliveries = webhook_manager.send_event(event_type, payload)
    
    # Persist each delivery to database
    for delivery in deliveries:
        try:
            await save_webhook_delivery(delivery)
        except Exception as e:
            logger.error(f"Failed to persist webhook delivery: {e}")


@app.on_event("startup")
async def load_webhooks_from_db():
    """Load registered webhooks from database on startup and configure DB interface"""
    # Configure the webhook manager's database interface
    set_db_interface(save_webhook_delivery, get_webhook_deliveries)
    logger.info("Webhook delivery database interface configured")
    
    try:
        webhooks = await db.webhooks.find({}, {"_id": 0}).to_list(100)
        for wh in webhooks:
            try:
                config = WebhookConfig(
                    name=wh.get("name", "Unknown"),
                    destination=WebhookDestination(wh.get("destination", "generic_webhook")),
                    url=wh.get("url", ""),
                    events=[WebhookEventType(e) for e in wh.get("events", [])],
                    channel=wh.get("channel"),
                    secret=wh.get("secret"),
                    enabled=wh.get("enabled", True)
                )
                config.id = wh.get("id", config.id)
                webhook_manager.webhooks[config.id] = config
                logger.info(f"Loaded webhook from DB: {config.name}")
            except Exception as e:
                logger.error(f"Failed to load webhook {wh.get('id')}: {e}")
        logger.info(f"Loaded {len(webhooks)} webhooks from database")
    except Exception as e:
        logger.error(f"Failed to load webhooks: {e}")


@app.on_event("shutdown")
async def shutdown_db_client():
    client.close()