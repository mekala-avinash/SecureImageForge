PAVEDROAD_ROOT ?= /app
SERVICE_NAME    = {{service_name}}
.DEFAULT_GOAL  := help

help: ## Show targets.
	@awk 'BEGIN {FS=":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

bootstrap: ## Tidy modules.
	go mod tidy

run: ## Run locally.
	OTEL_SERVICE_NAME=$(SERVICE_NAME) go run ./cmd/server

test: ## Unit tests + race detector.
	go test -race -coverprofile=coverage.out ./...

lint: ## golangci-lint + Dockerfile lint.
	golangci-lint run ./... && hadolint Dockerfile

image-build: ## Build local image.
	docker buildx build --platform linux/amd64 -t $(SERVICE_NAME):dev --load .

image-scan: ## Trivy + grype.
	trivy image $(SERVICE_NAME):dev && grype $(SERVICE_NAME):dev

helm-lint: ## Render & validate chart.
	helm dependency update helm && helm template helm | kubeconform --strict

sbom: ## Generate SBOM (SPDX).
	syft $(SERVICE_NAME):dev -o spdx-json > sbom.spdx.json

.PHONY: help bootstrap run test lint image-build image-scan helm-lint sbom
