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

    let _ = *tick.read();
    let rows = builds::list(&state.repo, 10_000).unwrap_or_default();
    let total = rows.len();
    let succeeded = rows.iter().filter(|r| r.status == "succeeded").count();
    let failed = rows.iter().filter(|r| r.status == "failed").count();
    let running = rows.iter().filter(|r| r.status == "running").count();
    let pending = rows.iter().filter(|r| r.status == "pending").count();

    rsx! {
        section {
            class: "view",
            header { class: "view-header", h1 { "Dashboard" } }
            div {
                class: "tiles",
                Tile { label: "Total",     value: total.to_string(),     tone: "neutral" }
                Tile { label: "Succeeded", value: succeeded.to_string(), tone: "ok"      }
                Tile { label: "Failed",    value: failed.to_string(),    tone: "fail"    }
                Tile { label: "Running",   value: running.to_string(),   tone: "warn"    }
                Tile { label: "Pending",   value: pending.to_string(),   tone: "neutral" }
            }
            div {
                class: "panel",
                h2 { "Recent activity" }
                if rows.is_empty() {
                    p { class: "muted", "No builds yet — click ‘New build’ to start one." }
                } else {
                    table {
                        class: "data-table",
                        thead { tr {
                            th { "Build" }
                            th { "Runtime" }
                            th { "Status" }
                            th { "Created" }
                        }}
                        tbody {
                            {rows.iter().take(10).map(|r| {
                                let id_short = &r.id[..8.min(r.id.len())];
                                rsx! {
                                    tr { key: "{r.id}",
                                        td { code { "{id_short}" } " " span { class: "build-name", "{r.name}" } }
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
        div { class: "tile tile-{tone}",
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
    rsx! { span { class: "badge badge-{tone}", "{status}" } }
}
