"""
P1 Features Backend Tests - SLSA, VEX, and Webhooks
Tests for:
- SLSA Level 3/4 provenance generation
- VEX document generation
- Webhook ChatOps integration (Slack/Teams/Discord)
"""
import pytest
import requests
import os
import time

BASE_URL = os.environ.get('REACT_APP_BACKEND_URL', '').rstrip('/')

# Get a valid build ID for testing
def get_test_build_id():
    """Get a completed build ID for testing"""
    response = requests.get(f"{BASE_URL}/api/builds")
    if response.status_code == 200:
        builds = response.json()
        for build in builds:
            if build.get('status') == 'completed':
                return build.get('id')
    return None


class TestSLSAEndpoints:
    """SLSA Attestation endpoint tests"""
    
    def test_get_slsa_levels(self):
        """Test GET /api/slsa/levels returns SLSA level information"""
        response = requests.get(f"{BASE_URL}/api/slsa/levels")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "levels" in data, "Response should contain 'levels'"
        
        # Levels is a dict with keys "1", "2", "3", "4"
        levels = data["levels"]
        assert isinstance(levels, dict), "Levels should be a dictionary"
        assert len(levels) >= 4, "Should have at least 4 SLSA levels"
        
        # Verify level structure
        for level_key, level_data in levels.items():
            assert "name" in level_data, f"Level {level_key} should have 'name' field"
            assert "description" in level_data, f"Level {level_key} should have 'description' field"
            assert "requirements" in level_data, f"Level {level_key} should have 'requirements' field"
        
        print(f"SUCCESS: GET /api/slsa/levels - Found {len(levels)} SLSA levels")
    
    def test_generate_slsa_attestation(self):
        """Test GET /api/builds/{id}/slsa generates SLSA Level 3 attestation"""
        build_id = get_test_build_id()
        if not build_id:
            pytest.skip("No completed build available for testing")
        
        response = requests.get(f"{BASE_URL}/api/builds/{build_id}/slsa?level=3")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        
        # Verify attestation bundle structure
        assert "bundle_id" in data, "Response should contain 'bundle_id'"
        assert "provenance" in data, "Response should contain 'provenance'"
        assert "verification" in data, "Response should contain 'verification'"
        assert "slsa_level" in data, "Response should contain 'slsa_level'"
        
        # Verify provenance structure (in-toto format)
        provenance = data["provenance"]
        assert provenance.get("_type") == "https://in-toto.io/Statement/v1", "Should be in-toto v1 format"
        assert "subject" in provenance, "Provenance should have 'subject'"
        assert "predicate" in provenance, "Provenance should have 'predicate'"
        assert "predicateType" in provenance, "Provenance should have 'predicateType'"
        
        # Verify verification result
        verification = data["verification"]
        assert "verified" in verification, "Verification should have 'verified' field"
        assert "trust_score" in verification, "Verification should have 'trust_score' field"
        
        print(f"SUCCESS: GET /api/builds/{build_id}/slsa - Generated SLSA Level {data['slsa_level']} attestation")
    
    def test_verify_slsa_provenance(self):
        """Test POST /api/builds/{id}/slsa/verify verifies SLSA provenance"""
        build_id = get_test_build_id()
        if not build_id:
            pytest.skip("No completed build available for testing")
        
        # First generate attestation
        gen_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/slsa?level=3")
        if gen_response.status_code != 200:
            pytest.skip("Could not generate SLSA attestation for verification test")
        
        # Now verify
        response = requests.post(f"{BASE_URL}/api/builds/{build_id}/slsa/verify")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        # Response has verification nested
        assert "verification" in data, "Response should contain 'verification'"
        assert "slsa_level" in data, "Response should contain 'slsa_level'"
        
        verification = data["verification"]
        assert "verified" in verification, "Verification should have 'verified'"
        assert "checks" in verification, "Verification should have 'checks'"
        
        print(f"SUCCESS: POST /api/builds/{build_id}/slsa/verify - Verified: {verification['verified']}")


class TestVEXEndpoints:
    """VEX Document endpoint tests"""
    
    def test_get_vex_formats(self):
        """Test GET /api/vex/formats returns supported VEX formats (openvex, csaf)"""
        response = requests.get(f"{BASE_URL}/api/vex/formats")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "formats" in data, "Response should contain 'formats'"
        
        formats = data["formats"]
        format_ids = [f["id"] for f in formats]
        
        # Verify openvex and csaf are supported
        assert "openvex" in format_ids, "Should support OpenVEX format"
        assert "csaf" in format_ids, "Should support CSAF format"
        
        print(f"SUCCESS: GET /api/vex/formats - Found formats: {format_ids}")
    
    def test_generate_vex_document(self):
        """Test GET /api/builds/{id}/vex generates VEX document with exploitability analysis"""
        build_id = get_test_build_id()
        if not build_id:
            pytest.skip("No completed build available for testing")
        
        response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        
        # Verify OpenVEX structure
        assert "@context" in data, "VEX should have '@context'"
        assert "@id" in data, "VEX should have '@id'"
        assert "statements" in data, "VEX should have 'statements'"
        assert "summary" in data, "VEX should have 'summary'"
        
        # Verify summary contains exploitability analysis
        summary = data["summary"]
        assert "total_vulnerabilities" in summary, "Summary should have 'total_vulnerabilities'"
        assert "not_affected" in summary, "Summary should have 'not_affected'"
        assert "affected" in summary, "Summary should have 'affected'"
        assert "false_positive_rate" in summary, "Summary should have 'false_positive_rate'"
        
        print(f"SUCCESS: GET /api/builds/{build_id}/vex - Generated VEX with {summary['total_vulnerabilities']} vulns, {summary['false_positive_rate']}% false positive rate")
    
    def test_get_vex_summary(self):
        """Test GET /api/builds/{id}/vex/summary returns VEX executive summary"""
        build_id = get_test_build_id()
        if not build_id:
            pytest.skip("No completed build available for testing")
        
        # First generate VEX document
        gen_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex")
        if gen_response.status_code != 200:
            pytest.skip("Could not generate VEX document for summary test")
        
        response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex/summary")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        # Summary is nested under 'summary' key
        assert "summary" in data, "Response should have 'summary'"
        
        summary = data["summary"]
        assert "exploitable" in summary, "Summary should have 'exploitable'"
        assert "non_exploitable" in summary, "Summary should have 'non_exploitable'"
        assert "false_positive_rate" in summary, "Summary should have 'false_positive_rate'"
        assert "recommendation" in summary, "Summary should have 'recommendation'"
        
        print(f"SUCCESS: GET /api/builds/{build_id}/vex/summary - Exploitable: {summary['exploitable']}, Non-exploitable: {summary['non_exploitable']}")


class TestWebhookEndpoints:
    """Webhook ChatOps endpoint tests"""
    
    def test_get_webhook_events(self):
        """Test GET /api/webhooks/events returns 17 event types"""
        response = requests.get(f"{BASE_URL}/api/webhooks/events")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "events" in data, "Response should contain 'events'"
        
        events = data["events"]
        assert len(events) >= 17, f"Should have at least 17 event types, got {len(events)}"
        
        # Verify event structure
        for event in events:
            assert "id" in event, "Each event should have 'id'"
            assert "name" in event, "Each event should have 'name'"
        
        # Check for key event types
        event_ids = [e["id"] for e in events]
        expected_events = [
            "build.started", "build.completed", "build.failed",
            "vulnerability.critical", "slsa.attestation_generated", "vex.document_generated"
        ]
        for expected in expected_events:
            assert expected in event_ids, f"Should have event type: {expected}"
        
        print(f"SUCCESS: GET /api/webhooks/events - Found {len(events)} event types")
    
    def test_get_webhook_destinations(self):
        """Test GET /api/webhooks/destinations returns Slack, Teams, Discord, Generic"""
        response = requests.get(f"{BASE_URL}/api/webhooks/destinations")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "destinations" in data, "Response should contain 'destinations'"
        
        destinations = data["destinations"]
        dest_ids = [d["id"] for d in destinations]
        
        # Verify all 4 destinations
        expected_dests = ["slack", "microsoft_teams", "discord", "generic_webhook"]
        for expected in expected_dests:
            assert expected in dest_ids, f"Should have destination: {expected}"
        
        print(f"SUCCESS: GET /api/webhooks/destinations - Found destinations: {dest_ids}")
    
    def test_list_webhooks(self):
        """Test GET /api/webhooks returns webhook list and stats"""
        response = requests.get(f"{BASE_URL}/api/webhooks")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "webhooks" in data, "Response should contain 'webhooks'"
        assert "stats" in data, "Response should contain 'stats'"
        
        # Verify stats structure
        stats = data["stats"]
        assert "registered_webhooks" in stats, "Stats should have 'registered_webhooks'"
        assert "enabled_webhooks" in stats, "Stats should have 'enabled_webhooks'"
        assert "total_deliveries" in stats, "Stats should have 'total_deliveries'"
        assert "success_rate" in stats, "Stats should have 'success_rate'"
        
        print(f"SUCCESS: GET /api/webhooks - {stats['registered_webhooks']} webhooks, {stats['success_rate']}% success rate")
    
    def test_create_webhook(self):
        """Test POST /api/webhooks creates new webhook"""
        webhook_data = {
            "name": "TEST_P1_Webhook",
            "destination": "slack",
            "url": "https://hooks.slack.com/services/TEST/TEST/TEST",
            "events": ["build.completed", "vulnerability.critical"],
            "channel": "#test-alerts",
            "enabled": True
        }
        
        response = requests.post(f"{BASE_URL}/api/webhooks", json=webhook_data)
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        assert "id" in data, "Response should contain 'id'"
        # Message confirms registration
        assert "message" in data, "Response should contain 'message'"
        assert "registered" in data.get("message", "").lower() or "created" in data.get("message", "").lower(), "Message should confirm creation"
        
        webhook_id = data["id"]
        print(f"SUCCESS: POST /api/webhooks - Created webhook: {webhook_id}")
        
        # Cleanup
        requests.delete(f"{BASE_URL}/api/webhooks/{webhook_id}")
        
        return webhook_id
    
    def test_test_webhook(self):
        """Test POST /api/webhooks/{id}/test sends test notification"""
        # First create a webhook
        webhook_data = {
            "name": "TEST_P1_TestWebhook",
            "destination": "generic_webhook",
            "url": "https://httpbin.org/post",
            "events": ["build.completed"],
            "enabled": True
        }
        
        create_response = requests.post(f"{BASE_URL}/api/webhooks", json=webhook_data)
        if create_response.status_code != 200:
            pytest.skip("Could not create webhook for test")
        
        webhook_id = create_response.json()["id"]
        
        # Test the webhook
        response = requests.post(f"{BASE_URL}/api/webhooks/{webhook_id}/test")
        
        # Note: Test may return queued status if external URL is unreachable
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        assert "status" in data, "Response should contain 'status'"
        # Status can be success, queued, or failed
        assert data["status"] in ["success", "queued", "failed"], f"Status should be valid, got: {data['status']}"
        
        print(f"SUCCESS: POST /api/webhooks/{webhook_id}/test - Status: {data['status']}")
        
        # Cleanup
        requests.delete(f"{BASE_URL}/api/webhooks/{webhook_id}")
    
    def test_delete_webhook(self):
        """Test DELETE /api/webhooks/{id} removes webhook"""
        # First create a webhook
        webhook_data = {
            "name": "TEST_P1_DeleteWebhook",
            "destination": "slack",
            "url": "https://hooks.slack.com/services/DELETE/TEST/TEST",
            "events": ["build.completed"],
            "enabled": True
        }
        
        create_response = requests.post(f"{BASE_URL}/api/webhooks", json=webhook_data)
        if create_response.status_code != 200:
            pytest.skip("Could not create webhook for delete test")
        
        webhook_id = create_response.json()["id"]
        
        # Delete the webhook
        response = requests.delete(f"{BASE_URL}/api/webhooks/{webhook_id}")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        assert data.get("message") == "Webhook deleted successfully", "Should confirm deletion"
        
        print(f"SUCCESS: DELETE /api/webhooks/{webhook_id} - Webhook deleted")
    
    def test_get_delivery_history(self):
        """Test GET /api/webhooks/delivery-history returns delivery history"""
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=20")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
        
        data = response.json()
        assert "deliveries" in data, "Response should contain 'deliveries'"
        
        # Verify delivery structure if any exist
        if data["deliveries"]:
            delivery = data["deliveries"][0]
            assert "event_type" in delivery, "Delivery should have 'event_type'"
            assert "status" in delivery, "Delivery should have 'status'"
        
        print(f"SUCCESS: GET /api/webhooks/delivery-history - Found {len(data['deliveries'])} deliveries")


class TestIntegration:
    """Integration tests for P1 features"""
    
    def test_slsa_vex_workflow(self):
        """Test complete SLSA + VEX workflow for a build"""
        build_id = get_test_build_id()
        if not build_id:
            pytest.skip("No completed build available for testing")
        
        # 1. Generate SLSA attestation
        slsa_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/slsa?level=3")
        assert slsa_response.status_code == 200, "SLSA generation should succeed"
        slsa_data = slsa_response.json()
        
        # 2. Verify SLSA
        verify_response = requests.post(f"{BASE_URL}/api/builds/{build_id}/slsa/verify")
        assert verify_response.status_code == 200, "SLSA verification should succeed"
        
        # 3. Generate VEX document
        vex_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex")
        assert vex_response.status_code == 200, "VEX generation should succeed"
        vex_data = vex_response.json()
        
        # 4. Get VEX summary
        summary_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/vex/summary")
        assert summary_response.status_code == 200, "VEX summary should succeed"
        
        print(f"SUCCESS: Complete SLSA+VEX workflow for build {build_id}")
        print(f"  - SLSA Level: {slsa_data['slsa_level']}")
        print(f"  - VEX False Positive Rate: {vex_data['summary']['false_positive_rate']}%")


# Cleanup test webhooks
@pytest.fixture(scope="module", autouse=True)
def cleanup_test_webhooks():
    """Cleanup TEST_ prefixed webhooks after tests"""
    yield
    # Cleanup
    response = requests.get(f"{BASE_URL}/api/webhooks")
    if response.status_code == 200:
        webhooks = response.json().get("webhooks", [])
        for webhook in webhooks:
            if webhook.get("name", "").startswith("TEST_P1_"):
                requests.delete(f"{BASE_URL}/api/webhooks/{webhook['id']}")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
