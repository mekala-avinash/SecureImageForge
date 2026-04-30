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

    let state_for_closure = state.clone();
    let mut check = move || {
        error.set(None);
        match updates::check(&state_for_closure) {
            Ok(d) => {
                decision.set(Some(d));
                last.set(Some(now_iso()));
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        section { class: "view",
            header { class: "view-header", h1 { "Settings" } }

            div { class: "panel",
                h2 { "Application" }
                div { class: "kv-grid",
                    Kv { k: "Version", v: env!("CARGO_PKG_VERSION").to_string() }
                    Kv { k: "Channel", v: state.config.updater.channel.clone() }
                    Kv { k: "Feed",    v: state.config.updater.feed_url.clone() }
                    Kv { k: "Auto check", v: state.config.updater.auto_check.to_string() }
                }
            }

            div { class: "panel",
                h2 { "Updates" }
                div { class: "row",
                    button { class: "btn btn-primary", onclick: move |_| check(), "Check for updates" }
                    if let Some(t) = last.read().as_ref() {
                        span { class: "muted", "Last checked: {t}" }
                    }
                }
                {match decision.read().as_ref() {
                    None => rsx! { p { class: "muted", "Click ‘Check for updates’ to query the feed." } },
                    Some(UpdateDecision::UpToDate) => rsx! {
                        p { span { class: "badge badge-ok", "up to date" } }
                    },
                    Some(UpdateDecision::UpdateAvailable { from, to, release }) => rsx! {
                        div {
                            p { span { class: "badge badge-warn", "update available" } " "
                                "{from} → {to}" }
                            p { class: "muted", "Platform: {release.platform}" }
                            p { class: "muted", "URL: " code { "{release.url}" } }
                            p { class: "muted", "SHA-256: " code { "{release.sha256}" } }
                        }
                    },
                    Some(UpdateDecision::UpgradeRequired { current, minimum }) => rsx! {
                        div {
                            p { span { class: "badge badge-fail", "upgrade required" } " "
                                "{current} < {minimum}" }
                        }
                    },
                }}
                if let Some(e) = error.read().as_ref() {
                    p { class: "form-error", "{e}" }
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
