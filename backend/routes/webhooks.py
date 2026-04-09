"""
Webhook routes for ChatOps integration
"""
from datetime import datetime, timezone
from typing import Any, Dict

from fastapi import APIRouter, HTTPException
import logging

from database import db
from services.webhook_manager import (
    WebhookManager, 
    WebhookConfig, 
    WebhookEventType, 
    WebhookDestination
)

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/webhooks", tags=["webhooks"])

# Webhook manager instance - will be set from main app
webhook_manager: WebhookManager = None


def set_webhook_manager(manager: WebhookManager):
    """Set the webhook manager instance from main app"""
    global webhook_manager
    webhook_manager = manager


async def get_webhook_deliveries(limit: int = 50):
    """Get webhook delivery history from database"""
    deliveries = await db.webhook_deliveries.find(
        {},
        {"_id": 0}
    ).sort("created_at", -1).limit(limit).to_list(limit)
    return deliveries


async def get_webhook_delivery_stats():
    """Get webhook delivery statistics from database"""
    total = await db.webhook_deliveries.count_documents({})
    success = await db.webhook_deliveries.count_documents({"status": "success"})
    failed = await db.webhook_deliveries.count_documents({"status": "failed"})
    
    registered = len(webhook_manager.webhooks) if webhook_manager else 0
    enabled = sum(1 for w in webhook_manager.webhooks.values() if w.enabled) if webhook_manager else 0
    
    return {
        "total_deliveries": total,
        "successful": success,
        "failed": failed,
        "success_rate": round((success / total * 100) if total > 0 else 0, 2),
        "registered_webhooks": registered,
        "enabled_webhooks": enabled
    }


def _get_event_description(event: WebhookEventType) -> str:
    """Get description for webhook event type"""
    descriptions = {
        WebhookEventType.BUILD_STARTED: "Triggered when a new build starts",
        WebhookEventType.BUILD_COMPLETED: "Triggered when a build completes successfully",
        WebhookEventType.BUILD_FAILED: "Triggered when a build fails",
        WebhookEventType.CRITICAL_CVE_DETECTED: "Triggered when critical vulnerabilities are detected",
        WebhookEventType.REMEDIATION_APPLIED: "Triggered when auto-remediation is applied",
        WebhookEventType.REMEDIATION_FAILED: "Triggered when auto-remediation fails",
        WebhookEventType.IMAGE_DEPRECATED: "Triggered when an image is marked as deprecated",
        WebhookEventType.IMAGE_TOMBSTONED: "Triggered when an image is tombstoned",
        WebhookEventType.BASE_IMAGE_UPDATED: "Triggered when a base image update is available",
        WebhookEventType.POLICY_VIOLATION: "Triggered when a policy violation is detected",
        WebhookEventType.DRIFT_DETECTED: "Triggered when configuration drift is detected",
        WebhookEventType.DRIFT_RESOLVED: "Triggered when drift is resolved",
        WebhookEventType.EXCEPTION_REQUESTED: "Triggered when an exception is requested",
        WebhookEventType.EXCEPTION_APPROVED: "Triggered when an exception is approved",
        WebhookEventType.EXCEPTION_REJECTED: "Triggered when an exception is rejected",
        WebhookEventType.SLSA_ATTESTATION_GENERATED: "Triggered when SLSA attestation is generated",
        WebhookEventType.VEX_DOCUMENT_GENERATED: "Triggered when VEX document is generated"
    }
    return descriptions.get(event, event.value)


@router.get("")
async def list_webhooks():
    """List all registered webhooks"""
    webhooks = webhook_manager.get_webhooks()
    stats = await get_webhook_delivery_stats()
    
    return {
        "webhooks": webhooks,
        "stats": stats
    }


@router.post("")
async def create_webhook(webhook_data: Dict[str, Any]):
    """Register a new webhook"""
    required_fields = ["name", "destination", "url", "events"]
    for field in required_fields:
        if field not in webhook_data:
            raise HTTPException(status_code=400, detail=f"Missing required field: {field}")
    
    try:
        destination = WebhookDestination(webhook_data["destination"])
    except ValueError:
        raise HTTPException(
            status_code=400, 
            detail=f"Invalid destination. Must be one of: {[d.value for d in WebhookDestination]}"
        )
    
    try:
        events = [WebhookEventType(e) for e in webhook_data["events"]]
    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Invalid event type: {e}")
    
    config = WebhookConfig(
        name=webhook_data["name"],
        destination=destination,
        url=webhook_data["url"],
        events=events,
        channel=webhook_data.get("channel"),
        secret=webhook_data.get("secret"),
        enabled=webhook_data.get("enabled", True)
    )
    
    webhook_id = webhook_manager.register_webhook(config)
    
    # Store in database for persistence
    await db.webhooks.update_one(
        {"id": webhook_id},
        {"$set": {
            "id": webhook_id,
            "name": config.name,
            "destination": config.destination.value,
            "url": config.url,
            "events": [e.value for e in config.events],
            "channel": config.channel,
            "enabled": config.enabled,
            "created_at": datetime.now(timezone.utc).isoformat()
        }},
        upsert=True
    )
    
    return {
        "id": webhook_id,
        "message": f"Webhook '{config.name}' registered successfully"
    }


@router.delete("/{webhook_id}")
async def delete_webhook(webhook_id: str):
    """Delete a webhook"""
    if webhook_manager.unregister_webhook(webhook_id):
        await db.webhooks.delete_one({"id": webhook_id})
        return {"message": "Webhook deleted successfully"}
    
    raise HTTPException(status_code=404, detail="Webhook not found")


@router.patch("/{webhook_id}")
async def update_webhook(webhook_id: str, updates: Dict[str, Any]):
    """Update webhook configuration"""
    webhook = webhook_manager.update_webhook(webhook_id, updates)
    if not webhook:
        raise HTTPException(status_code=404, detail="Webhook not found")
    
    # Update in database
    await db.webhooks.update_one(
        {"id": webhook_id},
        {"$set": updates}
    )
    
    return {"message": "Webhook updated successfully", "webhook": webhook.to_dict()}


@router.post("/{webhook_id}/test")
async def test_webhook(webhook_id: str):
    """Send a test event to a webhook"""
    if webhook_id not in webhook_manager.webhooks:
        raise HTTPException(status_code=404, detail="Webhook not found")
    
    test_payload = {
        "build_id": "test-build-123",
        "image_tag": "secureforge/test:latest",
        "message": "This is a test notification from SecureImage Forge",
        "detail_url": "https://secureforge.enterprise/test"
    }
    
    # Temporarily enable all events for the test
    webhook = webhook_manager.webhooks[webhook_id]
    original_events = webhook.events
    webhook.events = [WebhookEventType.BUILD_COMPLETED]
    
    try:
        # Use async method for delivery
        deliveries = await webhook_manager.send_event_async(
            WebhookEventType.BUILD_COMPLETED,
            test_payload,
            sync=True
        )
        
        if deliveries and deliveries[0].get("status") == "success":
            return {
                "status": "success",
                "message": "Test webhook delivered successfully",
                "delivery": deliveries[0]
            }
        else:
            return {
                "status": "queued",
                "message": "Test webhook queued for delivery",
                "delivery": deliveries[0] if deliveries else None
            }
    finally:
        webhook.events = original_events


@router.get("/events")
async def list_webhook_events():
    """List all available webhook event types"""
    return {
        "events": [
            {"id": e.value, "name": e.name, "description": _get_event_description(e)}
            for e in WebhookEventType
        ]
    }


@router.get("/destinations")
async def list_webhook_destinations():
    """List supported webhook destinations"""
    return {
        "destinations": [
            {"id": "slack", "name": "Slack", "description": "Slack Incoming Webhooks with Block Kit"},
            {"id": "microsoft_teams", "name": "Microsoft Teams", "description": "Teams Incoming Webhooks with Adaptive Cards"},
            {"id": "discord", "name": "Discord", "description": "Discord Webhooks with Embeds"},
            {"id": "generic_webhook", "name": "Generic HTTP", "description": "Standard HTTP POST with JSON payload"}
        ]
    }


@router.get("/delivery-history")
async def get_webhook_delivery_history(limit: int = 50):
    """Get webhook delivery history from database"""
    # Fetch from database
    deliveries = await get_webhook_deliveries(limit)
    
    # Fallback to in-memory if DB is empty
    if not deliveries:
        deliveries = webhook_manager.get_delivery_history(limit)
    
    # Get stats from database
    stats = await get_webhook_delivery_stats()
    
    return {
        "deliveries": deliveries,
        "stats": stats
    }
