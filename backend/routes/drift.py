"""
Drift detection routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime, timezone
from typing import Dict, Any
import uuid
import random
import logging

from database import db
from services.drift_detector import simulate_k8s_runtime_images
from services.webhook_manager import WebhookEventType

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/drift", tags=["drift"])

# Webhook helper - will be set from main app
_send_webhook_and_persist = None


def set_webhook_helper(helper_fn):
    """Set the webhook helper function from main app"""
    global _send_webhook_and_persist
    _send_webhook_and_persist = helper_fn


@router.get("/runtime-images")
async def get_runtime_images():
    """Get all registered runtime images from Kubernetes clusters"""
    # In production, this would connect to K8s API
    # For now, return simulated data plus any DB-stored images
    db_images = await db.runtime_images.find({}, {"_id": 0}).to_list(100)
    
    if not db_images:
        # Return simulated K8s runtime images
        return {
            "images": simulate_k8s_runtime_images(),
            "source": "simulated"
        }
    
    return {
        "images": db_images,
        "source": "database"
    }


@router.post("/register-image")
async def register_runtime_image(image_data: Dict[str, Any]):
    """Register a new runtime image for monitoring"""
    required_fields = ["namespace", "pod_name", "image_tag"]
    for field in required_fields:
        if field not in image_data:
            raise HTTPException(status_code=400, detail=f"Missing required field: {field}")
    
    image = {
        "id": str(uuid.uuid4()),
        "namespace": image_data["namespace"],
        "pod_name": image_data["pod_name"],
        "image_tag": image_data["image_tag"],
        "cluster": image_data.get("cluster", "default"),
        "registered_at": datetime.now(timezone.utc).isoformat(),
        "last_scanned": None,
        "drift_status": "unknown"
    }
    
    await db.runtime_images.insert_one(image)
    
    return {"id": image["id"], "message": "Image registered for monitoring"}


@router.get("/scan")
async def run_drift_scan():
    """Run drift detection scan on all registered images"""
    runtime_images = simulate_k8s_runtime_images()
    
    # Simulate drift detection
    results = []
    total_drifted = 0
    critical_drifts = 0
    
    for image in runtime_images:
        # Simulate drift detection logic
        has_drift = random.random() > 0.7  # 30% chance of drift
        drift_details = []
        risk_level = "low"
        
        if has_drift:
            total_drifted += 1
            drift_types = ["digest_mismatch", "unauthorized_shell", "root_user", "missing_labels", "config_change"]
            num_drifts = random.randint(1, 3)
            drift_details = random.sample(drift_types, num_drifts)
            
            if "unauthorized_shell" in drift_details or "root_user" in drift_details:
                risk_level = "critical"
                critical_drifts += 1
            elif "digest_mismatch" in drift_details:
                risk_level = "high"
            else:
                risk_level = "medium"
        
        results.append({
            "image_id": image["id"],
            "namespace": image["namespace"],
            "pod_name": image["pod_name"],
            "image_tag": image["image_tag"],
            "has_drift": has_drift,
            "drift_details": drift_details,
            "risk_level": risk_level,
            "scanned_at": datetime.now(timezone.utc).isoformat()
        })
    
    # Store scan record
    scan_record = {
        "id": str(uuid.uuid4()),
        "scanned_at": datetime.now(timezone.utc).isoformat(),
        "total_images": len(runtime_images),
        "drifted_count": total_drifted,
        "critical_count": critical_drifts,
        "results": results
    }
    
    await db.drift_scans.insert_one(scan_record)
    
    # Send webhook if critical drifts found
    if critical_drifts > 0 and _send_webhook_and_persist:
        await _send_webhook_and_persist(WebhookEventType.DRIFT_DETECTED, {
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
        "results": results
    }


@router.get("/history")
async def get_drift_scan_history(limit: int = 10):
    """Get drift scan history"""
    scans = await db.drift_scans.find(
        {},
        {"_id": 0, "results": 0}
    ).sort("scanned_at", -1).limit(limit).to_list(limit)
    
    return {"scans": scans}


@router.get("/stats")
async def get_drift_statistics():
    """Get drift detection statistics"""
    total_scans = await db.drift_scans.count_documents({})
    
    # Get latest scan for current status
    latest_scan = await db.drift_scans.find_one(
        {},
        {"_id": 0},
        sort=[("scanned_at", -1)]
    )
    
    return {
        "total_scans": total_scans,
        "latest_scan": latest_scan,
        "monitoring_active": True
    }
