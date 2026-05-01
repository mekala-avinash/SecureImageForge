use dioxus::prelude::*;
use crate::state::use_app_state;
use crate::views::Route;

#[component]
pub fn Onboarding(route: Signal<Route>) -> Element {
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

    rsx! {
        section { class: "view",
            header { class: "view-header", h1 { "Welcome to SecureImageForge" } }
            div { class: "panel",
                h2 { "First-run setup" }
                p { "SecureImageForge relies on several open-source tools (BuildKit, Trivy, Syft, Cosign, OPA) which are missing on your system." }
                
                div { class: "form-actions",
                    button { 
                        class: "btn btn-primary", 
                        disabled: *running.read() || *success.read(),
                        onclick: move |_| { download_tools(); },
                        if *running.read() {
                            "Downloading..."
                        } else if *success.read() {
                            "Downloaded!"
                        } else {
                            "Download tools"
                        }
                    }
                }

                if !logs.read().is_empty() {
                    pre { class: "log", "{logs.read()}" }
                }

                if !error.read().is_empty() {
                    p { class: "form-error", "{error.read()}" }
                }

                if *success.read() {
                    div { class: "panel",
                        h2 { "Step 2: Start BuildKit" }
                        p { "You need to start the rootless BuildKit daemon before building images:" }
                        pre { class: "log", "buildkitd --rootless &" }
                        button { class: "btn btn-primary", onclick: move |_| route.set(Route::NewBuild), "Continue" }
                    }
                }
            }
        }
    }
}
