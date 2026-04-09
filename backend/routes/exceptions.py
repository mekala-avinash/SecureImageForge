"""
Exception management routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime, timezone, timedelta
from typing import Dict, Any, Optional
import uuid
import logging

from database import db
from services.exception_manager import get_exception_templates
from services.webhook_manager import WebhookEventType

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/exceptions", tags=["exceptions"])

# Webhook helper - will be set from main app
_send_webhook_and_persist = None


def set_webhook_helper(helper_fn):
    """Set the webhook helper function from main app"""
    global _send_webhook_and_persist
    _send_webhook_and_persist = helper_fn


@router.get("")
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


@router.get("/templates")
async def get_exception_request_templates():
    """Get available exception request templates"""
    return {
        "templates": get_exception_templates()
    }


@router.post("")
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
    
    # Send webhook notification
    if _send_webhook_and_persist:
        await _send_webhook_and_persist(WebhookEventType.EXCEPTION_REQUESTED, {
            "exception_id": exception["id"],
            "build_id": exception["build_id"],
            "requestor": exception["requestor"],
            "policy_id": exception["policy_id"]
        })
    
    return {"id": exception["id"], "status": "pending", "message": "Exception request created successfully"}


@router.get("/{exception_id}")
async def get_exception(exception_id: str):
    """Get exception request details"""
    exception = await db.exceptions.find_one({"id": exception_id}, {"_id": 0})
    if not exception:
        raise HTTPException(status_code=404, detail="Exception not found")
    return exception


@router.post("/{exception_id}/approve")
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
    
    # Send webhook notification
    if _send_webhook_and_persist:
        await _send_webhook_and_persist(WebhookEventType.EXCEPTION_APPROVED, {
            "exception_id": exception_id,
            "build_id": exception["build_id"],
            "approver": approver
        })
    
    return {"status": "approved", "expires_at": expires_at.isoformat(), "message": "Exception approved"}


@router.post("/{exception_id}/reject")
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
