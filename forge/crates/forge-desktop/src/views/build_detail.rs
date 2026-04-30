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

    let summary = builds::summary(&state.repo, build_id).unwrap_or_default();
    let scan = builds::scan(&state.repo, build_id).unwrap_or_default();
    let sbom = builds::sbom(&state.repo, build_id).unwrap_or_default();
    let log = builds::log(&state.logs, build_id).unwrap_or_default();

    rsx! {
        section {
            class: "view",
            header {
                class: "view-header",
                h1 { "Build" }
                code { class: "muted", "{build_id}" }
            }

            if let Some(s) = summary.as_ref() {
                div {
                    class: "panel",
                    div {
                        class: "kv-grid",
                        Kv { k: "Name",    v: s.name.clone() }
                        Kv { k: "Runtime", v: s.runtime.clone() }
                        Kv { k: "Base",    v: s.base_image.clone() }
                        Kv { k: "Created", v: s.created_at.clone() }
                        Kv { k: "Finished", v: s.finished_at.clone().unwrap_or_else(|| "—".into()) }
                    }
                    div { class: "row", StatusBadge { status: s.status.clone() } }
                }
            } else {
                p { class: "muted", "Build not found." }
            }

            div {
                class: "panel",
                h2 { "Vulnerabilities" }
                if let Some(scan) = scan.as_ref() {
                    if scan.findings.is_empty() {
                        p { class: "muted", "No findings." }
                    } else {
                        table {
                            class: "data-table",
                            thead { tr {
                                th { "Severity" }
                                th { "ID" }
                                th { "Package" }
                                th { "Installed" }
                                th { "Fixed" }
                            }}
                            tbody {
                                {scan.findings.iter().map(|f| {
                                    let sev = format!("{:?}", f.severity).to_uppercase();
                                    let fix = f.fixed_version.clone().unwrap_or_else(|| "—".into());
                                    rsx! {
                                        tr {
                                            td { span { class: "badge badge-sev-{sev.to_lowercase()}", "{sev}" } }
                                            td { code { "{f.id}" } }
                                            td { "{f.package}" }
                                            td { class: "muted", "{f.installed_version}" }
                                            td { class: "muted", "{fix}" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                } else {
                    p { class: "muted", "Scan not yet recorded." }
                }
            }

            div {
                class: "panel",
                h2 { "SBOM" }
                if let Some(b) = sbom.as_ref() {
                    p {
                        "Format: "
                        code { "{b.format}" }
                    }
                    p {
                        "Components: "
                        code { {component_count(&b.document).to_string()} }
                    }
                } else {
                    p { class: "muted", "No SBOM recorded for this build." }
                }
            }

            div {
                class: "panel",
                h2 { "Build log" }
                if let Some(content) = log.as_ref() {
                    pre { class: "log", "{content}" }
                } else {
                    p { class: "muted", "No log captured yet." }
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
