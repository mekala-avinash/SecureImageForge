use dioxus::prelude::*;

use forge_core::updater::UpdateDecision;

use crate::services::updates;
use crate::state::use_app_state;

#[component]
pub fn SettingsView() -> Element {
    let state = use_app_state();
    let mut last = use_signal::<Option<String>>(|| None);
    let mut decision = use_signal::<Option<UpdateDecision>>(|| None);
    let mut error = use_signal::<Option<String>>(|| None);
    
    // Config fields
    let mut buildkit_addr = use_signal(|| state.config.buildkit.addr.clone());
    let mut registry_target = use_signal(|| state.config.registry.default_target.clone().unwrap_or_default());
    let mut updater_channel = use_signal(|| state.config.updater.channel.clone());
    let mut saved = use_signal(|| false);
    let mut save_error = use_signal::<Option<String>>(|| None);

    let state_for_closure = state.clone();
    let mut check = move || {
        error.set(None);
        let state_for_async = state_for_closure.clone();
        spawn(async move {
            match updates::check_async(&state_for_async).await {
                Ok(d) => {
                    decision.set(Some(d));
                    last.set(Some(now_iso()));
                }
                Err(e) => error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        section { class: "view",
            header { class: "view-header", h1 { "Core Settings" } }

            div { class: "glass-card",
                h2 { style: "margin-bottom: 20px;", "Kernel Configuration" }
                form {
                    class: "form",
                    onsubmit: move |_| {
                        save_error.set(None);
                        saved.set(false);
                        let mut new_config = (*state.config).clone();
                        new_config.buildkit.addr = buildkit_addr.read().clone();
                        
                        let target = registry_target.read().clone();
                        new_config.registry.default_target = if target.is_empty() { None } else { Some(target) };
                        
                        new_config.updater.channel = updater_channel.read().clone();
                        
                        match crate::services::config_service::save_config(&state.data_dir, &new_config) {
                            Ok(_) => saved.set(true),
                            Err(e) => save_error.set(Some(e.to_string())),
                        }
                    },
                    div { 
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 24px;",
                        div { class: "form-row",
                            label { "BuildKit Endpoint" }
                            input { r#type: "text", placeholder: "e.g. unix:///var/run/buildkit/buildkitd.sock", value: "{buildkit_addr.read()}", oninput: move |e| buildkit_addr.set(e.value()) }
                        }
                        div { class: "form-row",
                            label { "Default Registry Mirror" }
                            input { r#type: "text", placeholder: "e.g. docker.io", value: "{registry_target.read()}", oninput: move |e| registry_target.set(e.value()) }
                        }
                    }
                    div { 
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 24px;",
                        div { class: "form-row",
                            label { "Neural Update Channel" }
                            select { 
                                oninput: move |e| updater_channel.set(e.value()),
                                option { value: "stable", selected: *updater_channel.read() == "stable", "Stable Release" }
                                option { value: "beta", selected: *updater_channel.read() == "beta", "Beta Preview" }
                            }
                        }
                        div {}
                    }
                    div { class: "form-actions",
                        button { class: "btn-primary", r#type: "submit", "Commit Changes" }
                    }
                    if *saved.read() {
                        p { style: "color: var(--ok); font-weight: 600; margin-top: 16px;", "Kernel state updated. Restart recommended for full sync." }
                    }
                    if let Some(e) = save_error.read().as_ref() {
                        p { style: "color: var(--signal); font-weight: 600; margin-top: 16px;", "{e}" }
                    }
                }
            }

            div { class: "glass-card",
                h2 { style: "margin-bottom: 20px;", "Forge Updates" }
                div { 
                    style: "display: flex; align-items: center; gap: 20px; margin-bottom: 24px;",
                    button { class: "btn-primary", onclick: move |_| check(), "Check for Updates" }
                    if let Some(t) = last.read().as_ref() {
                        span { class: "muted", style: "font-size: 12px;", "Last sync: {t}" }
                    }
                }
                {match decision.read().as_ref() {
                    None => rsx! { p { class: "muted", "Initialize update check sequence." } },
                    Some(UpdateDecision::UpToDate) => rsx! {
                        div {
                            style: "display: flex; align-items: center; gap: 12px;",
                            span { class: "status-badge ok", "UP TO DATE" }
                            span { "System integrity verified at latest version." }
                        }
                    },
                    Some(UpdateDecision::UpdateAvailable { from, to, release }) => rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 12px;",
                            div {
                                style: "display: flex; align-items: center; gap: 12px;",
                                span { class: "status-badge warn", "PATCH AVAILABLE" }
                                span { style: "font-weight: 600;", "{from} → {to}" }
                            }
                            div { 
                                class: "log",
                                style: "opacity: 0.8;",
                                div { "Target: {release.platform}" }
                                div { "Source: {release.url}" }
                                div { "Digest: {release.sha256}" }
                            }
                        }
                    },
                    Some(UpdateDecision::UpgradeRequired { current, minimum }) => rsx! {
                        div {
                            style: "display: flex; align-items: center; gap: 12px;",
                            span { class: "status-badge fail", "UPGRADE MANDATORY" }
                            span { "{current} below minimum threshold {minimum}" }
                        }
                    },
                }}
                if let Some(e) = error.read().as_ref() {
                    p { style: "color: var(--signal); font-weight: 600; margin-top: 16px;", "{e}" }
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

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}
