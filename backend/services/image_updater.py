"""Base Image Update Checker Service"""
from typing import Dict, List, Any
from datetime import datetime, timezone, timedelta
import random

# Simulated base image versions
BASE_IMAGE_VERSIONS = {
    "alpine": {
        "current": "3.19",
        "available": ["3.19.1", "3.20.0"],
        "latest": "3.20.0",
        "eol_date": "2026-11-01"
    },
    "debian": {
        "current": "bookworm",
        "available": ["bookworm-20240110", "bookworm-20240201"],
        "latest": "bookworm-20240201",
        "eol_date": "2028-06-01"
    },
    "distroless": {
        "current": "latest",
        "available": ["latest"],
        "latest": "latest",
        "eol_date": "2030-01-01"
    }
}

RUNTIME_VERSIONS = {
    "java": {
        "current": "17",
        "available": ["17.0.10", "21.0.2"],
        "latest": "21.0.2",
        "lts": ["17", "21"]
    },
    "dotnet": {
        "current": "8.0",
        "available": ["8.0.1", "8.0.2"],
        "latest": "8.0.2",
        "lts": ["6.0", "8.0"]
    },
    "go": {
        "current": "1.21",
        "available": ["1.21.6", "1.22.0"],
        "latest": "1.22.0",
        "lts": ["1.21", "1.22"]
    },
    "nodejs": {
        "current": "20",
        "available": ["20.11.0", "21.6.1"],
        "latest": "21.6.1",
        "lts": ["18", "20"]
    }
}

def check_for_updates(base_image: str, runtime: str) -> Dict[str, Any]:
    """Check if there are updates available for base image and runtime"""
    base_updates = BASE_IMAGE_VERSIONS.get(base_image, {})
    runtime_updates = RUNTIME_VERSIONS.get(runtime, {})
    
    has_base_update = len(base_updates.get('available', [])) > 1
    has_runtime_update = len(runtime_updates.get('available', [])) > 1
    
    return {
        "base_image": base_image,
        "runtime": runtime,
        "has_updates": has_base_update or has_runtime_update,
        "base_image_updates": {
            "current": base_updates.get('current'),
            "latest": base_updates.get('latest'),
            "available_versions": base_updates.get('available', []),
            "update_available": has_base_update,
            "eol_date": base_updates.get('eol_date')
        },
        "runtime_updates": {
            "current": runtime_updates.get('current'),
            "latest": runtime_updates.get('latest'),
            "available_versions": runtime_updates.get('available', []),
            "update_available": has_runtime_update,
            "lts_versions": runtime_updates.get('lts', [])
        },
        "checked_at": datetime.now(timezone.utc).isoformat()
    }

def get_update_severity(update_info: Dict[str, Any]) -> str:
    """Determine severity of available updates"""
    if not update_info['has_updates']:
        return "none"
    
    # Check if EOL is approaching (within 6 months)
    if update_info['base_image_updates'].get('eol_date'):
        try:
            eol = datetime.fromisoformat(update_info['base_image_updates']['eol_date'])
            if (eol - datetime.now(timezone.utc)).days < 180:
                return "critical"
        except:
            pass
    
    # Check version difference
    base_current = update_info['base_image_updates'].get('current', '')
    base_latest = update_info['base_image_updates'].get('latest', '')
    
    if base_current != base_latest:
        return "high"
    
    return "medium"

def generate_update_recommendation(update_info: Dict[str, Any]) -> Dict[str, Any]:
    """Generate recommendation for updating"""
    if not update_info['has_updates']:
        return {
            "recommended": False,
            "action": "none",
            "message": "Image is up to date"
        }
    
    severity = get_update_severity(update_info)
    
    recommendations = {
        "critical": {
            "recommended": True,
            "action": "immediate",
            "message": "Base image approaching EOL. Immediate update required.",
            "priority": 1
        },
        "high": {
            "recommended": True,
            "action": "soon",
            "message": "New base image version available. Update recommended within 7 days.",
            "priority": 2
        },
        "medium": {
            "recommended": True,
            "action": "planned",
            "message": "Runtime updates available. Plan update in next release cycle.",
            "priority": 3
        }
    }
    
    return recommendations.get(severity, recommendations["medium"])

def simulate_cve_in_old_version(base_image: str, runtime: str) -> List[Dict[str, Any]]:
    """Simulate CVEs that would be fixed by updating"""
    cves = []
    
    # Simulate some CVEs in old versions
    if base_image == "alpine":
        cves.append({
            "id": "CVE-2024-1234",
            "package": "alpine-base",
            "severity": "HIGH",
            "fixed_in": "3.20.0",
            "description": "Buffer overflow in Alpine base package"
        })
    
    if runtime == "java":
        cves.append({
            "id": "CVE-2024-5678",
            "package": "openjdk",
            "severity": "MEDIUM",
            "fixed_in": "17.0.10",
            "description": "Security vulnerability in JVM"
        })
    
    return cves

def get_rebuild_impact(build_count: int) -> Dict[str, Any]:
    """Estimate impact of rebuilding images"""
    avg_build_time = 5  # minutes
    
    return {
        "affected_builds": build_count,
        "estimated_time_minutes": build_count * avg_build_time,
        "estimated_cost": build_count * 0.10,  # $0.10 per build
        "recommendation": "Schedule during maintenance window" if build_count > 10 else "Can rebuild immediately"
    }