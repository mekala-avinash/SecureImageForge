# syntax=docker/dockerfile:1.7
# Hardened Node.js 20 runtime — Chainguard Wolfi, ~80 MB target.
ARG NODE_VERSION=20

FROM cgr.dev/chainguard/node:${NODE_VERSION}-dev AS deps
WORKDIR /build
COPY package.json yarn.lock ./
RUN --mount=type=cache,target=/root/.yarn/cache \
    yarn install --frozen-lockfile --production=false

FROM deps AS build
COPY . .
ENV NODE_ENV=production
RUN yarn build && yarn install --frozen-lockfile --production=true && npm prune --omit=dev

FROM cgr.dev/chainguard/node:${NODE_VERSION} AS runtime
ARG SERVICE_NAME=node-service
ARG VERSION
LABEL org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="ACME"
WORKDIR /app
ENV NODE_ENV=production NODE_OPTIONS="--enable-source-maps --max-old-space-size=384" PORT=8080 TZ=UTC
COPY --from=build --chown=65532:65532 /build/node_modules ./node_modules
COPY --from=build --chown=65532:65532 /build/dist ./dist
COPY --from=build --chown=65532:65532 /build/package.json ./
USER 65532:65532
EXPOSE 8080
STOPSIGNAL SIGTERM
HEALTHCHECK --interval=10s --timeout=2s --start-period=10s --retries=3 \
    CMD ["/usr/bin/node", "-e", "fetch('http://127.0.0.1:8080/healthz').then(r=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))"]
ENTRYPOINT ["/usr/bin/node", "dist/server.js"]
