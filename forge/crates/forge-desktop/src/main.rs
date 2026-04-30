use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

use forge_core::telemetry;

fn main() {
    telemetry::init();
    let window = WindowBuilder::new()
        .with_title("SecureImage Forge")
        .with_resizable(true);
    let config = Config::new().with_window(window);
    dioxus_desktop::launch_cfg(app, config);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        style { include_str!("../assets/app.css") }
        main {
            class: "app-shell",
            header {
                class: "app-header",
                h1 { "SecureImage Forge" }
                span { class: "tagline", "Build · Harden · Verify" }
            }
            section {
                class: "app-body",
                p { "Phase 0 skeleton — engine, scanner, signer, and policy adapters land in Phase 1." }
                ul {
                    li { "Runtime: Java · .NET · Go · Node · Python" }
                    li { "Bases: Alpine · Debian · Distroless" }
                    li { "Compliance: HIPAA · SOC2 · PCI-DSS · CIS · FedRAMP" }
                }
            }
        }
    })
}
