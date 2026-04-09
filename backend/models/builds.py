"""
Build-related Pydantic models
"""
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional
import uuid

from pydantic import BaseModel, ConfigDict, Field


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


class BuildConfigExtended(BuildConfigCreate):
    architecture: List[str] = Field(default_factory=lambda: ["amd64"])  # amd64, arm64


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


class ImageSignature(BaseModel):
    model_config = ConfigDict(extra="ignore")
    signature_id: str
    build_id: str
    image_tag: str
    digest: str
    signing_method: str
    signed_at: datetime
    verified: bool = False


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
