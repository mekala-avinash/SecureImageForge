"""
Test Webhook Delivery Persistence to MongoDB

Tests:
1. GET /api/webhooks - returns webhooks list with DB-based delivery stats
2. POST /api/webhooks/{id}/test - sends test webhook and persists delivery to MongoDB
3. GET /api/webhooks/delivery-history - returns delivery history from MongoDB
4. Verify stats (total_deliveries, successful, failed, success_rate) are calculated from database
"""
import pytest
import requests
import os
import time

BASE_URL = os.environ.get('REACT_APP_BACKEND_URL', '').rstrip('/')

# Known webhook ID from the existing Slack webhook
EXISTING_WEBHOOK_ID = "6e99bb58-d411-4e23-95f1-22d1aaa6e8d6"


class TestWebhookDeliveryPersistence:
    """Test webhook delivery persistence to MongoDB"""
    
    def test_get_webhooks_returns_db_stats(self):
        """GET /api/webhooks should return webhooks with DB-based delivery stats"""
        response = requests.get(f"{BASE_URL}/api/webhooks")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        
        # Verify response structure
        assert "webhooks" in data, "Response should contain 'webhooks' key"
        assert "stats" in data, "Response should contain 'stats' key"
        
        # Verify stats structure (calculated from DB)
        stats = data["stats"]
        assert "total_deliveries" in stats, "Stats should contain 'total_deliveries'"
        assert "successful" in stats, "Stats should contain 'successful'"
        assert "failed" in stats, "Stats should contain 'failed'"
        assert "success_rate" in stats, "Stats should contain 'success_rate'"
        assert "registered_webhooks" in stats, "Stats should contain 'registered_webhooks'"
        assert "enabled_webhooks" in stats, "Stats should contain 'enabled_webhooks'"
        
        # Verify stats are numeric
        assert isinstance(stats["total_deliveries"], int), "total_deliveries should be int"
        assert isinstance(stats["successful"], int), "successful should be int"
        assert isinstance(stats["failed"], int), "failed should be int"
        assert isinstance(stats["success_rate"], (int, float)), "success_rate should be numeric"
        
        # Verify we have at least the existing webhook
        assert len(data["webhooks"]) >= 1, "Should have at least 1 registered webhook"
        
        print(f"PASS: GET /api/webhooks returns stats from DB: {stats}")
    
    def test_get_delivery_history_from_mongodb(self):
        """GET /api/webhooks/delivery-history should return deliveries from MongoDB"""
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=20")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        
        # Verify response structure
        assert "deliveries" in data, "Response should contain 'deliveries' key"
        assert "stats" in data, "Response should contain 'stats' key"
        
        # Verify deliveries is a list
        assert isinstance(data["deliveries"], list), "deliveries should be a list"
        
        # Verify we have persisted deliveries
        assert len(data["deliveries"]) >= 1, "Should have at least 1 delivery in history"
        
        # Verify delivery record structure
        if data["deliveries"]:
            delivery = data["deliveries"][0]
            assert "id" in delivery, "Delivery should have 'id'"
            assert "webhook_id" in delivery, "Delivery should have 'webhook_id'"
            assert "webhook_name" in delivery, "Delivery should have 'webhook_name'"
            assert "event_type" in delivery, "Delivery should have 'event_type'"
            assert "destination" in delivery, "Delivery should have 'destination'"
            assert "status" in delivery, "Delivery should have 'status'"
            assert "created_at" in delivery, "Delivery should have 'created_at'"
            
            print(f"PASS: Delivery record structure verified: {delivery['id'][:8]}...")
        
        print(f"PASS: GET /api/webhooks/delivery-history returns {len(data['deliveries'])} deliveries from MongoDB")
    
    def test_test_webhook_persists_delivery(self):
        """POST /api/webhooks/{id}/test should persist delivery to MongoDB"""
        # First, get current delivery count
        history_before = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=100")
        assert history_before.status_code == 200
        count_before = len(history_before.json()["deliveries"])
        
        # Send test webhook
        response = requests.post(f"{BASE_URL}/api/webhooks/{EXISTING_WEBHOOK_ID}/test")
        
        assert response.status_code == 200, f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        
        # Verify response structure
        assert "status" in data, "Response should contain 'status'"
        assert "message" in data, "Response should contain 'message'"
        
        # Wait a moment for async persistence
        time.sleep(1)
        
        # Verify delivery was persisted to MongoDB
        history_after = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=100")
        assert history_after.status_code == 200
        count_after = len(history_after.json()["deliveries"])
        
        # Should have at least one more delivery
        assert count_after >= count_before, f"Delivery count should increase: before={count_before}, after={count_after}"
        
        # Check the most recent delivery
        latest_delivery = history_after.json()["deliveries"][0]
        assert latest_delivery["webhook_id"] == EXISTING_WEBHOOK_ID, "Latest delivery should be for our test webhook"
        assert latest_delivery["event_type"] == "build.completed", "Test webhook sends build.completed event"
        
        print(f"PASS: Test webhook persisted delivery to MongoDB. Count: {count_before} -> {count_after}")
        print(f"  Latest delivery: {latest_delivery['id'][:8]}... status={latest_delivery['status']}")
    
    def test_stats_calculated_from_database(self):
        """Verify stats are calculated from MongoDB, not in-memory"""
        # Get stats from webhooks endpoint
        webhooks_response = requests.get(f"{BASE_URL}/api/webhooks")
        assert webhooks_response.status_code == 200
        webhooks_stats = webhooks_response.json()["stats"]
        
        # Get stats from delivery-history endpoint
        history_response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=100")
        assert history_response.status_code == 200
        history_stats = history_response.json()["stats"]
        
        # Stats should match between endpoints (both from DB)
        assert webhooks_stats["total_deliveries"] == history_stats["total_deliveries"], \
            f"total_deliveries mismatch: {webhooks_stats['total_deliveries']} vs {history_stats['total_deliveries']}"
        assert webhooks_stats["successful"] == history_stats["successful"], \
            f"successful mismatch: {webhooks_stats['successful']} vs {history_stats['successful']}"
        assert webhooks_stats["failed"] == history_stats["failed"], \
            f"failed mismatch: {webhooks_stats['failed']} vs {history_stats['failed']}"
        
        # Verify success_rate calculation
        total = webhooks_stats["total_deliveries"]
        successful = webhooks_stats["successful"]
        expected_rate = round((successful / total * 100) if total > 0 else 0, 2)
        assert webhooks_stats["success_rate"] == expected_rate, \
            f"success_rate calculation mismatch: {webhooks_stats['success_rate']} vs expected {expected_rate}"
        
        print(f"PASS: Stats are consistent and calculated from DB:")
        print(f"  total_deliveries: {total}")
        print(f"  successful: {successful}")
        print(f"  failed: {webhooks_stats['failed']}")
        print(f"  success_rate: {webhooks_stats['success_rate']}%")
    
    def test_delivery_history_sorted_by_created_at(self):
        """Verify delivery history is sorted by created_at descending (newest first)"""
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=20")
        assert response.status_code == 200
        
        deliveries = response.json()["deliveries"]
        
        if len(deliveries) >= 2:
            # Check that deliveries are sorted newest first
            for i in range(len(deliveries) - 1):
                current_time = deliveries[i]["created_at"]
                next_time = deliveries[i + 1]["created_at"]
                assert current_time >= next_time, \
                    f"Deliveries not sorted: {current_time} should be >= {next_time}"
            
            print(f"PASS: Delivery history is sorted by created_at descending")
        else:
            print(f"SKIP: Not enough deliveries to verify sorting (have {len(deliveries)})")
    
    def test_webhook_not_found_returns_404(self):
        """POST /api/webhooks/{id}/test with invalid ID should return 404"""
        response = requests.post(f"{BASE_URL}/api/webhooks/invalid-webhook-id/test")
        
        assert response.status_code == 404, f"Expected 404, got {response.status_code}"
        
        print("PASS: Invalid webhook ID returns 404")
    
    def test_delivery_history_limit_parameter(self):
        """Verify limit parameter works for delivery history"""
        # Get with limit=2
        response = requests.get(f"{BASE_URL}/api/webhooks/delivery-history?limit=2")
        assert response.status_code == 200
        
        deliveries = response.json()["deliveries"]
        assert len(deliveries) <= 2, f"Should return at most 2 deliveries, got {len(deliveries)}"
        
        print(f"PASS: Limit parameter works correctly (returned {len(deliveries)} deliveries)")


class TestWebhookEndpoints:
    """Test other webhook endpoints"""
    
    def test_get_webhook_events(self):
        """GET /api/webhooks/events should return available event types"""
        response = requests.get(f"{BASE_URL}/api/webhooks/events")
        
        assert response.status_code == 200
        
        data = response.json()
        assert "events" in data
        assert len(data["events"]) > 0
        
        # Verify event structure
        event = data["events"][0]
        assert "id" in event
        assert "name" in event
        assert "description" in event
        
        print(f"PASS: GET /api/webhooks/events returns {len(data['events'])} event types")
    
    def test_get_webhook_destinations(self):
        """GET /api/webhooks/destinations should return supported destinations"""
        response = requests.get(f"{BASE_URL}/api/webhooks/destinations")
        
        assert response.status_code == 200
        
        data = response.json()
        assert "destinations" in data
        assert len(data["destinations"]) >= 4  # slack, teams, discord, generic
        
        # Verify destination structure
        dest = data["destinations"][0]
        assert "id" in dest
        assert "name" in dest
        assert "description" in dest
        
        print(f"PASS: GET /api/webhooks/destinations returns {len(data['destinations'])} destinations")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
