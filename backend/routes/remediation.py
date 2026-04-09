"""
Vulnerability remediation routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime, timezone
from typing import Dict, Any
import uuid
import logging

from database import db
from services.vulnerability_remediation import (
    analyze_vulnerabilities,
    generate_remediated_dockerfile,
    create_remediation_record,
    simulate_delta_scan,
    CVE_REMEDIATION_DATABASE,
    RemediationStatus
)

logger = logging.getLogger(__name__)

router = APIRouter(tags=["remediation"])

# Webhook helper - will be set from main app
_send_webhook_and_persist = None
_webhook_event_type_build_completed = None


def set_webhook_helper(helper_fn, event_type):
    """Set the webhook helper function from main app"""
    global _send_webhook_and_persist, _webhook_event_type_build_completed
    _send_webhook_and_persist = helper_fn
    _webhook_event_type_build_completed = event_type


@router.get("/builds/{build_id}/vulnerabilities/analysis")
async def analyze_build_vulnerabilities(build_id: str):
    """Get detailed vulnerability analysis with remediation status"""
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found")
    
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0}) if build else None
    
    runtime = config.get("runtime", "java") if config else "java"
    base_image = config.get("base_image", "alpine") if config else "alpine"
    
    analysis = analyze_vulnerabilities(
        scan.get("vulnerabilities", {}),
        runtime=runtime,
        base_image=base_image
    )
    
    return {
        "build_id": build_id,
        "analysis": analysis
    }


@router.post("/builds/{build_id}/remediate")
async def auto_remediate_build(build_id: str, options: Dict[str, Any] = None):
    """Auto-remediate all fixable vulnerabilities"""
    if options is None:
        options = {}
    
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="Scan results not found. Run a scan first.")
    
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    
    runtime = config.get("runtime", "java") if config else "java"
    base_image = config.get("base_image", "alpine") if config else "alpine"
    
    # Analyze vulnerabilities
    analysis = analyze_vulnerabilities(
        scan.get("vulnerabilities", {}),
        runtime=runtime,
        base_image=base_image
    )
    
    # Generate remediated Dockerfile
    remediated = generate_remediated_dockerfile(
        analysis,
        runtime=runtime,
        base_image=base_image,
        original_config=config
    )
    
    # Run delta scan
    delta = simulate_delta_scan(analysis, remediated["applied_fixes"])
    
    # Create remediation record
    remediation_id = str(uuid.uuid4())
    record = create_remediation_record(
        build_id=build_id,
        remediation_id=remediation_id,
        analysis=analysis,
        applied_fixes=remediated["applied_fixes"],
        delta_scan=delta,
        auto_mode=True
    )
    
    # Store remediation
    await db.remediations.update_one(
        {"build_id": build_id},
        {"$set": record},
        upsert=True
    )
    
    return {
        "remediation_id": remediation_id,
        "status": "completed",
        "dockerfile": remediated["dockerfile"],
        "applied_fixes": remediated["applied_fixes"],
        "fixes_count": remediated["fixes_applied_count"],
        "delta_scan": delta,
        "message": f"Auto-remediation completed with {remediated['fixes_applied_count']} fixes applied"
    }


@router.post("/builds/{build_id}/remediate/{cve_id}")
async def remediate_single_cve(build_id: str, cve_id: str):
    """Remediate a single CVE"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    # Check if CVE is in our database
    cve_upper = cve_id.upper()
    if cve_upper not in CVE_REMEDIATION_DATABASE:
        raise HTTPException(status_code=404, detail=f"CVE {cve_id} not found in remediation database")
    
    cve_info = CVE_REMEDIATION_DATABASE[cve_upper]
    
    # Generate fix
    fix = {
        "cve_id": cve_upper,
        "fix_type": cve_info.get("fix_type", "package_upgrade"),
        "fix_command": cve_info.get("fix_command", ""),
        "new_version": cve_info.get("fixed_version"),
        "estimated_time": cve_info.get("estimated_time_minutes", 5)
    }
    
    # Store single CVE remediation
    await db.cve_remediations.update_one(
        {"build_id": build_id, "cve_id": cve_upper},
        {"$set": {
            "build_id": build_id,
            "cve_id": cve_upper,
            "fix": fix,
            "status": "applied",
            "applied_at": datetime.now(timezone.utc).isoformat()
        }},
        upsert=True
    )
    
    return {
        "cve_id": cve_upper,
        "status": "remediated",
        "fix": fix,
        "message": f"CVE {cve_upper} has been remediated"
    }


@router.get("/builds/{build_id}/remediation-history")
async def get_remediation_history(build_id: str):
    """Get remediation history for a build"""
    remediations = await db.remediations.find(
        {"build_id": build_id},
        {"_id": 0}
    ).sort("created_at", -1).to_list(100)
    
    cve_fixes = await db.cve_remediations.find(
        {"build_id": build_id},
        {"_id": 0}
    ).to_list(100)
    
    return {
        "build_id": build_id,
        "full_remediations": remediations,
        "individual_cve_fixes": cve_fixes
    }


@router.get("/remediation/cve-database")
async def list_known_cves():
    """List all CVEs in the remediation database"""
    cves = []
    for cve_id, info in CVE_REMEDIATION_DATABASE.items():
        cves.append({
            "cve_id": cve_id,
            "severity": info.get("severity", "unknown"),
            "affected_component": info.get("affected_component", "unknown"),
            "description": info.get("description", ""),
            "fix_available": info.get("status") == RemediationStatus.AUTO_FIXABLE.value,
            "fix_type": info.get("fix_type"),
            "estimated_time_minutes": info.get("estimated_time_minutes", 5)
        })
    
    return {
        "total_cves": len(cves),
        "cves": sorted(cves, key=lambda x: x["severity"], reverse=True)
    }


@router.get("/remediation/stats")
async def get_remediation_statistics():
    """Get overall remediation statistics"""
    total_remediations = await db.remediations.count_documents({})
    total_cve_fixes = await db.cve_remediations.count_documents({})
    
    # Count by status in CVE database
    auto_fixable = sum(1 for v in CVE_REMEDIATION_DATABASE.values() 
                       if v.get("status") == RemediationStatus.AUTO_FIXABLE.value)
    patch_available = sum(1 for v in CVE_REMEDIATION_DATABASE.values() 
                          if v.get("status") == RemediationStatus.PATCH_AVAILABLE.value)
    manual_required = sum(1 for v in CVE_REMEDIATION_DATABASE.values() 
                          if v.get("status") == RemediationStatus.MANUAL_REQUIRED.value)
    
    return {
        "total_remediations": total_remediations,
        "total_cve_fixes": total_cve_fixes,
        "cve_database": {
            "total": len(CVE_REMEDIATION_DATABASE),
            "auto_fixable": auto_fixable,
            "patch_available": patch_available,
            "manual_required": manual_required
        }
    }


@router.get("/remediation/policies")
async def get_remediation_policies():
    """Get all remediation policies"""
    policies = await db.remediation_policies.find({}, {"_id": 0}).to_list(100)
    
    active_policy = None
    for p in policies:
        if p.get("is_active"):
            active_policy = p
            break
    
    return {
        "policies": policies,
        "active_policy": active_policy
    }


@router.post("/remediation/policies")
async def create_remediation_policy(policy_data: Dict[str, Any]):
    """Create or update a remediation policy"""
    required_fields = ["name", "mode"]
    for field in required_fields:
        if field not in policy_data:
            raise HTTPException(status_code=400, detail=f"Missing required field: {field}")
    
    valid_modes = ["strict", "graceful", "notify_only"]
    if policy_data["mode"] not in valid_modes:
        raise HTTPException(status_code=400, detail=f"Invalid mode. Must be one of: {valid_modes}")
    
    policy = {
        "id": policy_data.get("id", str(uuid.uuid4())),
        "name": policy_data["name"],
        "description": policy_data.get("description", ""),
        "mode": policy_data["mode"],
        "auto_fix_critical": policy_data.get("auto_fix_critical", True),
        "auto_fix_high": policy_data.get("auto_fix_high", True),
        "auto_fix_medium": policy_data.get("auto_fix_medium", False),
        "fail_on_unfixable_critical": policy_data.get("fail_on_unfixable_critical", True),
        "notify_on_remediation": policy_data.get("notify_on_remediation", True),
        "is_active": policy_data.get("is_active", False),
        "created_at": datetime.now(timezone.utc).isoformat(),
        "updated_at": datetime.now(timezone.utc).isoformat()
    }
    
    await db.remediation_policies.update_one(
        {"id": policy["id"]},
        {"$set": policy},
        upsert=True
    )
    
    return {"id": policy["id"], "message": "Policy created/updated successfully", "policy": policy}


@router.post("/remediation/policies/{policy_id}/activate")
async def activate_remediation_policy(policy_id: str):
    """Activate a remediation policy (deactivates others)"""
    policy = await db.remediation_policies.find_one({"id": policy_id})
    if not policy:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    # Deactivate all policies
    await db.remediation_policies.update_many({}, {"$set": {"is_active": False}})
    
    # Activate selected policy
    await db.remediation_policies.update_one(
        {"id": policy_id},
        {"$set": {"is_active": True}}
    )
    
    return {"message": f"Policy '{policy.get('name')}' activated"}
