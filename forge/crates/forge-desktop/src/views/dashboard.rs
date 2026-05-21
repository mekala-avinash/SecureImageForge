use dioxus::prelude::*;

use crate::services::builds;
use crate::state::use_app_state;

#[component]
pub fn Dashboard() -> Element {
    let state = use_app_state();
    let mut tick = use_signal(|| 0u32);

    // Poll the repo every 2 seconds so the dashboard reflects in-flight builds.
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            tick += 1;
        }
    });

    let rows_resource = use_resource(move || {
        let repo = state.repo.clone();
        let _ = *tick.read();
        async move { builds::list_async(&repo, 10_000).await.unwrap_or_default() }
    });

    let rows = match &*rows_resource.read() {
        Some(r) => r.clone(),
        None => vec![],
    };
    let total = rows.len();
    let succeeded = rows.iter().filter(|r| r.status == "succeeded").count();
    let failed = rows.iter().filter(|r| r.status == "failed").count();
    let running = rows.iter().filter(|r| r.status == "running").count();
    let pending = rows.iter().filter(|r| r.status == "pending").count();

    rsx! {
        section {
            class: "view",
            header { class: "view-header", h1 { "Mission Control" } }
            div {
                class: "stats-grid",
                Tile { label: "Total Builds", value: total.to_string(),     tone: "neutral" }
                Tile { label: "Succeeded",    value: succeeded.to_string(), tone: "ok"      }
                Tile { label: "Failed",       value: failed.to_string(),    tone: "fail"    }
                Tile { label: "Active",       value: (running + pending).to_string(), tone: "warn" }
            }
            div {
                class: "glass-card",
                h2 { style: "margin-bottom: 20px;", "Temporal Activity Feed" }
                if rows.is_empty() {
                    p { class: "muted", "No active forges detected. Initialize a new build sequence." }
                } else {
                    table {
                        class: "data-table",
                        thead { tr {
                            th { "System ID" }
                            th { "Runtime Environment" }
                            th { "Integrity Status" }
                            th { "Timestamp" }
                        }}
                        tbody {
                            {rows.iter().take(10).map(|r| {
                                let id_short = &r.id[..8.min(r.id.len())];
                                rsx! {
                                    tr { 
                                        key: "{r.id}",
                                        td { 
                                            span { class: "mono", style: "color: var(--accent);", "{id_short}" }
                                            " "
                                            span { style: "font-weight: 600; margin-left: 8px;", "{r.name}" }
                                        }
                                        td { "{r.runtime}" }
                                        td { StatusBadge { status: r.status.clone() } }
                                        td { class: "muted", "{r.created_at}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Tile(label: String, value: String, tone: &'static str) -> Element {
    rsx! {
        div { class: "glass-card tile tile-{tone}",
            div { class: "tile-label", "{label}" }
            div { class: "tile-value", "{value}" }
        }
    }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
    let tone = match status.as_str() {
        "succeeded" => "ok",
        "failed" => "fail",
        "running" => "warn",
        _ => "neutral",
    };
    rsx! { span { class: "status-badge {tone}", "{status}" } }
}
