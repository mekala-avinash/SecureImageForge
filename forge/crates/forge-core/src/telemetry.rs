//! Tracing initialization. Optional OpenTelemetry OTLP export is gated by
//! the `otlp` workspace feature; without it the daemon emits structured
//! tracing-subscriber output to stdout and the configured endpoint is just
//! announced for diagnostics.

use std::sync::Once;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static INIT: Once = Once::new();

/// Initialize tracing once. Idempotent — repeated calls are no-ops.
pub fn init() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_env("FORGE_LOG")
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("valid env filter");
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).compact())
            .init();
    });
}

/// Initialize tracing with optional OTLP export.
#[cfg(not(feature = "otlp"))]
pub fn init_with_endpoint(otlp_endpoint: Option<&str>, service_name: Option<&str>) {
    init();
    if let Some(endpoint) = otlp_endpoint {
        tracing::info!(
            otlp.endpoint = %endpoint,
            otlp.service = service_name.unwrap_or("forge"),
            "OTLP exporter requested but `otlp` feature is disabled at compile time"
        );
    }
}

/// Initialize tracing with optional OTLP export. When `otlp_endpoint` is
/// Some, the OTLP HTTP/protobuf exporter is installed and traces are
/// forwarded alongside the stdout layer.
#[cfg(feature = "otlp")]
pub fn init_with_endpoint(otlp_endpoint: Option<&str>, service_name: Option<&str>) {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_sdk::Resource;

    INIT.call_once(|| {
        let filter = EnvFilter::try_from_env("FORGE_LOG")
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("valid env filter");

        let stdout = fmt::layer().with_target(false).compact();

        if let Some(endpoint) = otlp_endpoint {
            let service = service_name.unwrap_or("forge").to_string();
            let exporter = match opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(endpoint)
                .build()
            {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("[telemetry] failed to build OTLP exporter: {err}");
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(stdout)
                        .init();
                    return;
                }
            };
            let provider = TracerProvider::builder()
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    service.clone(),
                )]))
                .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                .build();
            opentelemetry::global::set_tracer_provider(provider.clone());
            let tracer = provider.tracer(service);
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout)
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .init();
            tracing::info!(otlp.endpoint = %endpoint, "OTLP tracing exporter installed");
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(stdout)
                .init();
        }
    });
}
