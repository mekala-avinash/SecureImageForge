PAVEDROAD_ROOT ?= /app
SERVICE_NAME    = {{service_name}}
.DEFAULT_GOAL  := help

help: ## Show targets.
	@awk 'BEGIN {FS=":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

bootstrap: ## Install deps.
	npm ci --no-audit --no-fund

run: ## Run with auto-reload.
	OTEL_SERVICE_NAME=$(SERVICE_NAME) npm run dev

test: ## Vitest + coverage.
	npm test

lint: ## ESLint + Dockerfile lint.
	npm run lint

image-build: ## Build local image.
	docker buildx build -t $(SERVICE_NAME):dev --load .

image-scan: ## Trivy + grype.
	trivy image $(SERVICE_NAME):dev && grype $(SERVICE_NAME):dev

helm-lint: ## Render & validate chart.
	helm dependency update helm && helm template helm | kubeconform --strict

sbom: ## Generate SBOM (SPDX).
	syft $(SERVICE_NAME):dev -o spdx-json > sbom.spdx.json

.PHONY: help bootstrap run test lint image-build image-scan helm-lint sbom
