//! Prometheus exporter installation. The recorder is global; install once per
//! process. The `/metrics` route reads the rendered output from this same
//! handle so we don't need to bind a second listener.

use std::sync::OnceLock;

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

pub fn install_recorder() {
    let _ = HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("install prometheus recorder")
    });
}

pub fn render() -> String {
    HANDLE
        .get()
        .map(|h| h.render())
        .unwrap_or_else(|| "# metrics recorder not installed\n".to_string())
}
