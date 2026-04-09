"""
Webhook Manager - ChatOps Integration with Real HTTP Delivery

Supports:
- Slack Incoming Webhooks
- Microsoft Teams Incoming Webhooks
- Discord Webhooks
- Generic HTTP Webhooks

Features:
- Async HTTP delivery with httpx
- Retry logic with exponential backoff
- Delivery logging and history
- Rate limiting
- Message formatting per platform
"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone, timedelta
from enum import Enum
import uuid
import json
import asyncio
import logging
import httpx

logger = logging.getLogger(__name__)


class WebhookEventType(str, Enum):
    """Supported webhook event types"""
    BUILD_STARTED = "build.started"
    BUILD_COMPLETED = "build.completed"
    BUILD_FAILED = "build.failed"
    CRITICAL_CVE_DETECTED = "vulnerability.critical"
    REMEDIATION_APPLIED = "remediation.applied"
    REMEDIATION_FAILED = "remediation.failed"
    IMAGE_DEPRECATED = "lifecycle.deprecated"
    IMAGE_TOMBSTONED = "lifecycle.tombstoned"
    BASE_IMAGE_UPDATED = "evergreen.update_available"
    POLICY_VIOLATION = "policy.violation"
    DRIFT_DETECTED = "drift.detected"
    DRIFT_RESOLVED = "drift.resolved"
    EXCEPTION_REQUESTED = "exception.requested"
    EXCEPTION_APPROVED = "exception.approved"
    EXCEPTION_REJECTED = "exception.rejected"
    SLSA_ATTESTATION_GENERATED = "slsa.attestation_generated"
    VEX_DOCUMENT_GENERATED = "vex.document_generated"


class WebhookDestination(str, Enum):
    """Supported webhook destinations"""
    SLACK = "slack"
    TEAMS = "microsoft_teams"
    DISCORD = "discord"
    GENERIC = "generic_webhook"


class DeliveryStatus(str, Enum):
    """Webhook delivery status"""
    PENDING = "pending"
    SUCCESS = "success"
    FAILED = "failed"
    RETRYING = "retrying"


class WebhookConfig:
    """Webhook configuration"""
    
    def __init__(
        self,
        name: str,
        destination: WebhookDestination,
        url: str,
        events: List[WebhookEventType],
        channel: Optional[str] = None,
        secret: Optional[str] = None,
        enabled: bool = True
    ):
        self.id = str(uuid.uuid4())
        self.name = name
        self.destination = destination
        self.url = url
        self.events = events
        self.channel = channel
        self.secret = secret
        self.enabled = enabled
        self.created_at = datetime.now(timezone.utc)
        self.last_delivery_at = None
        self.delivery_count = 0
        self.failure_count = 0
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "destination": self.destination.value,
            "url": self.url[:50] + "..." if len(self.url) > 50 else self.url,  # Mask URL
            "events": [e.value for e in self.events],
            "channel": self.channel,
            "enabled": self.enabled,
            "created_at": self.created_at.isoformat(),
            "last_delivery_at": self.last_delivery_at.isoformat() if self.last_delivery_at else None,
            "delivery_count": self.delivery_count,
            "failure_count": self.failure_count
        }


class WebhookManager:
    """
    Manage webhooks for ChatOps integration.
    
    Supports real HTTP delivery to Slack, Teams, Discord, and generic webhooks.
    """
    
    def __init__(self, base_url: str = "https://secureforge.enterprise"):
        self.webhooks: Dict[str, WebhookConfig] = {}
        self.delivery_log: List[Dict[str, Any]] = []
        self.base_url = base_url
        self.max_retries = 3
        self.retry_delay = 1  # seconds
        self._rate_limit_window = 60  # seconds
        self._rate_limit_max = 30  # max deliveries per window
        self._delivery_timestamps: List[datetime] = []
    
    def register_webhook(self, config: WebhookConfig) -> str:
        """Register a new webhook"""
        self.webhooks[config.id] = config
        logger.info(f"Registered webhook: {config.name} ({config.destination.value})")
        return config.id
    
    def unregister_webhook(self, webhook_id: str) -> bool:
        """Unregister a webhook"""
        if webhook_id in self.webhooks:
            del self.webhooks[webhook_id]
            return True
        return False
    
    def update_webhook(self, webhook_id: str, updates: Dict[str, Any]) -> Optional[WebhookConfig]:
        """Update webhook configuration"""
        if webhook_id not in self.webhooks:
            return None
        
        webhook = self.webhooks[webhook_id]
        if 'enabled' in updates:
            webhook.enabled = updates['enabled']
        if 'events' in updates:
            webhook.events = [WebhookEventType(e) for e in updates['events']]
        if 'name' in updates:
            webhook.name = updates['name']
        
        return webhook
    
    def get_webhooks(self) -> List[Dict[str, Any]]:
        """Get all registered webhooks"""
        return [w.to_dict() for w in self.webhooks.values()]
    
    def send_event(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """
        Send event to all matching webhooks with REAL HTTP delivery.
        Uses synchronous httpx client for immediate delivery.
        """
        deliveries = []
        
        for webhook in self.webhooks.values():
            if not webhook.enabled:
                continue
            
            if event_type not in webhook.events:
                continue
            
            # Check rate limit
            if not self._check_rate_limit():
                logger.warning(f"Rate limit exceeded for webhook {webhook.name}")
                continue
            
            # Format message for destination
            message = self._format_message(webhook.destination, event_type, payload)
            
            # Create delivery record
            delivery = self._create_pending_delivery(webhook, event_type)
            
            # Actually deliver the webhook via HTTP
            try:
                with httpx.Client(timeout=10.0) as client:
                    headers = {"Content-Type": "application/json"}
                    
                    if webhook.secret:
                        headers["X-Webhook-Secret"] = webhook.secret
                    
                    response = client.post(
                        webhook.url,
                        json=message,
                        headers=headers
                    )
                    
                    if response.is_success:
                        delivery["status"] = DeliveryStatus.SUCCESS.value
                        delivery["response_code"] = response.status_code
                        delivery["delivered_at"] = datetime.now(timezone.utc).isoformat()
                        logger.info(f"Webhook delivered successfully: {webhook.name} - {event_type.value}")
                    else:
                        delivery["status"] = DeliveryStatus.FAILED.value
                        delivery["response_code"] = response.status_code
                        delivery["error"] = response.text[:200]
                        webhook.failure_count += 1
                        logger.warning(f"Webhook delivery failed: {webhook.name} - {response.status_code}")
                        
            except httpx.TimeoutException:
                delivery["status"] = DeliveryStatus.FAILED.value
                delivery["error"] = "Request timeout"
                webhook.failure_count += 1
                logger.error(f"Webhook timeout: {webhook.name}")
            except httpx.RequestError as e:
                delivery["status"] = DeliveryStatus.FAILED.value
                delivery["error"] = str(e)[:200]
                webhook.failure_count += 1
                logger.error(f"Webhook request error: {webhook.name} - {e}")
            except Exception as e:
                delivery["status"] = DeliveryStatus.FAILED.value
                delivery["error"] = f"Unexpected error: {str(e)[:100]}"
                webhook.failure_count += 1
                logger.error(f"Webhook unexpected error: {webhook.name} - {e}")
            
            deliveries.append(delivery)
            self.delivery_log.append(delivery)
            
            # Update webhook stats
            webhook.last_delivery_at = datetime.now(timezone.utc)
            webhook.delivery_count += 1
        
        return deliveries
        
        return deliveries
    
    async def send_event_async(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any],
        sync: bool = False
    ) -> List[Dict[str, Any]]:
        """
        Send event to all matching webhooks.
        
        Args:
            event_type: Type of event to send
            payload: Event payload data
            sync: If True, wait for delivery confirmation
        
        Returns:
            List of delivery results
        """
        deliveries = []
        
        for webhook in self.webhooks.values():
            if not webhook.enabled:
                continue
            
            if event_type not in webhook.events:
                continue
            
            # Check rate limit
            if not self._check_rate_limit():
                logger.warning(f"Rate limit exceeded for webhook {webhook.name}")
                continue
            
            # Format message for destination
            message = self._format_message(webhook.destination, event_type, payload)
            
            # Deliver webhook
            if sync:
                delivery = await self._deliver_webhook_async(webhook, event_type, message)
            else:
                # Fire and forget with logging
                delivery = self._create_pending_delivery(webhook, event_type)
                asyncio.create_task(self._deliver_with_retry(webhook, event_type, message, delivery['id']))
            
            deliveries.append(delivery)
            self.delivery_log.append(delivery)
            
            # Update webhook stats
            webhook.last_delivery_at = datetime.now(timezone.utc)
            webhook.delivery_count += 1
        
        return deliveries
    
    def send_event_sync(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> List[Dict[str, Any]]:
        """Synchronous wrapper for send_event (for non-async contexts)"""
        # Check if we're already in an event loop
        try:
            loop = asyncio.get_running_loop()
            # We're in an async context, create a task
            import concurrent.futures
            with concurrent.futures.ThreadPoolExecutor() as executor:
                future = executor.submit(
                    asyncio.run,
                    self.send_event(event_type, payload, sync=True)
                )
                return future.result(timeout=30)
        except RuntimeError:
            # No running event loop, safe to use asyncio.run
            return asyncio.run(self.send_event(event_type, payload, sync=True))
    
    def _check_rate_limit(self) -> bool:
        """Check if we're within rate limits"""
        now = datetime.now(timezone.utc)
        window_start = now - timedelta(seconds=self._rate_limit_window)
        
        # Clean old timestamps
        self._delivery_timestamps = [
            ts for ts in self._delivery_timestamps if ts > window_start
        ]
        
        if len(self._delivery_timestamps) >= self._rate_limit_max:
            return False
        
        self._delivery_timestamps.append(now)
        return True
    
    def _create_pending_delivery(
        self,
        webhook: WebhookConfig,
        event_type: WebhookEventType
    ) -> Dict[str, Any]:
        """Create a pending delivery record"""
        return {
            "id": str(uuid.uuid4()),
            "webhook_id": webhook.id,
            "webhook_name": webhook.name,
            "event_type": event_type.value,
            "destination": webhook.destination.value,
            "status": DeliveryStatus.PENDING.value,
            "created_at": datetime.now(timezone.utc).isoformat(),
            "delivered_at": None,
            "response_code": None,
            "error": None,
            "retries": 0
        }
    
    async def _deliver_webhook_async(
        self,
        webhook: WebhookConfig,
        event_type: WebhookEventType,
        message: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Deliver webhook with async HTTP client"""
        delivery = self._create_pending_delivery(webhook, event_type)
        
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                headers = {"Content-Type": "application/json"}
                
                # Add secret header if configured
                if webhook.secret:
                    headers["X-Webhook-Secret"] = webhook.secret
                
                response = await client.post(
                    webhook.url,
                    json=message,
                    headers=headers
                )
                
                delivery["status"] = DeliveryStatus.SUCCESS.value if response.is_success else DeliveryStatus.FAILED.value
                delivery["response_code"] = response.status_code
                delivery["delivered_at"] = datetime.now(timezone.utc).isoformat()
                
                if not response.is_success:
                    delivery["error"] = response.text[:200]
                    webhook.failure_count += 1
                
        except httpx.TimeoutException:
            delivery["status"] = DeliveryStatus.FAILED.value
            delivery["error"] = "Request timeout"
            webhook.failure_count += 1
        except httpx.RequestError as e:
            delivery["status"] = DeliveryStatus.FAILED.value
            delivery["error"] = str(e)[:200]
            webhook.failure_count += 1
        except Exception as e:
            delivery["status"] = DeliveryStatus.FAILED.value
            delivery["error"] = f"Unexpected error: {str(e)[:100]}"
            webhook.failure_count += 1
        
        return delivery
    
    async def _deliver_with_retry(
        self,
        webhook: WebhookConfig,
        event_type: WebhookEventType,
        message: Dict[str, Any],
        delivery_id: str
    ):
        """Deliver webhook with retry logic"""
        for attempt in range(self.max_retries):
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    headers = {"Content-Type": "application/json"}
                    
                    if webhook.secret:
                        headers["X-Webhook-Secret"] = webhook.secret
                    
                    response = await client.post(
                        webhook.url,
                        json=message,
                        headers=headers
                    )
                    
                    # Update delivery log
                    for delivery in self.delivery_log:
                        if delivery['id'] == delivery_id:
                            if response.is_success:
                                delivery["status"] = DeliveryStatus.SUCCESS.value
                            else:
                                delivery["status"] = DeliveryStatus.FAILED.value
                                delivery["error"] = response.text[:200]
                            delivery["response_code"] = response.status_code
                            delivery["delivered_at"] = datetime.now(timezone.utc).isoformat()
                            delivery["retries"] = attempt
                            break
                    
                    if response.is_success:
                        logger.info(f"Webhook delivered: {webhook.name} - {event_type.value}")
                        return
                    
            except Exception as e:
                logger.warning(f"Webhook delivery attempt {attempt + 1} failed: {e}")
            
            # Wait before retry (exponential backoff)
            if attempt < self.max_retries - 1:
                await asyncio.sleep(self.retry_delay * (2 ** attempt))
        
        # All retries failed
        webhook.failure_count += 1
        logger.error(f"Webhook delivery failed after {self.max_retries} attempts: {webhook.name}")
    
    def _format_message(
        self,
        destination: WebhookDestination,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Format message for specific destination platform"""
        if destination == WebhookDestination.SLACK:
            return self._format_slack_message(event_type, payload)
        elif destination == WebhookDestination.TEAMS:
            return self._format_teams_message(event_type, payload)
        elif destination == WebhookDestination.DISCORD:
            return self._format_discord_message(event_type, payload)
        else:
            return self._format_generic_message(event_type, payload)
    
    def _format_slack_message(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Format Slack Block Kit message"""
        color_map = {
            WebhookEventType.BUILD_COMPLETED: "#36a64f",  # green
            WebhookEventType.BUILD_FAILED: "#dc3545",  # red
            WebhookEventType.CRITICAL_CVE_DETECTED: "#dc3545",
            WebhookEventType.REMEDIATION_APPLIED: "#36a64f",
            WebhookEventType.BASE_IMAGE_UPDATED: "#ffc107",  # yellow
            WebhookEventType.POLICY_VIOLATION: "#dc3545",
            WebhookEventType.DRIFT_DETECTED: "#ffc107",
            WebhookEventType.EXCEPTION_APPROVED: "#36a64f",
            WebhookEventType.SLSA_ATTESTATION_GENERATED: "#0d6efd",  # blue
            WebhookEventType.VEX_DOCUMENT_GENERATED: "#0d6efd"
        }
        
        emoji_map = {
            WebhookEventType.BUILD_COMPLETED: ":white_check_mark:",
            WebhookEventType.BUILD_FAILED: ":x:",
            WebhookEventType.CRITICAL_CVE_DETECTED: ":warning:",
            WebhookEventType.REMEDIATION_APPLIED: ":wrench:",
            WebhookEventType.BASE_IMAGE_UPDATED: ":arrows_counterclockwise:",
            WebhookEventType.POLICY_VIOLATION: ":no_entry:",
            WebhookEventType.DRIFT_DETECTED: ":mag:",
            WebhookEventType.EXCEPTION_APPROVED: ":ballot_box_with_check:",
            WebhookEventType.SLSA_ATTESTATION_GENERATED: ":lock:",
            WebhookEventType.VEX_DOCUMENT_GENERATED: ":page_facing_up:"
        }
        
        title = self._get_event_title(event_type)
        description = self._get_event_description(event_type, payload)
        color = color_map.get(event_type, "#439FE0")
        emoji = emoji_map.get(event_type, ":information_source:")
        
        blocks = [
            {
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": f"{emoji} {title}",
                    "emoji": True
                }
            },
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": description
                }
            }
        ]
        
        # Add fields based on payload
        fields = []
        if payload.get('build_id'):
            fields.append({
                "type": "mrkdwn",
                "text": f"*Build ID:*\n`{payload['build_id'][:8]}`"
            })
        if payload.get('image_tag'):
            fields.append({
                "type": "mrkdwn",
                "text": f"*Image:*\n`{payload['image_tag']}`"
            })
        if payload.get('slsa_level'):
            fields.append({
                "type": "mrkdwn",
                "text": f"*SLSA Level:*\n{payload['slsa_level']}"
            })
        if payload.get('cve_count'):
            fields.append({
                "type": "mrkdwn",
                "text": f"*Critical CVEs:*\n{payload['cve_count']}"
            })
        if payload.get('fixes_count'):
            fields.append({
                "type": "mrkdwn",
                "text": f"*Fixes Applied:*\n{payload['fixes_count']}"
            })
        
        if fields:
            blocks.append({
                "type": "section",
                "fields": fields[:10]  # Slack limit
            })
        
        # Add action button if detail_url provided
        if payload.get('detail_url'):
            blocks.append({
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "View Details",
                            "emoji": True
                        },
                        "url": payload['detail_url'],
                        "action_id": "view_details"
                    }
                ]
            })
        
        # Add context footer
        blocks.append({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": f"SecureImage Forge | {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M UTC')}"
                }
            ]
        })
        
        return {
            "blocks": blocks,
            "attachments": [{"color": color, "blocks": []}]  # Color bar
        }
    
    def _format_teams_message(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Format Microsoft Teams Adaptive Card message"""
        title = self._get_event_title(event_type)
        description = self._get_event_description(event_type, payload)
        
        # Determine theme color
        theme_colors = {
            WebhookEventType.BUILD_COMPLETED: "Good",
            WebhookEventType.BUILD_FAILED: "Attention",
            WebhookEventType.CRITICAL_CVE_DETECTED: "Attention",
            WebhookEventType.REMEDIATION_APPLIED: "Good"
        }
        
        facts = []
        if payload.get('build_id'):
            facts.append({"title": "Build ID", "value": payload['build_id'][:8]})
        if payload.get('image_tag'):
            facts.append({"title": "Image", "value": payload['image_tag']})
        if payload.get('slsa_level'):
            facts.append({"title": "SLSA Level", "value": str(payload['slsa_level'])})
        
        card = {
            "type": "message",
            "attachments": [
                {
                    "contentType": "application/vnd.microsoft.card.adaptive",
                    "content": {
                        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                        "type": "AdaptiveCard",
                        "version": "1.4",
                        "body": [
                            {
                                "type": "TextBlock",
                                "size": "Large",
                                "weight": "Bolder",
                                "text": title,
                                "style": "heading"
                            },
                            {
                                "type": "TextBlock",
                                "text": description,
                                "wrap": True
                            },
                            {
                                "type": "FactSet",
                                "facts": facts
                            } if facts else None
                        ],
                        "actions": [
                            {
                                "type": "Action.OpenUrl",
                                "title": "View Details",
                                "url": payload.get('detail_url', self.base_url)
                            }
                        ]
                    }
                }
            ]
        }
        
        # Remove None items
        card["attachments"][0]["content"]["body"] = [
            b for b in card["attachments"][0]["content"]["body"] if b
        ]
        
        return card
    
    def _format_discord_message(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Format Discord embed message"""
        title = self._get_event_title(event_type)
        description = self._get_event_description(event_type, payload)
        
        color_map = {
            WebhookEventType.BUILD_COMPLETED: 3066993,  # green
            WebhookEventType.BUILD_FAILED: 15158332,  # red
            WebhookEventType.CRITICAL_CVE_DETECTED: 15158332,
            WebhookEventType.REMEDIATION_APPLIED: 3066993,
            WebhookEventType.SLSA_ATTESTATION_GENERATED: 3447003  # blue
        }
        
        fields = []
        if payload.get('build_id'):
            fields.append({"name": "Build ID", "value": f"`{payload['build_id'][:8]}`", "inline": True})
        if payload.get('image_tag'):
            fields.append({"name": "Image", "value": f"`{payload['image_tag']}`", "inline": True})
        if payload.get('slsa_level'):
            fields.append({"name": "SLSA Level", "value": str(payload['slsa_level']), "inline": True})
        
        return {
            "embeds": [
                {
                    "title": title,
                    "description": description,
                    "color": color_map.get(event_type, 5814783),
                    "fields": fields,
                    "footer": {
                        "text": "SecureImage Forge"
                    },
                    "timestamp": datetime.now(timezone.utc).isoformat()
                }
            ]
        }
    
    def _format_generic_message(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Format generic webhook message"""
        return {
            "event_type": event_type.value,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "source": "SecureImage Forge",
            "title": self._get_event_title(event_type),
            "description": self._get_event_description(event_type, payload),
            "payload": payload
        }
    
    def _get_event_title(self, event_type: WebhookEventType) -> str:
        """Get human-readable event title"""
        titles = {
            WebhookEventType.BUILD_STARTED: "Build Started",
            WebhookEventType.BUILD_COMPLETED: "Build Completed Successfully",
            WebhookEventType.BUILD_FAILED: "Build Failed",
            WebhookEventType.CRITICAL_CVE_DETECTED: "Critical Vulnerability Detected",
            WebhookEventType.REMEDIATION_APPLIED: "Auto-Remediation Applied",
            WebhookEventType.REMEDIATION_FAILED: "Remediation Failed",
            WebhookEventType.BASE_IMAGE_UPDATED: "Base Image Update Available",
            WebhookEventType.POLICY_VIOLATION: "Policy Violation Detected",
            WebhookEventType.DRIFT_DETECTED: "Configuration Drift Detected",
            WebhookEventType.DRIFT_RESOLVED: "Drift Resolved",
            WebhookEventType.EXCEPTION_REQUESTED: "Exception Request Submitted",
            WebhookEventType.EXCEPTION_APPROVED: "Exception Request Approved",
            WebhookEventType.EXCEPTION_REJECTED: "Exception Request Rejected",
            WebhookEventType.SLSA_ATTESTATION_GENERATED: "SLSA Attestation Generated",
            WebhookEventType.VEX_DOCUMENT_GENERATED: "VEX Document Generated"
        }
        return titles.get(event_type, event_type.value)
    
    def _get_event_description(
        self,
        event_type: WebhookEventType,
        payload: Dict[str, Any]
    ) -> str:
        """Get event description"""
        if event_type == WebhookEventType.BUILD_COMPLETED:
            return f"Image `{payload.get('image_tag', 'unknown')}` has been successfully built, scanned, and signed."
        
        elif event_type == WebhookEventType.BUILD_FAILED:
            return f"Build failed for `{payload.get('image_tag', 'unknown')}`. Reason: {payload.get('error', 'Unknown error')}"
        
        elif event_type == WebhookEventType.CRITICAL_CVE_DETECTED:
            return f"**{payload.get('cve_count', 0)}** critical vulnerabilities detected in `{payload.get('image_tag', 'unknown')}`. Immediate attention required."
        
        elif event_type == WebhookEventType.REMEDIATION_APPLIED:
            return f"Auto-remediation applied **{payload.get('fixes_count', 0)}** fixes to `{payload.get('image_tag', 'unknown')}`."
        
        elif event_type == WebhookEventType.SLSA_ATTESTATION_GENERATED:
            return f"SLSA Level {payload.get('slsa_level', 3)} attestation generated for `{payload.get('image_tag', 'unknown')}`. Provenance is verified."
        
        elif event_type == WebhookEventType.VEX_DOCUMENT_GENERATED:
            fp_rate = payload.get('false_positive_rate', 0)
            return f"VEX analysis complete. **{fp_rate}%** false positive rate. {payload.get('not_affected', 0)} vulnerabilities determined non-exploitable."
        
        elif event_type == WebhookEventType.DRIFT_DETECTED:
            return f"Configuration drift detected in **{payload.get('namespace', 'unknown')}** namespace. {payload.get('critical_drifts', 0)} critical issues found."
        
        elif event_type == WebhookEventType.EXCEPTION_APPROVED:
            return f"Exception request approved by {payload.get('approver', 'admin')}. Expires: {payload.get('expires_at', 'N/A')}"
        
        else:
            return payload.get('message', f'Event triggered: {event_type.value}')
    
    def get_delivery_history(self, limit: int = 50) -> List[Dict[str, Any]]:
        """Get recent delivery history"""
        return sorted(
            self.delivery_log[-limit:],
            key=lambda x: x.get('created_at', ''),
            reverse=True
        )
    
    def get_delivery_stats(self) -> Dict[str, Any]:
        """Get delivery statistics"""
        total = len(self.delivery_log)
        success = sum(1 for d in self.delivery_log if d['status'] == DeliveryStatus.SUCCESS.value)
        failed = sum(1 for d in self.delivery_log if d['status'] == DeliveryStatus.FAILED.value)
        
        return {
            "total_deliveries": total,
            "successful": success,
            "failed": failed,
            "success_rate": round((success / total * 100) if total > 0 else 0, 2),
            "registered_webhooks": len(self.webhooks),
            "enabled_webhooks": sum(1 for w in self.webhooks.values() if w.enabled)
        }
