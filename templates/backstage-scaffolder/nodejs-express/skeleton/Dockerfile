# {{service_name}} — hardened Node 20 runtime. Wraps /app/images/runtimes/nodejs/Dockerfile.
# syntax=docker/dockerfile:1.7
ARG NODE_VERSION=20

FROM cgr.dev/chainguard/node:${NODE_VERSION}-dev AS deps
WORKDIR /build
COPY package.json yarn.lock* package-lock.json* ./
RUN --mount=type=cache,target=/root/.npm npm ci --no-audit --no-fund

FROM deps AS build
COPY . .
ENV NODE_ENV=production
RUN npm run build && npm prune --omit=dev

FROM cgr.dev/chainguard/node:${NODE_VERSION} AS runtime
ARG SERVICE_NAME={{service_name}}
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
ENTRYPOINT ["/usr/bin/node", "dist/server.js"]
