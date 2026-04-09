"""
Test P1 Features: Real Slack Webhook Integration and Sigstore Availability
Tests:
- Slack webhook is registered and active
- Test webhook endpoint sends actual HTTP request to Slack (code 200)
- Build completion triggers Slack notification
- SLSA attestation generation triggers Slack notification
- VEX document generation triggers Slack notification
- Webhook delivery history shows successful deliveries
- Sigstore library is installed and available for signing
- SLSA attestation indicates Sigstore availability
"""
import pytest
import requests
import os
import time

BASE_URL = os.environ.get('REACT_APP_BACKEND_URL', '').rstrip('/')

class TestSlackWebhookIntegration:
    """Test real Slack webhook integration"""
    
    def test_webhook_registered_and_active(self):
        """Verify Slack webhook is registered and active in the system"""
        response = requests.get(f"{BASE_URL}/api/webhooks")
        assert response.status_code == 200
        
        data = response.json()
        webhooks = data.get("webhooks", [])
        
        # Find the Slack webhook
        slack_webhook = None
        for wh in webhooks:
            if wh.get("destination") == "slack" and wh.get("enabled"):
                slack_webhook = wh
                break
        
        assert slack_webhook is not None, "No active Slack webhook found"
        assert slack_webhook.get("name") == "Production Security Alerts"
        assert "hooks.slack.com" in slack_webhook.get("url", "")
        assert slack_webhook.get("enabled") is True
        
        # Verify subscribed events
        events = slack_webhook.get("events", [])
        assert "build.completed" in events
        assert "slsa.attestation_generated" in events
        assert "vex.document_generated" in events
        
        print(f"PASS: Slack webhook '{slack_webhook['name']}' is registered and active")
        print(f"  - Webhook ID: {slack_webhook['id']}")
        print(f"  - Events subscribed: {len(events)}")
        print(f"  - Delivery count: {slack_webhook.get('delivery_count', 0)}")
    
    def test_webhook_stats(self):
        """Verify webhook stats show successful deliveries"""
        # First, trigger a test webhook to ensure there's at least one delivery
        webhooks_response = requests.get(f"{BASE_URL}/api/webhooks")
        assert webhooks_response.status_code == 200
        
        webhooks = webhooks_response.json().get("webhooks", [])
        if webhooks:
            webhook_id = webhooks[0]["id"]
            # Trigger a test delivery
            requests.post(f"{BASE_URL}/api/webhooks/{webhook_id}/test")
            time.sleep(0.5)  # Wait for delivery
        
        # Now check stats
        response = requests.get(f"{BASE_URL}/api/webhooks")
        assert response.status_code == 200
        
        data = response.json()
        stats = data.get("stats", {})
        
        assert stats.get("registered_webhooks", 0) >= 1
        assert stats.get("enabled_webhooks", 0) >= 1
        
        # Check delivery history for actual delivery count
        history_response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=50")
        assert history_response.status_code == 200
        deliveries = history_response.json().get("deliveries", [])
        
        # Verify we have deliveries
        assert len(deliveries) >= 1, "No deliveries found in history"
        
        print(f"PASS: Webhook stats show healthy delivery metrics")
        print(f"  - Registered webhooks: {stats.get('registered_webhooks')}")
        print(f"  - Stats total_deliveries: {stats.get('total_deliveries')}")
        print(f"  - History deliveries: {len(deliveries)}")
        print(f"  - Success rate: {stats.get('success_rate')}%")
    
    def test_webhook_test_endpoint_sends_real_http(self):
        """Test webhook endpoint sends actual HTTP request to Slack (code 200)"""
        # First get the webhook ID
        response = requests.get(f"{BASE_URL}/api/webhooks")
        assert response.status_code == 200
        
        webhooks = response.json().get("webhooks", [])
        slack_webhook = next((wh for wh in webhooks if wh.get("destination") == "slack"), None)
        assert slack_webhook is not None, "No Slack webhook found"
        
        webhook_id = slack_webhook["id"]
        
        # Test the webhook
        test_response = requests.post(f"{BASE_URL}/api/webhooks/{webhook_id}/test")
        assert test_response.status_code == 200
        
        data = test_response.json()
        assert data.get("status") == "success"
        assert "delivered successfully" in data.get("message", "").lower()
        
        # Verify delivery details
        delivery = data.get("delivery", {})
        assert delivery.get("status") == "success"
        assert delivery.get("response_code") == 200
        assert delivery.get("error") is None
        
        print(f"PASS: Test webhook sent real HTTP request to Slack")
        print(f"  - Response code: {delivery.get('response_code')}")
        print(f"  - Delivery ID: {delivery.get('id')}")
    
    def test_delivery_history_shows_successful_deliveries(self):
        """Verify webhook delivery history shows successful deliveries"""
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=20")
        assert response.status_code == 200
        
        data = response.json()
        deliveries = data.get("deliveries", [])
        
        assert len(deliveries) >= 1, "No deliveries found in history"
        
        # Check for successful deliveries
        successful = [d for d in deliveries if d.get("status") == "success"]
        assert len(successful) >= 1, "No successful deliveries found"
        
        # Verify delivery details
        for delivery in successful[:3]:  # Check first 3
            assert delivery.get("response_code") == 200
            assert delivery.get("error") is None
            assert delivery.get("destination") == "slack"
        
        print(f"PASS: Delivery history shows {len(successful)} successful deliveries")
        
        # Check for different event types
        event_types = set(d.get("event_type") for d in successful)
        print(f"  - Event types delivered: {event_types}")


class TestSLSAAttestationWithSlackNotification:
    """Test SLSA attestation triggers Slack notification"""
    
    def test_slsa_attestation_triggers_slack_notification(self):
        """SLSA attestation generation triggers Slack notification"""
        # Get a completed build
        builds_response = requests.get(f"{BASE_URL}/api/builds")
        assert builds_response.status_code == 200
        
        builds = builds_response.json()
        completed_build = next((b for b in builds if b.get("status") == "completed"), None)
        assert completed_build is not None, "No completed build found"
        
        build_id = completed_build["id"]
        
        # Get initial delivery count
        initial_history = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=50").json()
        initial_slsa_count = len([d for d in initial_history.get("deliveries", []) 
                                   if d.get("event_type") == "slsa.attestation_generated"])
        
        # Generate SLSA attestation
        slsa_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/slsa?level=3")
        assert slsa_response.status_code == 200
        
        slsa_data = slsa_response.json()
        assert slsa_data.get("provenance") is not None
        
        # Wait a moment for webhook delivery
        time.sleep(0.5)
        
        # Check delivery history for SLSA notification
        final_history = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=50").json()
        final_slsa_count = len([d for d in final_history.get("deliveries", []) 
                                 if d.get("event_type") == "slsa.attestation_generated"])
        
        assert final_slsa_count > initial_slsa_count, "SLSA attestation did not trigger Slack notification"
        
        # Verify the latest SLSA delivery was successful
        slsa_deliveries = [d for d in final_history.get("deliveries", []) 
                          if d.get("event_type") == "slsa.attestation_generated"]
        latest_slsa = slsa_deliveries[0] if slsa_deliveries else None
        
        assert latest_slsa is not None
        assert latest_slsa.get("status") == "success"
        assert latest_slsa.get("response_code") == 200
        
        print(f"PASS: SLSA attestation triggered Slack notification")
        print(f"  - Build ID: {build_id}")
        print(f"  - Delivery status: {latest_slsa.get('status')}")


class TestVEXDocumentWithSlackNotification:
    """Test VEX document generation triggers Slack notification"""
    
    def test_vex_document_triggers_slack_notification(self):
        """VEX document generation triggers Slack notification"""
        # Get a completed build
        builds_response = requests.get(f"{BASE_URL}/api/builds")
        assert builds_response.status_code == 200
        
        builds = builds_response.json()
        completed_build = next((b for b in builds if b.get("status") == "completed"), None)
        assert completed_build is not None, "No completed build found"
        
        build_id = completed_build["id"]
        
        # Get initial delivery count
        initial_history = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=50").json()
        initial_vex_count = len([d for d in initial_history.get("deliveries", []) 
                                  if d.get("event_type") == "vex.document_generated"])
        
        # Generate VEX document
        vex_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex")
        assert vex_response.status_code == 200
        
        vex_data = vex_response.json()
        assert vex_data.get("@context") is not None
        
        # Wait a moment for webhook delivery
        time.sleep(0.5)
        
        # Check delivery history for VEX notification
        final_history = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=50").json()
        final_vex_count = len([d for d in final_history.get("deliveries", []) 
                                if d.get("event_type") == "vex.document_generated"])
        
        assert final_vex_count > initial_vex_count, "VEX document did not trigger Slack notification"
        
        # Verify the latest VEX delivery was successful
        vex_deliveries = [d for d in final_history.get("deliveries", []) 
                         if d.get("event_type") == "vex.document_generated"]
        latest_vex = vex_deliveries[0] if vex_deliveries else None
        
        assert latest_vex is not None
        assert latest_vex.get("status") == "success"
        assert latest_vex.get("response_code") == 200
        
        print(f"PASS: VEX document triggered Slack notification")
        print(f"  - Build ID: {build_id}")
        print(f"  - Delivery status: {latest_vex.get('status')}")


class TestSigstoreAvailability:
    """Test Sigstore library is installed and available"""
    
    def test_sigstore_library_installed(self):
        """Verify Sigstore library is installed"""
        try:
            import sigstore
            assert sigstore is not None
            print(f"PASS: Sigstore library is installed")
            print(f"  - Version: {sigstore.__version__ if hasattr(sigstore, '__version__') else 'unknown'}")
        except ImportError:
            pytest.fail("Sigstore library is not installed")
    
    def test_slsa_attestation_indicates_sigstore_available(self):
        """SLSA attestation indicates Sigstore availability"""
        # Get a completed build
        builds_response = requests.get(f"{BASE_URL}/api/builds")
        assert builds_response.status_code == 200
        
        builds = builds_response.json()
        completed_build = next((b for b in builds if b.get("status") == "completed"), None)
        assert completed_build is not None, "No completed build found"
        
        build_id = completed_build["id"]
        
        # Generate SLSA attestation
        slsa_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/slsa?level=3")
        assert slsa_response.status_code == 200
        
        slsa_data = slsa_response.json()
        provenance = slsa_data.get("provenance", {})
        signatures = provenance.get("signatures", [])
        
        assert len(signatures) >= 1, "No signatures found in attestation"
        
        signature = signatures[0]
        
        # Verify Sigstore availability is indicated
        assert signature.get("sigstore_available") is True, "Sigstore not marked as available"
        
        # Verify signing method mentions Sigstore
        signing_method = signature.get("signing_method", "")
        assert "Sigstore available" in signing_method or "sigstore" in signing_method.lower()
        
        print(f"PASS: SLSA attestation indicates Sigstore availability")
        print(f"  - sigstore_available: {signature.get('sigstore_available')}")
        print(f"  - signing_method: {signing_method}")


class TestWebhooksPageData:
    """Test data for Webhooks page UI"""
    
    def test_webhooks_page_displays_registered_webhook(self):
        """Webhooks page displays registered webhook with stats"""
        response = requests.get(f"{BASE_URL}/api/webhooks")
        assert response.status_code == 200
        
        data = response.json()
        webhooks = data.get("webhooks", [])
        stats = data.get("stats", {})
        
        # Verify webhook data for UI display
        assert len(webhooks) >= 1
        
        webhook = webhooks[0]
        assert "id" in webhook
        assert "name" in webhook
        assert "destination" in webhook
        assert "url" in webhook
        assert "events" in webhook
        assert "enabled" in webhook
        assert "delivery_count" in webhook
        assert "failure_count" in webhook
        
        # Verify stats for UI display
        assert "registered_webhooks" in stats
        assert "enabled_webhooks" in stats
        assert "total_deliveries" in stats
        assert "success_rate" in stats
        
        print(f"PASS: Webhooks page data is complete for UI display")
        print(f"  - Webhook: {webhook['name']}")
        print(f"  - Stats: {stats}")
    
    def test_recent_deliveries_section_data(self):
        """Recent Deliveries section shows successful webhook calls"""
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=20")
        assert response.status_code == 200
        
        data = response.json()
        deliveries = data.get("deliveries", [])
        
        assert len(deliveries) >= 1, "No deliveries for Recent Deliveries section"
        
        # Verify delivery data for UI display
        delivery = deliveries[0]
        assert "id" in delivery
        assert "webhook_id" in delivery
        assert "webhook_name" in delivery
        assert "event_type" in delivery
        assert "destination" in delivery
        assert "status" in delivery
        assert "response_code" in delivery
        assert "created_at" in delivery
        
        # Verify successful deliveries exist
        successful = [d for d in deliveries if d.get("status") == "success"]
        assert len(successful) >= 1, "No successful deliveries to display"
        
        print(f"PASS: Recent Deliveries section has data")
        print(f"  - Total deliveries: {len(deliveries)}")
        print(f"  - Successful: {len(successful)}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
