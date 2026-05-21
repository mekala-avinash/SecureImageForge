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
                    class: "btn-primary",
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
                        class: "data-table",
                        thead { tr {
                            th { "Protocol ID" }
                            th { "Alias" }
                            th { "Env" }
                            th { "Base Matrix" }
                            th { "Integrity" }
                            th { "Timestamp" }
                            th { "Control" }
                        }}
                        tbody {
                            {rows.iter().map(|r| {
                                let id_full = r.id.clone();
                                let id_short = id_full[..8.min(id_full.len())].to_string();
                                let parsed = Uuid::parse_str(&id_full).ok();
                                rsx! {
                                    tr { 
                                        key: "{id_full}",
                                        td { span { class: "mono", style: "color: var(--accent);", "{id_short}" } }
                                        td { style: "font-weight: 600;", "{r.name}" }
                                        td { "{r.runtime}" }
                                        td { class: "mono", style: "font-size: 12px; opacity: 0.7;", "{r.base_image}" }
                                        td { StatusBadge { status: r.status.clone() } }
                                        td { style: "color: var(--muted);", "{r.created_at}" }
                                        td {
                                            if let Some(uuid) = parsed {
                                                button {
                                                    class: "btn-ghost",
                                                    style: "padding: 6px 12px; font-size: 11px;",
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
