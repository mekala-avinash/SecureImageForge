use dioxus::prelude::*;
use uuid::Uuid;

use crate::services::builds;
use crate::state::use_app_state;
use crate::views::dashboard::StatusBadge;

#[component]
pub fn BuildDetail(build_id: Uuid) -> Element {
    let state = use_app_state();
    let mut tick = use_signal(|| 0u32);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            tick += 1;
        }
    });
    let _ = *tick.read();

    let repo_summary = state.repo.clone();
    let summary_res = use_resource(move || {
        let repo = repo_summary.clone();
        let _ = *tick.read();
        async move { builds::summary_async(&repo, build_id).await.unwrap_or_default() }
    });
    let repo_scan = state.repo.clone();
    let scan_res = use_resource(move || {
        let repo = repo_scan.clone();
        let _ = *tick.read();
        async move { builds::scan_async(&repo, build_id).await.unwrap_or_default() }
    });
    let repo_sbom = state.repo.clone();
    let sbom_res = use_resource(move || {
        let repo = repo_sbom.clone();
        let _ = *tick.read();
        async move { builds::sbom_async(&repo, build_id).await.unwrap_or_default() }
    });

    let logs_store = state.logs.clone();
    let log_res = use_resource(move || {
        let logs = logs_store.clone();
        let _ = *tick.read();
        async move { builds::log_async(&logs, build_id).await.unwrap_or_default() }
    });

    let summary = match &*summary_res.read() {
        Some(s) => s.clone(),
        None => None,
    };
    let scan = match &*scan_res.read() {
        Some(s) => s.clone(),
        None => None,
    };
    let sbom = match &*sbom_res.read() {
        Some(s) => s.clone(),
        None => None,
    };
    let log = match &*log_res.read() {
        Some(Some(l)) => Some(l.clone()),
        _ => None,
    };

    rsx! {
        section {
            class: "view",
            header {
                class: "view-header",
                div {
                    style: "display: flex; align-items: baseline; gap: 16px;",
                    h1 { "Archive Inspection" }
                    span { class: "mono", style: "opacity: 0.5; font-size: 14px;", "{build_id}" }
                }
            }

            if let Some(s) = summary.as_ref() {
                div {
                    class: "glass-card",
                    div {
                        class: "meta-grid",
                        div {
                            class: "meta-item",
                            div { class: "meta-label", "Alias" }
                            div { class: "meta-value", "{s.name}" }
                        }
                        div {
                            class: "meta-item",
                            div { class: "meta-label", "Runtime" }
                            div { class: "meta-value", "{s.runtime}" }
                        }
                        div {
                            class: "meta-item",
                            div { class: "meta-label", "Base Image" }
                            div { class: "meta-value mono", style: "font-size: 12px;", "{s.base_image}" }
                        }
                        div {
                            class: "meta-item",
                            div { class: "meta-label", "Operational" }
                            div { class: "meta-value", StatusBadge { status: s.status.clone() } }
                        }
                        div {
                            class: "meta-item",
                            div { class: "meta-label", "Lifecycle" }
                            div { class: "meta-value", style: "font-size: 11px; opacity: 0.7;",
                                div { "Inception: {s.created_at}" }
                                if let Some(f) = s.finished_at.as_ref() {
                                    div { "Archived: {f}" }
                                }
                            }
                        }
                    }
                }
            } else {
                p { class: "muted", "Forge archives not found for this ID." }
            }

            div {
                class: "glass-card",
                h2 { style: "margin-bottom: 20px;", "Vulnerability Scan" }
                if let Some(scan) = scan.as_ref() {
                    if scan.findings.is_empty() {
                        p { class: "muted", "Zero vulnerabilities detected in current matrix." }
                    } else {
                        table {
                            class: "data-table",
                            thead { tr {
                                th { "Severity" }
                                th { "CVE ID" }
                                th { "Target Component" }
                                th { "Current" }
                                th { "Remediation" }
                            }}
                            tbody {
                                {scan.findings.iter().map(|f| {
                                    let sev = format!("{:?}", f.severity).to_uppercase();
                                    let fix = f.fixed_version.clone().unwrap_or_else(|| "—".into());
                                    rsx! {
                                        tr {
                                            td { span { class: "badge-sev-{sev.to_lowercase()}", "{sev}" } }
                                            td { span { class: "mono", style: "color: var(--accent);", "{f.id}" } }
                                            td { "{f.package}" }
                                            td { class: "mono", style: "font-size: 12px; opacity: 0.7;", "{f.installed_version}" }
                                            td { class: "mono", style: "font-weight: 600;", "{fix}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                } else {
                    p { class: "muted", "Scan analytics in progress..." }
                }
            }

            div {
                style: "display: grid; grid-template-columns: 1fr 2fr; gap: 32px;",
                div {
                    class: "glass-card",
                    h2 { style: "margin-bottom: 20px;", "SBOM Analysis" }
                    if let Some(b) = sbom.as_ref() {
                        div {
                            style: "display: flex; flex-direction: column; gap: 16px;",
                            div {
                                class: "meta-item",
                                div { class: "meta-label", "Specification" }
                                div { class: "meta-value mono", style: "color: var(--accent);", "{b.format}" }
                            }
                            div {
                                class: "meta-item",
                                div { class: "meta-label", "Component Density" }
                                div { class: "meta-value", style: "font-size: 32px; font-weight: 800;", {component_count(&b.document).to_string()} }
                            }
                        }
                    } else {
                        p { class: "muted", "BOM manifest not yet compiled." }
                    }
                }

                div {
                    class: "glass-card",
                    h2 { style: "margin-bottom: 20px;", "System Logs" }
                    if let Some(content) = log.as_ref() {
                        pre { 
                            class: "log",
                            style: "max-height: 300px;",
                            "{content}" 
                        }
                    } else {
                        p { class: "muted", "Awaiting stream from Forge kernel..." }
                    }
                }
            }
        }
    }
}

#[component]
fn Kv(k: &'static str, v: String) -> Element {
    rsx! {
        div { class: "kv",
            div { class: "kv-key", "{k}" }
            div { class: "kv-val", "{v}" }
        }
    }
}

fn component_count(doc: &serde_json::Value) -> usize {
    doc.get("components")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .or_else(|| {
            doc.get("packages")
                .and_then(|p| p.as_array())
                .map(|a| a.len())
        })
        .unwrap_or(0)
}
