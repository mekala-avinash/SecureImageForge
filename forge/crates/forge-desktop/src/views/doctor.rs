use dioxus::prelude::*;

use crate::state::use_app_state;

#[component]
pub fn DoctorView() -> Element {
    let state = use_app_state();
    let mut logs = use_signal(String::new);
    let mut running = use_signal(|| false);
    let mut success = use_signal(|| false);
    let mut error = use_signal(String::new);

    let mut download_tools = move || {
        running.set(true);
        error.set(String::new());
        logs.set(String::new());

        let cmd = std::process::Command::new("cargo")
            .args(["xtask", "dev-setup"])
            .output();

        match cmd {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                logs.set(format!("{}\n{}", stdout, stderr));
                if out.status.success() {
                    success.set(true);
                } else {
                    error.set("Failed to download tools.".into());
                }
            }
            Err(e) => {
                error.set(e.to_string());
            }
        }
        running.set(false);
    };
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
            header { class: "view-header", h1 { "System Diagnostics" } }
            div { class: "glass-card",
                div { 
                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 24px;",
                    div {
                        div { style: "color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 4px;", "Vendor Prefix" }
                        div { class: "mono", style: "color: var(--accent);", "{prefix}" }
                    }
                    div {
                        div { style: "color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 4px;", "Operational Root" }
                        div { class: "mono", "{state.data_dir.display()}" }
                    }
                }
            }

            div { class: "glass-card",
                div { 
                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px;",
                    h2 { "Resolved Toolchain" }
                    if resolutions.iter().any(|(_, res)| res.is_err()) {
                        button { 
                            class: "btn-primary", 
                            disabled: *running.read() || *success.read(),
                            onclick: move |_| { download_tools(); },
                            if *running.read() {
                                "Synchronizing..."
                            } else if *success.read() {
                                "Sync Complete"
                            } else {
                                "Provision Toolchain"
                            }
                        }
                    }
                }
                
                if !logs.read().is_empty() {
                    pre { 
                        class: "log",
                        "{logs.read()}"
                    }
                }

                if !error.read().is_empty() {
                    p { style: "color: var(--signal); font-weight: 600;", "{error.read()}" }
                }

                table { 
                    class: "data-table",
                    thead { tr { 
                        th { "Protocol/Tool" } 
                        th { "Resolution Path" } 
                    } }
                    tbody {
                        {resolutions.iter().map(|(tool, res)| {
                            let (badge, value) = match res {
                                Ok(p) => ("ok", p.clone()),
                                Err(e) => ("fail", e.clone()),
                            };
                            let status_text = if badge == "ok" { "OPERATIONAL" } else { "MISSING" };
                            rsx! {
                                tr { 
                                    key: "{tool}",
                                    td { span { class: "mono", style: "color: var(--accent);", "{tool}" } }
                                    td { 
                                        div {
                                            style: "display: flex; align-items: center; gap: 12px;",
                                            span { class: "status-badge {badge}", "{status_text}" }
                                            span { class: "mono", style: "font-size: 12px; opacity: 0.7;", "{value}" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            if let Some(m) = manifest.as_ref() {
                div { class: "glass-card",
                    h2 { style: "margin-bottom: 20px;", "Vendor Manifest" }
                    table { 
                        class: "data-table",
                        thead { tr {
                            th { "Tool" }
                            th { "Version" }
                            th { "Platform" }
                            th { "SHA-256 Digest" }
                        }}
                        tbody {
                            {m.tools.iter().map(|t| rsx! {
                                tr { 
                                    key: "{t.name}-{t.platform}",
                                    td { "{t.name}" }
                                    td { span { class: "mono", "{t.version}" } }
                                    td { "{t.platform}" }
                                    td { class: "mono", style: "font-size: 11px; opacity: 0.5;", "{t.sha256}" }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
