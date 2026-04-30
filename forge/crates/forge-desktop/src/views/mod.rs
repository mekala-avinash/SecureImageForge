//! Top-level Dioxus views. The shell renders a sidebar + main panel; routing
//! is handled by a single `Route` enum stored in shared signal state.

use dioxus::prelude::*;

mod build_detail;
mod builds_list;
mod dashboard;
mod doctor;
mod new_build;
mod settings;

pub use build_detail::BuildDetail;
pub use builds_list::BuildsList;
pub use dashboard::Dashboard;
pub use doctor::DoctorView;
pub use new_build::NewBuild;
pub use settings::SettingsView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Dashboard,
    Builds,
    NewBuild,
    Build(uuid::Uuid),
    Doctor,
    Settings,
}

#[component]
pub fn App() -> Element {
    let route = use_signal(|| Route::Dashboard);
    rsx! {
        style { {include_str!("../../assets/app.css")} }
        div {
            class: "app-shell",
            Sidebar { route: route }
            main {
                class: "app-main",
                {match &*route.read() {
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
                    div { class: "brand-name", "SecureImage Forge" }
                    div { class: "brand-tag", "Build · Harden · Verify" }
                }
            }
            nav {
                class: "app-nav",
                {nav(Route::Dashboard, "Dashboard", "▣")}
                {nav(Route::Builds,    "Builds",    "≡")}
                {nav(Route::NewBuild,  "New build", "+")}
                {nav(Route::Doctor,    "Doctor",    "✓")}
                {nav(Route::Settings,  "Settings",  "⚙")}
            }
            div {
                class: "sidebar-footer",
                span { "v" }
                span { {env!("CARGO_PKG_VERSION")} }
            }
        }
    }
}
