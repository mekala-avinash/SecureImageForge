"""Smoke test for the scaffolded paved-road FastAPI service."""
from fastapi.testclient import TestClient
from main import app

c = TestClient(app)


def test_healthz():
    assert c.get("/healthz").status_code == 200


def test_ready():
    assert c.get("/ready").json()["ok"] is True


def test_metrics_format():
    assert "http_requests_total" in c.get("/metrics").text


def test_list_items():
    r = c.get("/api/v1/items")
    assert r.status_code == 200
    assert "items" in r.json()
