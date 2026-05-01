//! Top-level Dioxus views. The shell renders a sidebar + main panel; routing
//! is handled by a single `Route` enum stored in shared signal state.

use dioxus::prelude::*;

mod build_detail;
mod builds_list;
mod dashboard;
mod doctor;
mod new_build;
mod onboarding;
mod settings;

pub use build_detail::BuildDetail;
pub use builds_list::BuildsList;
pub use dashboard::Dashboard;
pub use doctor::DoctorView;
pub use new_build::NewBuild;
pub use onboarding::Onboarding;
pub use settings::SettingsView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Onboarding,
    Dashboard,
    Builds,
    NewBuild,
    Build(uuid::Uuid),
    Doctor,
    Settings,
}

#[component]
pub fn App() -> Element {
    let state = crate::state::use_app_state();
    let needs_onboarding = ["buildctl", "trivy", "syft", "cosign", "opa"]
        .iter()
        .any(|t| state.toolchain.resolve(t).is_err());
    
    let route = use_signal(|| if needs_onboarding { Route::Onboarding } else { Route::Dashboard });
    rsx! {
        style { {include_str!("../../assets/app.css")} }
        div {
            class: "app-shell",
            Sidebar { route: route }
            main {
                class: "app-main",
                {match &*route.read() {
                    Route::Onboarding => rsx! { Onboarding { route: route } },
                    Route::Dashboard => rsx! { Dashboard {} },
                    Route::Builds => rsx! { BuildsList { route: route } },
                    Route::NewBuild => rsx! { NewBuild { route: route } },
                    Route::Build(id) => rsx! { BuildDetail { build_id: *id } },
                    Route::Doctor => rsx! { DoctorView {} },
                    Route::Settings => rsx! { SettingsView {} },
                }}
            }
        }
    }
}

#[component]
fn Sidebar(route: Signal<Route>) -> Element {
    let nav = |target: Route, label: &'static str, icon: &'static str| {
        let active = *route.read() == target;
        let class = if active {
            "nav-item active"
        } else {
            "nav-item"
        };
        rsx! {
            button {
                class: "{class}",
                onclick: move |_| route.set(target.clone()),
                span { class: "nav-icon", "{icon}" }
                span { class: "nav-label", "{label}" }
            }
        }
    };

    rsx! {
        aside {
            class: "app-sidebar",
            div {
                class: "brand",
                span { class: "brand-mark", "◆" }
                div {
                    class: "brand-text",
                    div { class: "brand-name", "SECUREIMAGE" }
                    div { class: "brand-tag", "Forge OS v0.1" }
                }
            }
            nav {
                class: "app-nav",
                {nav(Route::Dashboard, "Mission Control", "▣")}
                {nav(Route::Builds,    "Build History",    "≡")}
                {nav(Route::NewBuild,  "Initialize Forge", "+")}
                {nav(Route::Doctor,    "System Diagnostics", "✓")}
                {nav(Route::Settings,  "Core Settings",  "⚙")}
            }
            div {
                class: "sidebar-footer",
                span { style: "opacity: 0.5;", "●" }
                span { "SecureImage Forge" }
                span { style: "margin-left: auto; opacity: 0.5;", "{env!(\"CARGO_PKG_VERSION\")}" }
            }
        }
    }
}
