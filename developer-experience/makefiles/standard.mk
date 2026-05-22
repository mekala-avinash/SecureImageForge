# Standard Makefile fragments. Service repos `include` from here.
# All targets are idempotent.
.DEFAULT_GOAL := help

# ─────────────────────────────────────────────────────────────────────────────
# Service-level defaults. Override in the service's Makefile.
SERVICE_NAME       ?= $(shell basename $(CURDIR))
IMAGE_REPO         ?= registry.acme.io/$(SERVICE_NAME)
IMAGE_TAG          ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo dev)
PLATFORMS          ?= linux/amd64,linux/arm64
PYTHON             ?= python3
# ─────────────────────────────────────────────────────────────────────────────

help:  ## Show this help.
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

bootstrap:  ## Install dependencies.
	@$(PYTHON) -m pip install --require-hashes -r requirements.lock || $(PYTHON) -m pip install -r requirements.txt

dev:  ## Start local dev dependencies (Postgres/Redis/OTel/Jaeger).
	docker compose -f $(PAVEDROAD_ROOT)/developer-experience/docker-compose/docker-compose.yml up -d
	@echo "Jaeger UI:  http://localhost:16686"
	@echo "Prometheus: http://localhost:9090"

run:  ## Run the service with auto-reload.
	OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
	OTEL_SERVICE_NAME=$(SERVICE_NAME) \
	uvicorn app.main:app --host 0.0.0.0 --port 8080 --reload

test:  ## Run unit tests.
	pytest -q

lint:  ## Lint code + manifests.
	ruff check . && hadolint Dockerfile && helm template helm | kubeconform --strict

image-build:  ## Build a multi-arch image and load locally (amd64).
	docker buildx build --platform linux/amd64 -t $(IMAGE_REPO):$(IMAGE_TAG) --load .

image-scan:  ## Trivy-scan the local image.
	trivy image $(IMAGE_REPO):$(IMAGE_TAG)

image-publish:  ## Multi-arch push + sign + attest. Used by CI; works locally too if creds.
	IMAGE_REPO=$(IMAGE_REPO) IMAGE_TAG=$(IMAGE_TAG) PLATFORMS=$(PLATFORMS) \
	$(PAVEDROAD_ROOT)/images/buildkit/build.sh

helm-lint:  ## Render and validate the chart.
	helm dependency update helm && helm template helm | kubeconform --strict

down:  ## Stop local dev dependencies.
	docker compose -f $(PAVEDROAD_ROOT)/developer-experience/docker-compose/docker-compose.yml down -v

image-tag:  ## Print the resolved image:tag.
	@echo $(IMAGE_REPO):$(IMAGE_TAG)

.PHONY: help bootstrap dev run test lint image-build image-scan image-publish helm-lint down image-tag
