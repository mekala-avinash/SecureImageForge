use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize tracing once. Idempotent — repeated calls are no-ops.
pub fn init() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let filter = EnvFilter::try_from_env("FORGE_LOG")
            .or_else(|_| EnvFilter::try_new("info"))
            .expect("valid env filter");
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).compact())
            .init();
    });
}
