PAVEDROAD_ROOT ?= /app
SERVICE_NAME    = {{service_name}}
.DEFAULT_GOAL  := help

help: ## Show targets.
	@awk 'BEGIN {FS=":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

bootstrap: ## Fetch deps.
	cargo fetch

run: ## Run locally.
	OTEL_SERVICE_NAME=$(SERVICE_NAME) cargo run --release

test: ## Tests + clippy.
	cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings

lint: ## Clippy + Dockerfile lint.
	cargo clippy --all-targets --all-features -- -D warnings && hadolint Dockerfile

image-build: ## Build image.
	docker buildx build -t $(SERVICE_NAME):dev --load .

image-scan: ## Trivy + grype.
	trivy image $(SERVICE_NAME):dev && grype $(SERVICE_NAME):dev

sbom: ## SBOM (SPDX).
	syft $(SERVICE_NAME):dev -o spdx-json > sbom.spdx.json

.PHONY: help bootstrap run test lint image-build image-scan sbom
