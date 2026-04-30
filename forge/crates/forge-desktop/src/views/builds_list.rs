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

    let _ = *tick.read();
    let rows = builds::list(&state.repo, 200).unwrap_or_default();

    rsx! {
        section {
            class: "view",
            header {
                class: "view-header",
                h1 { "Builds" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| route.set(Route::NewBuild),
                    "+ New build"
                }
            }
            div {
                class: "panel",
                if rows.is_empty() {
                    p { class: "muted", "No builds yet." }
                } else {
                    table {
                        class: "data-table",
                        thead { tr {
                            th { "ID" }
                            th { "Name" }
                            th { "Runtime" }
                            th { "Base" }
                            th { "Status" }
                            th { "Created" }
                            th { "" }
                        }}
                        tbody {
                            {rows.iter().map(|r| {
                                let id_full = r.id.clone();
                                let id_short = id_full[..8.min(id_full.len())].to_string();
                                let parsed = Uuid::parse_str(&id_full).ok();
                                rsx! {
                                    tr { key: "{id_full}",
                                        td { code { "{id_short}" } }
                                        td { class: "build-name", "{r.name}" }
                                        td { "{r.runtime}" }
                                        td { "{r.base_image}" }
                                        td { StatusBadge { status: r.status.clone() } }
                                        td { class: "muted", "{r.created_at}" }
                                        td {
                                            if let Some(uuid) = parsed {
                                                button {
                                                    class: "btn btn-ghost",
                                                    onclick: move |_| route.set(Route::Build(uuid)),
                                                    "Open"
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
