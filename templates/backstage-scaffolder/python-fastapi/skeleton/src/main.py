"""Paved-road FastAPI entrypoint.

Wired in for every {{language}} service scaffolded via `pavedroad new service`.
Includes:
  - /healthz, /ready, /metrics (Prometheus + OTel-friendly)
  - structured JSON logs with trace correlation
  - OTel auto-instrumentation hook (no-op if the Operator already injected the SDK)
"""
from __future__ import annotations

import json
import logging
import os
import sys
import time
from datetime import datetime, timezone
from typing import Any, Awaitable, Callable

from fastapi import FastAPI, Request, Response
from fastapi.responses import JSONResponse, PlainTextResponse
from prometheus_client import CONTENT_TYPE_LATEST, Counter, Histogram, generate_latest


# ── structured JSON logging with trace correlation ────────────────────────────
class JsonFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, Any] = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "level": record.levelname.lower(),
            "service": os.environ.get("OTEL_SERVICE_NAME", "{{service_name}}"),
            "msg": record.getMessage(),
        }
        # Propagate trace context if OTel SDK is loaded.
        try:
            from opentelemetry.trace import get_current_span
            ctx = get_current_span().get_span_context()
            if ctx and ctx.is_valid:
                payload["trace_id"] = f"{ctx.trace_id:032x}"
                payload["span_id"]  = f"{ctx.span_id:016x}"
        except Exception:
            pass
        return json.dumps(payload)


_handler = logging.StreamHandler(sys.stdout)
_handler.setFormatter(JsonFormatter())
logging.basicConfig(level=os.environ.get("LOG_LEVEL", "INFO"), handlers=[_handler], force=True)
log = logging.getLogger(__name__)

# ── app + metrics ────────────────────────────────────────────────────────────
app = FastAPI(title="{{service_name}}", version="0.1.0")

REQUESTS: Counter = Counter("http_requests_total", "Total HTTP requests", ["method", "path", "code"])
LATENCY:  Histogram = Histogram("http_request_duration_seconds", "HTTP request duration", ["method", "path"])


@app.middleware("http")
async def metrics_middleware(request: Request, call_next: Callable[[Request], Awaitable[Response]]) -> Response:
    start = time.perf_counter()
    response = await call_next(request)
    elapsed = time.perf_counter() - start
    REQUESTS.labels(request.method, request.url.path, str(response.status_code)).inc()
    LATENCY.labels(request.method, request.url.path).observe(elapsed)
    return response


@app.get("/healthz")
async def healthz() -> JSONResponse:
    return JSONResponse({"ok": True})


@app.get("/ready")
async def ready() -> JSONResponse:
    # Extend with real dependency checks (DB, Redis) in production.
    return JSONResponse({"ok": True})


@app.get("/metrics")
async def metrics() -> PlainTextResponse:
    return PlainTextResponse(generate_latest(), media_type=CONTENT_TYPE_LATEST)


@app.get("/api/v1/items")
async def list_items() -> dict[str, list[dict[str, Any]]]:
    log.info("listing items")
    return {"items": [{"id": 1, "name": "example"}]}


# ── optional OTel manual init when not using the Operator ────────────────────
if os.environ.get("OTEL_SDK_INIT") == "manual":
    from opentelemetry.instrumentation.fastapi import FastAPIInstrumentor
    FastAPIInstrumentor.instrument_app(app)
