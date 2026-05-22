# Thin wrapper around the canonical hardened runtime; uses ARG-driven service name.
# syntax=docker/dockerfile:1.7
ARG PYTHON_VERSION=3.12

FROM cgr.dev/chainguard/python:${PYTHON_VERSION}-dev AS builder
WORKDIR /build
ENV PIP_NO_CACHE_DIR=1 VIRTUAL_ENV=/opt/venv
RUN python -m venv ${VIRTUAL_ENV}
ENV PATH=/opt/venv/bin:$PATH
COPY requirements.txt ./
RUN pip install -r requirements.txt
COPY src/ ./
ENV SOURCE_DATE_EPOCH=0
RUN python -m compileall -q -j 0 ${VIRTUAL_ENV} /build

FROM cgr.dev/chainguard/python:${PYTHON_VERSION} AS runtime
LABEL org.opencontainers.image.title="reference-service" \
      org.opencontainers.image.vendor="ACME"
WORKDIR /app
ENV PATH=/opt/venv/bin:$PATH PYTHONUNBUFFERED=1 PORT=8080 TZ=UTC
COPY --from=builder --chown=65532:65532 /opt/venv /opt/venv
COPY --from=builder --chown=65532:65532 /build/reference_service /app/reference_service
USER 65532:65532
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/opt/venv/bin/python", "-m", "uvicorn", "reference_service.main:app", "--host", "0.0.0.0", "--port", "8080"]
