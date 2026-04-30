use dioxus::prelude::*;

use crate::state::use_app_state;

#[component]
pub fn DoctorView() -> Element {
    let state = use_app_state();
    let prefix = state
        .toolchain
        .prefix()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(none)".into());
    let manifest = state.toolchain.load_manifest();

    let resolutions: Vec<(&'static str, Result<String, String>)> =
        ["buildctl", "trivy", "syft", "cosign", "opa"]
            .iter()
            .map(|tool| {
                let r = state
                    .toolchain
                    .resolve(tool)
                    .map(|p| p.display().to_string())
                    .map_err(|e| e.to_string());
                (*tool, r)
            })
            .collect();

    rsx! {
        section {
            class: "view",
            header { class: "view-header", h1 { "Doctor" } }
            div { class: "panel",
                div { class: "kv-grid",
                    div { class: "kv",
                        div { class: "kv-key", "Vendor prefix" }
                        div { class: "kv-val", code { "{prefix}" } }
                    }
                    div { class: "kv",
                        div { class: "kv-key", "Data dir" }
                        div { class: "kv-val", code { "{state.data_dir.display()}" } }
                    }
                }
            }

            div { class: "panel",
                h2 { "Tool resolution" }
                table { class: "data-table",
                    thead { tr { th { "Tool" } th { "Path" } } }
                    tbody {
                        {resolutions.iter().map(|(tool, res)| {
                            let (badge, value) = match res {
                                Ok(p) => ("ok", p.clone()),
                                Err(e) => ("fail", e.clone()),
                            };
                            rsx! {
                                tr { key: "{tool}",
                                    td { code { "{tool}" } }
                                    td {
                                        span { class: "badge badge-{badge}",
                                            if badge == "ok" { "found" } else { "missing" }
                                        }
                                        " "
                                        code { "{value}" }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            if let Some(m) = manifest.as_ref() {
                div { class: "panel",
                    h2 { "Vendor manifest" }
                    table { class: "data-table",
                        thead { tr {
                            th { "Tool" }
                            th { "Version" }
                            th { "Platform" }
                            th { "SHA-256" }
                        }}
                        tbody {
                            {m.tools.iter().map(|t| rsx! {
                                tr { key: "{t.name}-{t.platform}",
                                    td { "{t.name}" }
                                    td { code { "{t.version}" } }
                                    td { "{t.platform}" }
                                    td { class: "muted", code { "{t.sha256}" } }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
