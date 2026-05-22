# {{service_name}} — hardened Go runtime (chainguard/static, ~12 MB).
# Thin wrapper around /app/images/runtimes/go/Dockerfile.
# syntax=docker/dockerfile:1.7
ARG GO_VERSION=1.23

FROM golang:${GO_VERSION}-alpine AS builder
WORKDIR /src
ENV CGO_ENABLED=0 GOFLAGS="-trimpath -mod=readonly" GOOS=linux
RUN apk add --no-cache ca-certificates git tzdata
COPY go.mod go.sum ./
RUN --mount=type=cache,target=/root/.cache/go-build \
    --mount=type=cache,target=/go/pkg/mod \
    go mod download -x
COPY . .
ARG VERSION=dev
ARG VCS_REF=unknown
ARG BUILD_DATE
ARG TARGETARCH
RUN --mount=type=cache,target=/root/.cache/go-build \
    --mount=type=cache,target=/go/pkg/mod \
    GOARCH=${TARGETARCH} go build \
      -ldflags="-s -w -buildid= -extldflags=-static \
        -X main.version=${VERSION} -X main.commit=${VCS_REF} -X main.date=${BUILD_DATE}" \
      -tags="netgo,osusergo,static_build" \
      -o /out/app ./cmd/server

FROM cgr.dev/chainguard/static:latest AS runtime
ARG SERVICE_NAME={{service_name}}
ARG VERSION
LABEL org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="ACME"
COPY --from=builder --chown=65532:65532 /out/app /app
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
USER 65532:65532
EXPOSE 8080
ENV TZ=UTC PORT=8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/app"]
