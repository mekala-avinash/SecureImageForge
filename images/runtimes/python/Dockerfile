# syntax=docker/dockerfile:1.7
# =============================================================================
# Hardened Python 3.12 Runtime Image
# Base: Chainguard Wolfi (rolling, signed). Runtime stage is distroless.
# Target size: ≤ 75 MB.
#
# Usage in a service repo:
#   FROM registry.acme.io/runtime/python:3.12@sha256:DIGEST AS runtime
#   COPY --from=builder /app /app
#   ENTRYPOINT ["/usr/bin/python", "-m", "uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8080"]
#
# Or use this Dockerfile directly with your app (recommended).
# =============================================================================

ARG PYTHON_VERSION=3.12

# -----------------------------------------------------------------------------
# Stage 1 — builder. Has compilers + pip; produces a venv we can copy clean.
# -----------------------------------------------------------------------------
FROM cgr.dev/chainguard/python:${PYTHON_VERSION}-dev AS builder
WORKDIR /build

ENV PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PIP_REQUIRE_HASHES=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    VIRTUAL_ENV=/opt/venv

RUN python -m venv ${VIRTUAL_ENV}
ENV PATH="${VIRTUAL_ENV}/bin:${PATH}"

# Dependency layer — copy lockfile only so this layer caches well.
COPY requirements.txt requirements.lock ./
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-deps --require-hashes -r requirements.lock

# Application code.
COPY . .

# Pre-compile bytecode for deterministic startup.
ENV SOURCE_DATE_EPOCH=0
RUN python -m compileall -q -j 0 ${VIRTUAL_ENV} /build/app

# -----------------------------------------------------------------------------
# Stage 2 — runtime. No shell, no pip, no compilers, non-root.
# -----------------------------------------------------------------------------
FROM cgr.dev/chainguard/python:${PYTHON_VERSION} AS runtime

ARG BUILD_DATE
ARG VCS_REF
ARG VERSION
ARG SERVICE_NAME=python-service
LABEL org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.vendor="ACME" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.source="https://github.com/acme/${SERVICE_NAME}" \
      org.opencontainers.image.base.name="cgr.dev/chainguard/python:${PYTHON_VERSION}"

WORKDIR /app
ENV PATH="/opt/venv/bin:${PATH}" \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONHASHSEED=random \
    PORT=8080 \
    TZ=UTC

COPY --from=builder --chown=65532:65532 /opt/venv /opt/venv
COPY --from=builder --chown=65532:65532 /build/app /app/app

USER 65532:65532
EXPOSE 8080
STOPSIGNAL SIGTERM

# Self-healthcheck — uses stdlib so we don't add a dep.
HEALTHCHECK --interval=10s --timeout=2s --start-period=10s --retries=3 \
    CMD ["/opt/venv/bin/python", "-c", \
         "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:8080/healthz', timeout=1).status==200 else 1)"]

# Default entrypoint suits FastAPI/Starlette. Override per service.
ENTRYPOINT ["/opt/venv/bin/python", "-m", "uvicorn", "app.main:app", \
            "--host", "0.0.0.0", "--port", "8080", \
            "--workers", "2", "--proxy-headers", "--forwarded-allow-ips=*"]
