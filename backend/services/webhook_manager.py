"""Webhook Manager - ChatOps Integration"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
from enum import Enum
import uuid
import json

class WebhookEventType(str, Enum):
    BUILD_COMPLETED = "build.completed"
    BUILD_FAILED = "build.failed"
    CRITICAL_CVE_DETECTED = "vulnerability.critical"
    IMAGE_DEPRECATED = "lifecycle.deprecated"
    IMAGE_TOMBSTONED = "lifecycle.tombstoned"
    BASE_IMAGE_UPDATED = "evergreen.update_available"
    POLICY_VIOLATION = "policy.violation"
    DRIFT_DETECTED = "drift.detected"
    EXCEPTION_REQUESTED = "exception.requested"
    EXCEPTION_APPROVED = "exception.approved"

class WebhookDestination(str, Enum):
    SLACK = "slack"
    TEAMS = "microsoft_teams"
    DISCORD = "discord"
    GENERIC = "generic_webhook"

class WebhookConfig:
    """Webhook configuration"""
    
    def __init__(self, name: str, destination: WebhookDestination, url: str, 
                 events: List[WebhookEventType], channel: Optional[str] = None):
        self.id = str(uuid.uuid4())
        self.name = name
        self.destination = destination
        self.url = url
        self.events = events
        self.channel = channel
        self.enabled = True
        self.created_at = datetime.now(timezone.utc)

class WebhookManager:
    """Manage webhooks for ChatOps integration"""
    
    def __init__(self):
        self.webhooks: Dict[str, WebhookConfig] = {}
        self.delivery_log: List[Dict[str, Any]] = []
    
    def register_webhook(self, config: WebhookConfig):
        """Register a new webhook"""
        self.webhooks[config.id] = config
    
    def send_event(self, event_type: WebhookEventType, payload: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Send event to all matching webhooks"""
        deliveries = []
        
        for webhook in self.webhooks.values():
            if not webhook.enabled:
                continue
            
            if event_type not in webhook.events:
                continue
            
            message = self._format_message(webhook.destination, event_type, payload)
            delivery = self._deliver_webhook(webhook, event_type, message)
            deliveries.append(delivery)
            self.delivery_log.append(delivery)
        
        return deliveries
    
    def _format_message(self, destination: WebhookDestination, event_type: WebhookEventType, 
                       payload: Dict[str, Any]) -> Dict[str, Any]:
        """Format message for specific destination"""
        
        if destination == WebhookDestination.SLACK:
            return self._format_slack_message(event_type, payload)
        elif destination == WebhookDestination.TEAMS:
            return self._format_teams_message(event_type, payload)
        else:
            return {"event_type": event_type.value, "payload": payload}
    
    def _format_slack_message(self, event_type: WebhookEventType, payload: Dict[str, Any]) -> Dict[str, Any]:
        """Format Slack message"""
        
        color_map = {
            WebhookEventType.BUILD_COMPLETED: "good",
            WebhookEventType.BUILD_FAILED: "danger",
            WebhookEventType.CRITICAL_CVE_DETECTED: "danger",
            WebhookEventType.BASE_IMAGE_UPDATED: "warning",
            WebhookEventType.POLICY_VIOLATION: "danger"
        }
        
        icon_map = {
            WebhookEventType.BUILD_COMPLETED: ":white_check_mark:",
            WebhookEventType.BUILD_FAILED: ":x:",
            WebhookEventType.CRITICAL_CVE_DETECTED: ":warning:",
            WebhookEventType.BASE_IMAGE_UPDATED: ":arrows_counterclockwise:",
            WebhookEventType.POLICY_VIOLATION: ":no_entry:"
        }
        
        message = {
            "attachments": [
                {
                    "color": color_map.get(event_type, "#439FE0"),
                    "title": f"{icon_map.get(event_type, ':information_source:')} {self._get_event_title(event_type)}",
                    "text": self._get_event_description(event_type, payload),
                    "fields": self._get_slack_fields(event_type, payload),
                    "footer": "SecureImage Forge",
                    "ts": int(datetime.now(timezone.utc).timestamp())
                }
            ]
        }
        
        # Add action buttons
        if event_type == WebhookEventType.BASE_IMAGE_UPDATED:
            message["attachments"][0]["actions"] = [
                {
                    "type": "button",
                    "text": "Review & Deploy",
                    "url": payload.get('pr_url', '#')
                }
            ]
        
        return message
    
    def _format_teams_message(self, event_type: WebhookEventType, payload: Dict[str, Any]) -> Dict[str, Any]:
        """Format Microsoft Teams message"""
        return {
            "@type": "MessageCard",
            "@context": "https://schema.org/extensions",
            "summary": self._get_event_title(event_type),
            "themeColor": "0078D7",
            "title": self._get_event_title(event_type),
            "text": self._get_event_description(event_type, payload),
            "potentialAction": [
                {
                    "@type": "OpenUri",
                    "name": "View Details",
                    "targets": [
                        {
                            "os": "default",
                            "uri": payload.get('detail_url', 'https://secureforge.enterprise')
                        }
                    ]
                }
            ]
        }
    
    def _get_event_title(self, event_type: WebhookEventType) -> str:
        """Get human-readable event title"""
        titles = {
            WebhookEventType.BUILD_COMPLETED: "Build Completed Successfully",
            WebhookEventType.BUILD_FAILED: "Build Failed",
            WebhookEventType.CRITICAL_CVE_DETECTED: "Critical Vulnerability Detected",
            WebhookEventType.BASE_IMAGE_UPDATED: "Base Image Update Available",
            WebhookEventType.POLICY_VIOLATION: "Policy Violation Detected",
            WebhookEventType.DRIFT_DETECTED: "Configuration Drift Detected"
        }
        return titles.get(event_type, event_type.value)
    
    def _get_event_description(self, event_type: WebhookEventType, payload: Dict[str, Any]) -> str:
        """Get event description"""
        if event_type == WebhookEventType.BUILD_COMPLETED:
            return f"Image `{payload.get('image_tag')}` has been successfully built and signed."
        elif event_type == WebhookEventType.CRITICAL_CVE_DETECTED:
            return f"{payload.get('cve_count', 0)} critical vulnerabilities detected in `{payload.get('image_tag')}`."
        elif event_type == WebhookEventType.BASE_IMAGE_UPDATED:
            return f"New {payload.get('base_image')} patch available. {payload.get('reason')}"
        else:
            return payload.get('message', 'Event triggered')
    
    def _get_slack_fields(self, event_type: WebhookEventType, payload: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Get Slack message fields"""
        fields = []
        
        if 'image_tag' in payload:
            fields.append({"title": "Image", "value": payload['image_tag'], "short": True})
        
        if 'build_id' in payload:
            fields.append({"title": "Build ID", "value": payload['build_id'][:8], "short": True})
        
        if event_type == WebhookEventType.CRITICAL_CVE_DETECTED:
            fields.append({"title": "Critical CVEs", "value": str(payload.get('cve_count', 0)), "short": True})
        
        return fields
    
    def _deliver_webhook(self, webhook: WebhookConfig, event_type: WebhookEventType, 
                        message: Dict[str, Any]) -> Dict[str, Any]:
        """Deliver webhook (simulated)"""
        # In production, would make actual HTTP POST
        delivery = {
            "id": str(uuid.uuid4()),
            "webhook_id": webhook.id,
            "webhook_name": webhook.name,
            "event_type": event_type.value,
            "destination": webhook.destination.value,
            "status": "success",
            "delivered_at": datetime.now(timezone.utc).isoformat(),
            "response_code": 200
        }
        return delivery