//! Domain metric helpers built on top of the `metrics` facade. Counters and
//! histograms are registered lazily on first use; the API crate installs a
//! `metrics-exporter-prometheus` recorder and exposes `/metrics`.

use metrics::{counter, histogram};

pub const BUILD_STARTED: &str = "forge_builds_started_total";
pub const BUILD_SUCCEEDED: &str = "forge_builds_succeeded_total";
pub const BUILD_FAILED: &str = "forge_builds_failed_total";
pub const POLICY_DENIED: &str = "forge_policy_denied_total";
pub const SCAN_DURATION: &str = "forge_scan_duration_seconds";
pub const DRIFT_NEW_CRITICAL: &str = "forge_drift_new_critical_total";
pub const DRIFT_NEW_HIGH: &str = "forge_drift_new_high_total";

pub fn record_build_started(runtime: &str) {
    counter!(BUILD_STARTED, "runtime" => runtime.to_string()).increment(1);
}

pub fn record_build_succeeded(runtime: &str) {
    counter!(BUILD_SUCCEEDED, "runtime" => runtime.to_string()).increment(1);
}

pub fn record_build_failed(runtime: &str, reason: &str) {
    counter!(BUILD_FAILED, "runtime" => runtime.to_string(), "reason" => reason.to_string())
        .increment(1);
}

pub fn record_policy_denied(profile: &str) {
    counter!(POLICY_DENIED, "profile" => profile.to_string()).increment(1);
}

pub fn record_scan_duration(scanner: &str, seconds: f64) {
    histogram!(SCAN_DURATION, "scanner" => scanner.to_string()).record(seconds);
}

pub fn record_drift_delta(new_critical: u64, new_high: u64) {
    if new_critical > 0 {
        counter!(DRIFT_NEW_CRITICAL).increment(new_critical);
    }
    if new_high > 0 {
        counter!(DRIFT_NEW_HIGH).increment(new_high);
    }
}
