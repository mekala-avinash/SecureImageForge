# {{service_name}} — hardened Rust runtime (chainguard/static, ~6 MB).
# syntax=docker/dockerfile:1.7
ARG RUST_VERSION=1.83
FROM rust:${RUST_VERSION}-alpine AS builder
WORKDIR /src
RUN apk add --no-cache musl-dev openssl-dev pkgconf
ENV RUSTFLAGS="-C target-feature=+crt-static"
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
ARG VERSION=dev
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release && cp target/release/app /out-app

FROM cgr.dev/chainguard/static:latest AS runtime
ARG SERVICE_NAME={{service_name}}
ARG VERSION
LABEL org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="ACME"
COPY --from=builder --chown=65532:65532 /out-app /app
USER 65532:65532
EXPOSE 8080
ENV TZ=UTC PORT=8080 RUST_LOG=info
STOPSIGNAL SIGTERM
ENTRYPOINT ["/app"]
