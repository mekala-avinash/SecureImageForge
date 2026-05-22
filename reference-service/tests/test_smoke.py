"""Smoke test — verifies endpoints render without errors."""
from fastapi.testclient import TestClient
from reference_service.main import app

c = TestClient(app)

def test_healthz():       assert c.get("/healthz").status_code == 200
def test_ready():         assert c.get("/ready").json()["ok"] is True
def test_metrics_format(): assert "http_requests_total" in c.get("/metrics").text
def test_items_crud():
    assert c.get("/api/v1/items").status_code == 200
    assert c.post("/api/v1/items", json={"id": 2, "name": "two"}).status_code == 201
    assert c.get("/api/v1/items/2").json()["name"] == "two"
    assert c.get("/api/v1/items/999").status_code == 404
