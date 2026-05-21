use dioxus::prelude::*;
use crate::state::use_app_state;
use crate::views::Route;

#[component]
pub fn Onboarding(route: Signal<Route>) -> Element {
    let _state = use_app_state();
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
            div { class: "glass-card",
                style: "max-width: 800px;",
                h2 { style: "margin-bottom: 16px;", "First-run Setup Sequence" }
                p { style: "margin-bottom: 24px; color: var(--muted);", "SecureImageForge relies on several hardened cloud-native utilities (BuildKit, Trivy, Syft, Cosign, OPA) which are currently missing on your local environment." }
                
                div { 
                    style: "margin-bottom: 24px;",
                    button { 
                        class: "btn-primary", 
                        disabled: *running.read() || *success.read(),
                        onclick: move |_| { download_tools(); },
                        if *running.read() {
                            "Downloading Host Binaries..."
                        } else if *success.read() {
                            "Binaries Downloaded!"
                        } else {
                            "Download Required Tools"
                        }
                    }
                }

                if !logs.read().is_empty() {
                    pre { class: "log", style: "margin-bottom: 24px;", "{logs.read()}" }
                }

                if !error.read().is_empty() {
                    p { style: "color: var(--signal); font-weight: 600; margin-bottom: 24px;", "{error.read()}" }
                }

                if *success.read() {
                    div { 
                        class: "glass-card",
                        style: "border-color: var(--accent-glow); background: rgba(0, 242, 255, 0.02); padding: 20px; display: flex; flex-direction: column; gap: 16px;",
                        h2 { "Step 2: Start BuildKit Daemon" }
                        p { class: "muted", "You must launch the rootless BuildKit daemon to execute secure build matrices:" }
                        pre { class: "log", "buildkitd --rootless &" }
                        div {
                            style: "display: flex; justify-content: flex-end;",
                            button { class: "btn-primary", onclick: move |_| route.set(Route::NewBuild), "Access Mission Control" }
                        }
                    }
                }
            }
        }
    }
}
