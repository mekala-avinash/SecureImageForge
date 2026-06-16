"""Reference service exercising the full paved road.

Endpoints:
  GET  /healthz        — liveness
  GET  /ready          — readiness (DB stub OK)
  GET  /metrics        — Prometheus metrics
  GET  /api/v1/items   — example
  POST /api/v1/items   — example
"""
from __future__ import annotations

import json
import logging
import os
import sys
import time
from datetime import datetime, timezone
from typing import Any, Awaitable, Callable

from fastapi import FastAPI, HTTPException, Request, Response
from fastapi.responses import JSONResponse, PlainTextResponse
from prometheus_client import CONTENT_TYPE_LATEST, Counter, Histogram, generate_latest
from pydantic import BaseModel


# ── structured JSON logs with trace correlation ──────────────────────────────
class JsonFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord) -> str:
        out: dict[str, Any] = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "level": record.levelname.lower(),
            "service": os.environ.get("OTEL_SERVICE_NAME", "reference-service"),
            "msg": record.getMessage(),
        }
        try:
            from opentelemetry.trace import get_current_span
            ctx = get_current_span().get_span_context()
            if ctx and ctx.is_valid:
                out["trace_id"] = f"{ctx.trace_id:032x}"
                out["span_id"]  = f"{ctx.span_id:016x}"
        except Exception:
            pass
        return json.dumps(out)


h = logging.StreamHandler(sys.stdout)
h.setFormatter(JsonFormatter())
logging.basicConfig(level=os.environ.get("LOG_LEVEL", "INFO"), handlers=[h], force=True)
log = logging.getLogger("reference_service")

# ── app + metrics ────────────────────────────────────────────────────────────
app = FastAPI(title="reference-service", version=os.environ.get("APP_VERSION", "0.1.0"))

REQS: Counter = Counter("http_requests_total", "Total HTTP requests", ["method", "path", "code"])
LAT:  Histogram = Histogram("http_request_duration_seconds", "HTTP request duration", ["method", "path"])

ITEMS: dict[int, dict[str, Any]] = {1: {"id": 1, "name": "example"}}


@app.middleware("http")
async def _metrics_mw(request: Request, call_next: Callable[[Request], Awaitable[Response]]) -> Response:
    start = time.perf_counter()
    resp = await call_next(request)
    REQS.labels(request.method, request.url.path, str(resp.status_code)).inc()
    LAT.labels(request.method, request.url.path).observe(time.perf_counter() - start)
    return resp


@app.get("/healthz")
async def healthz() -> JSONResponse:
    return JSONResponse({"ok": True})


@app.get("/ready")
async def ready() -> JSONResponse:
    # Replace stub with real dependency checks (DB, Redis, …) in production.
    return JSONResponse({"ok": True, "deps": {"db": "ok", "cache": "ok"}})


@app.get("/metrics")
async def metrics() -> PlainTextResponse:
    return PlainTextResponse(generate_latest(), media_type=CONTENT_TYPE_LATEST)


class Item(BaseModel):
    id: int
    name: str


@app.get("/api/v1/items")
async def list_items() -> dict[str, list[dict[str, Any]]]:
    log.info("listing items")
    return {"items": list(ITEMS.values())}


@app.get("/api/v1/items/{item_id}")
async def get_item(item_id: int) -> dict[str, Any]:
    item = ITEMS.get(item_id)
    if not item:
        raise HTTPException(status_code=404, detail="not found")
    return item


@app.post("/api/v1/items", status_code=201)
async def create_item(item: Item) -> Item:
    ITEMS[item.id] = item.model_dump()
    log.info("created item %s", item.id)
    return item


# OTel manual init when not using the in-cluster Operator (e.g., local dev).
if os.environ.get("OTEL_SDK_INIT", "auto") == "manual":
    from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
    FastAPIInstrumentor.instrument_app(app)
