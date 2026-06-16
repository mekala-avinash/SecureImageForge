# Hardened .NET 9 runtime on Chainguard.
# syntax=docker/dockerfile:1.7
ARG DOTNET_VERSION=9.0

FROM mcr.microsoft.com/dotnet/sdk:${DOTNET_VERSION}-alpine AS builder
WORKDIR /src
COPY *.csproj ./
RUN dotnet restore --runtime linux-musl-x64
COPY . .
ARG VERSION=dev
RUN dotnet publish -c Release \
      --runtime linux-musl-x64 \
      --self-contained true \
      -p:PublishTrimmed=true \
      -p:PublishReadyToRun=true \
      -p:Version=${VERSION} \
      -o /out

FROM cgr.dev/chainguard/dotnet-runtime:${DOTNET_VERSION} AS runtime
ARG SERVICE_NAME=dotnet-service
ARG VERSION
LABEL org.opencontainers.image.title="${SERVICE_NAME}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.vendor="ACME"
WORKDIR /app
ENV ASPNETCORE_URLS=http://+:8080 \
    ASPNETCORE_ENVIRONMENT=Production \
    DOTNET_PRINT_TELEMETRY_MESSAGE=false \
    DOTNET_NOLOGO=true \
    DOTNET_gcServer=1 \
    TZ=UTC PORT=8080
COPY --from=builder --chown=65532:65532 /out ./
USER 65532:65532
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["dotnet", "App.dll"]
