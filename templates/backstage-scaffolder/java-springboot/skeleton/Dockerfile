# {{service_name}} — hardened JRE 21 (chainguard/jre).
# Two-stage: maven build + distroless JRE runtime.
# syntax=docker/dockerfile:1.7
ARG JAVA_VERSION=21

FROM cgr.dev/chainguard/maven:latest AS builder
WORKDIR /build
COPY pom.xml ./
RUN --mount=type=cache,target=/root/.m2 mvn -B -q dependency:go-offline
COPY src/ ./src/
RUN --mount=type=cache,target=/root/.m2 mvn -B -q -DskipTests package \
    && java -Djarmode=layertools -jar target/{{service_name}}.jar extract --destination /build/extracted

FROM cgr.dev/chainguard/jre:openjdk-${JAVA_VERSION} AS runtime
ARG SERVICE_NAME={{service_name}}
ARG VERSION
LABEL org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="ACME"
WORKDIR /app
ENV JAVA_TOOL_OPTIONS="-XX:MaxRAMPercentage=75 -XX:+ExitOnOutOfMemoryError -Djava.security.egd=file:/dev/./urandom" \
    PORT=8080 TZ=UTC
COPY --from=builder --chown=65532:65532 /build/extracted/dependencies/    ./
COPY --from=builder --chown=65532:65532 /build/extracted/spring-boot-loader/ ./
COPY --from=builder --chown=65532:65532 /build/extracted/snapshot-dependencies/ ./
COPY --from=builder --chown=65532:65532 /build/extracted/application/ ./
USER 65532:65532
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["java", "org.springframework.boot.loader.launch.JarLauncher"]
