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

# Create the main app without a prefix
app = FastAPI(title="SecureImage Forge API")

# Create a router with the /api prefix
api_router = APIRouter(prefix="/api")

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
    # For demo purposes, return simulated results
    import random
    
    vuln_types = ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']
    vulnerabilities = {}
    
    for vtype in vuln_types:
        count = random.randint(0, 3) if vtype in ['CRITICAL', 'HIGH'] else random.randint(0, 10)
        vulnerabilities[vtype] = []
        for i in range(count):
            vulnerabilities[vtype].append({
                "id": f"CVE-2024-{random.randint(10000, 99999)}",
                "package": f"pkg-{random.choice(['openssl', 'libcurl', 'zlib', 'glibc'])}",
                "severity": vtype,
                "description": f"Sample {vtype.lower()} vulnerability in base image"
            })
    
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

# Registry Management
@api_router.post("/registries", response_model=Registry)
async def create_registry(registry: RegistryCreate):
    """Add a new container registry"""
    registry_obj = Registry(**registry.model_dump())
    registry_dict = registry_obj.model_dump()
    registry_dict['created_at'] = registry_dict['created_at'].isoformat()
    
    await db.registries.insert_one(registry_dict)
    return registry_obj

@api_router.get("/registries", response_model=List[Registry])
async def get_registries():
    """Get all configured registries"""
    registries = await db.registries.find({}, {"_id": 0}).to_list(100)
    
    for registry in registries:
        if isinstance(registry.get('created_at'), str):
            registry['created_at'] = datetime.fromisoformat(registry['created_at'])
    
    return registries

@api_router.delete("/registries/{registry_id}")
async def delete_registry(registry_id: str):
    """Delete a registry"""
    result = await db.registries.delete_one({"id": registry_id})
    
    if result.deleted_count == 0:
        raise HTTPException(status_code=404, detail="Registry not found")
    
    return {"message": "Registry deleted successfully"}

@api_router.post("/registries/{registry_id}/test")
async def test_registry(registry_id: str):
    """Test registry connection"""
    registry = await db.registries.find_one({"id": registry_id}, {"_id": 0})
    
    if not registry:
        raise HTTPException(status_code=404, detail="Registry not found")
    
    # Simulate registry test
    return {
        "registry_id": registry_id,
        "status": "connected",
        "message": f"Successfully connected to {registry['type']} registry at {registry['url']}"
    }

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

# Analytics Endpoints
@api_router.get("/analytics/trends")
async def get_analytics_trends(days: int = 30):
    """Get build and health score trends"""
    start_date = datetime.now(timezone.utc) - timedelta(days=days)
    
    # Get builds from the period
    builds = await db.build_history.find({
        "started_at": {"$gte": start_date.isoformat()}
    }, {"_id": 0}).to_list(1000)
    
    # Convert datetime strings
    for build in builds:
        if isinstance(build.get('started_at'), str):
            build['started_at'] = datetime.fromisoformat(build['started_at'])
    
    # Group by day
    daily_data = {}
    for build in builds:
        day = build['started_at'].date().isoformat()
        if day not in daily_data:
            daily_data[day] = {
                "date": day,
                "total": 0,
                "completed": 0,
                "failed": 0,
                "avg_compliance": 0,
                "compliance_scores": []
            }
        
        daily_data[day]["total"] += 1
        if build.get('status') == 'completed':
            daily_data[day]["completed"] += 1
        elif build.get('status') == 'failed':
            daily_data[day]["failed"] += 1
        
        if build.get('compliance_score'):
            daily_data[day]["compliance_scores"].append(build['compliance_score'])
    
    # Calculate averages
    trend_data = []
    for day_data in daily_data.values():
        if day_data["compliance_scores"]:
            day_data["avg_compliance"] = int(sum(day_data["compliance_scores"]) / len(day_data["compliance_scores"]))
        day_data.pop("compliance_scores")
        trend_data.append(day_data)
    
    trend_data.sort(key=lambda x: x["date"])
    
    return {
        "period_days": days,
        "trend_data": trend_data
    }

@api_router.get("/analytics/vulnerabilities")
async def get_vulnerability_analytics():
    """Get vulnerability trends across all builds"""
    completed_builds = await db.build_history.find({
        "status": "completed",
        "vulnerability_count": {"$exists": True}
    }, {"_id": 0}).to_list(1000)
    
    total_vulns = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
    vuln_by_runtime = {}
    
    for build in completed_builds:
        vuln_count = build.get('vulnerability_count', {})
        for severity in total_vulns.keys():
            total_vulns[severity] += vuln_count.get(severity, 0)
        
        # Get config to find runtime
        config = await db.build_configs.find_one({"id": build['config_id']}, {"_id": 0})
        if config:
            runtime = config.get('runtime', 'unknown')
            if runtime not in vuln_by_runtime:
                vuln_by_runtime[runtime] = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
            
            for severity in total_vulns.keys():
                vuln_by_runtime[runtime][severity] += vuln_count.get(severity, 0)
    
    return {
        "total_vulnerabilities": total_vulns,
        "by_runtime": vuln_by_runtime,
        "total_builds_analyzed": len(completed_builds)
    }

@api_router.get("/analytics/health-scores")
async def get_health_score_analytics():
    """Get health score distribution and trends"""
    completed_builds = await db.build_history.find({
        "status": "completed"
    }, {"_id": 0}).to_list(1000)
    
    scores = []
    grades = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
    
    for build in completed_builds:
        # Convert datetime strings
        if isinstance(build.get('started_at'), str):
            build['started_at'] = datetime.fromisoformat(build['started_at'])
        if build.get('completed_at') and isinstance(build['completed_at'], str):
            build['completed_at'] = datetime.fromisoformat(build['completed_at'])
        
        score = calculate_health_score(build)
        grade = get_health_grade(score)
        scores.append(score)
        grades[grade] = grades.get(grade, 0) + 1
    
    avg_score = int(sum(scores) / len(scores)) if scores else 0
    
    return {
        "average_health_score": avg_score,
        "grade_distribution": grades,
        "total_builds": len(completed_builds)
    }

@api_router.get("/analytics/success-rate")
async def get_success_rate_analytics(days: int = 30):
    """Get build success rate over time"""
    start_date = datetime.now(timezone.utc) - timedelta(days=days)
    
    builds = await db.build_history.find({
        "started_at": {"$gte": start_date.isoformat()}
    }, {"_id": 0}).to_list(1000)
    
    total = len(builds)
    completed = sum(1 for b in builds if b.get('status') == 'completed')
    failed = sum(1 for b in builds if b.get('status') == 'failed')
    
    success_rate = (completed / total * 100) if total > 0 else 0
    
    return {
        "period_days": days,
        "total_builds": total,
        "completed": completed,
        "failed": failed,
        "success_rate": round(success_rate, 2)
    }

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

@app.on_event("shutdown")
async def shutdown_db_client():
    client.close()