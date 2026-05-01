use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

use forge_core::telemetry;

mod services;
mod state;
mod views;

use state::{init_state, use_app_state};
use views::App;

fn main() {
    // muda panics on macOS if it cannot find the app name in Info.plist or the runtime env.
    // Set CARGO_PKG_NAME at runtime if it's not present to prevent this.
    if std::env::var("CARGO_PKG_NAME").is_err() {
        std::env::set_var("CARGO_PKG_NAME", "forge-desktop");
    }
    boot_log("telemetry init");
    telemetry::init();
    boot_log("init_state starting");
    // Bootstrap async state synchronously before the GUI runs so every view
    // can assume `use_app_state` is ready.
    let app_state = match init_state() {
        Ok(s) => s,
        Err(e) => {
            boot_log(&format!("init_state failed: {e}"));
            eprintln!("[forge-desktop] failed to initialize: {e}");
            std::process::exit(1);
        }
    };
    boot_log("init_state done");

    if should_install_tray() {
        let tray_result = std::panic::catch_unwind(services::tray::install);
        match tray_result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "tray icon unavailable; continuing without it");
            }
            Err(_) => {
                tracing::warn!("tray icon initialization panicked; continuing without tray");
            }
        }
    } else {
        tracing::info!("tray disabled for this launch mode");
    }

    let window = WindowBuilder::new()
        .with_title("SecureImage Forge")
        .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_resizable(true);
    let config = Config::new().with_window(window).with_menu(None);
    LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(app_state)
        .launch(root);
}

fn root() -> Element {
    use_app_state(); // ensures the context is registered for descendants
    rsx! { App {} }
}

fn should_install_tray() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Avoid tray initialization on macOS for now as it causes SIGABRT in bundles.
        // Opt-in via FORGE_ENABLE_TRAY=1
        if std::env::var_os("FORGE_ENABLE_TRAY") == Some("1".into()) {
            return true;
        }
        false
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[cfg(target_os = "macos")]
fn is_running_from_macos_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_owned()))
        .map(|exe| exe.contains(".app/Contents/MacOS/"))
        .unwrap_or(false)
}

fn boot_log(msg: &str) {
    let data_dir = std::env::var_os("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".forge"))
        .unwrap_or_else(|| std::path::PathBuf::from(".forge"));
    let _ = std::fs::create_dir_all(&data_dir);
    let log_path = data_dir.join("boot.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_path) {
        use std::io::Write;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(f, "[{}] {}", now, msg);
    }
}
