PAVEDROAD_ROOT ?= /app
SERVICE_NAME    = {{service_name}}
.DEFAULT_GOAL  := help

help: ## Show targets.
	@awk 'BEGIN {FS=":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

bootstrap: ## Restore packages.
	dotnet restore

run: ## Run locally.
	OTEL_SERVICE_NAME=$(SERVICE_NAME) dotnet run --no-launch-profile

test: ## Tests.
	dotnet test --logger trx --collect:"XPlat Code Coverage"

lint: ## Code analysis + Dockerfile lint.
	dotnet format --verify-no-changes && hadolint Dockerfile

image-build: ## Build image.
	docker buildx build -t $(SERVICE_NAME):dev --load .

image-scan: ## Trivy + grype.
	trivy image $(SERVICE_NAME):dev && grype $(SERVICE_NAME):dev

sbom: ## SBOM (SPDX).
	syft $(SERVICE_NAME):dev -o spdx-json > sbom.spdx.json

.PHONY: help bootstrap run test lint image-build image-scan sbom
