"""
SLSA attestation routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime, timezone
import logging

from database import db
from services.slsa_attestor import (
    generate_attestation_bundle,
    verify_slsa_provenance
)
from services.webhook_manager import WebhookEventType

logger = logging.getLogger(__name__)

router = APIRouter(tags=["slsa"])

# Webhook helper - will be set from main app
_send_webhook_and_persist = None


def set_webhook_helper(helper_fn):
    """Set the webhook helper function from main app"""
    global _send_webhook_and_persist
    _send_webhook_and_persist = helper_fn


@router.get("/builds/{build_id}/slsa")
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
    
    # Send webhook notification
    if _send_webhook_and_persist:
        await _send_webhook_and_persist(WebhookEventType.SLSA_ATTESTATION_GENERATED, {
            "build_id": build_id,
            "image_tag": build_data["image_tag"],
            "slsa_level": level,
            "detail_url": f"https://secureforge.enterprise/builds/{build_id}"
        })
    
    return attestation_bundle


@router.post("/builds/{build_id}/slsa/verify")
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


@router.get("/slsa/levels")
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
