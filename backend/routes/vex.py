"""
VEX document routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime, timezone
import logging

from database import db
from services.vex_generator import (
    generate_vex_document,
    get_vex_summary,
    VEXStatus
)
from services.webhook_manager import WebhookEventType

logger = logging.getLogger(__name__)

router = APIRouter(tags=["vex"])

# Webhook helper - will be set from main app
_send_webhook_and_persist = None


def set_webhook_helper(helper_fn):
    """Set the webhook helper function from main app"""
    global _send_webhook_and_persist
    _send_webhook_and_persist = helper_fn


@router.get("/builds/{build_id}/vex")
async def get_vex_document_endpoint(build_id: str, format: str = "openvex"):
    """Generate VEX document for a build"""
    build = await db.build_history.find_one({"id": build_id}, {"_id": 0})
    if not build:
        raise HTTPException(status_code=404, detail="Build not found")
    
    scan = await db.scan_results.find_one({"build_id": build_id}, {"_id": 0})
    if not scan:
        raise HTTPException(status_code=404, detail="No scan results found. Run a scan first.")
    
    config = await db.build_configs.find_one({"id": build.get("config_id")}, {"_id": 0})
    
    # Generate VEX document
    vex_doc = generate_vex_document(
        build_id=build_id,
        image_tag=build.get("image_tag", "unknown"),
        vulnerabilities=scan.get("vulnerabilities", {}),
        runtime=config.get("runtime", "java") if config else "java",
        format=format
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
    
    # Send webhook notification
    summary = vex_doc.get("summary", {})
    if _send_webhook_and_persist:
        await _send_webhook_and_persist(WebhookEventType.VEX_DOCUMENT_GENERATED, {
            "build_id": build_id,
            "image_tag": build.get("image_tag"),
            "false_positive_rate": summary.get("false_positive_rate", 0),
            "not_affected": summary.get("not_affected", 0),
            "affected": summary.get("affected", 0),
            "detail_url": f"https://secureforge.enterprise/builds/{build_id}"
        })
    
    return vex_doc


@router.get("/builds/{build_id}/vex/summary")
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
            raise HTTPException(status_code=404, detail="No scan results found")
        
        return get_vex_summary(scan.get("vulnerabilities", {}))
    
    return vex_stored.get("document", {}).get("summary", {})


@router.get("/vex/formats")
async def get_vex_formats():
    """Get supported VEX formats"""
    return {
        "formats": [
            {
                "id": "openvex",
                "name": "OpenVEX",
                "description": "Open standard for VEX documents",
                "version": "0.2.0"
            },
            {
                "id": "csaf",
                "name": "CSAF VEX",
                "description": "Common Security Advisory Framework VEX profile",
                "version": "2.0"
            }
        ],
        "default": "openvex",
        "status_codes": [s.value for s in VEXStatus]
    }
