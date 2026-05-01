use dioxus::prelude::*;
use uuid::Uuid;

use crate::services::builds;
use crate::state::use_app_state;
use crate::views::dashboard::StatusBadge;
use crate::views::Route;

#[component]
pub fn BuildsList(route: Signal<Route>) -> Element {
    let state = use_app_state();
    let mut tick = use_signal(|| 0u32);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            tick += 1;
        }
    });

    let rows_resource = use_resource(move || {
        let repo = state.repo.clone();
        let _ = *tick.read();
        async move { builds::list_async(&repo, 200).await.unwrap_or_default() }
    });

    let rows = match &*rows_resource.read() {
        Some(r) => r.clone(),
        None => vec![],
    };

    rsx! {
        section {
            class: "view",
            header {
                class: "view-header",
                h1 { "Build History" }
                button {
                    class: "nav-item active",
                    style: "border: 0; padding: 10px 20px; border-radius: 8px;",
                    onclick: move |_| route.set(Route::NewBuild),
                    "+ Initialize New Forge"
                }
            }
            div {
                class: "glass-card",
                if rows.is_empty() {
                    p { class: "muted", "No build logs found in the archives." }
                } else {
                    table {
                        style: "width: 100%; border-collapse: collapse;",
                        thead { tr {
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Protocol ID" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Alias" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Env" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Base Matrix" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Integrity" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Timestamp" }
                            th { style: "text-align: left; padding: 12px; color: var(--muted); font-size: 11px; text-transform: uppercase;", "Control" }
                        }}
                        tbody {
                            {rows.iter().map(|r| {
                                let id_full = r.id.clone();
                                let id_short = id_full[..8.min(id_full.len())].to_string();
                                let parsed = Uuid::parse_str(&id_full).ok();
                                rsx! {
                                    tr { 
                                        key: "{id_full}",
                                        style: "border-top: 1px solid var(--rule);",
                                        td { style: "padding: 16px 12px;", span { class: "mono", style: "color: var(--accent);", "{id_short}" } }
                                        td { style: "padding: 16px 12px; font-weight: 600;", "{r.name}" }
                                        td { style: "padding: 16px 12px;", "{r.runtime}" }
                                        td { style: "padding: 16px 12px; opacity: 0.7; font-size: 12px;", "{r.base_image}" }
                                        td { style: "padding: 16px 12px;", StatusBadge { status: r.status.clone() } }
                                        td { style: "padding: 16px 12px; color: var(--muted);", "{r.created_at}" }
                                        td { style: "padding: 16px 12px;",
                                            if let Some(uuid) = parsed {
                                                button {
                                                    class: "nav-item",
                                                    style: "padding: 6px 12px; font-size: 12px;",
                                                    onclick: move |_| route.set(Route::Build(uuid)),
                                                    "Inspect"
                                                }
                                            }
                                        }
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
