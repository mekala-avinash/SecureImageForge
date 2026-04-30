//! System tray integration. Provides a "show window" / "check for updates" /
//! "quit" menu so the app behaves like a daemon that stays available even
//! when the main window is closed.
//!
//! `tray-icon` requires the menu/icon to be created on the same thread that
//! runs the OS event loop. dioxus-desktop's `Config::with_custom_event_handler`
//! is the right hook, but we keep the implementation minimal — full
//! integration with main-thread event polling lands when we wire the updater
//! UI bridge below.

use std::sync::OnceLock;

use anyhow::Result;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

static TRAY: OnceLock<TrayHandles> = OnceLock::new();

#[allow(dead_code)]
struct TrayHandles {
    icon: TrayIcon,
    show_id: String,
    update_id: String,
    quit_id: String,
}

unsafe impl Send for TrayHandles {}
unsafe impl Sync for TrayHandles {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants are emitted by `poll_event`; full main-loop wiring lands in Phase 4.
pub enum TrayCommand {
    ShowWindow,
    CheckForUpdates,
    Quit,
}

/// Build the tray icon + menu. Must be called from the OS main thread.
pub fn install() -> Result<()> {
    if TRAY.get().is_some() {
        return Ok(());
    }
    let menu = Menu::new();
    let show = MenuItem::new("Show window", true, None);
    let update = MenuItem::new("Check for updates", true, None);
    let quit = MenuItem::new("Quit", true, None);
    menu.append_items(&[
        &show,
        &PredefinedMenuItem::separator(),
        &update,
        &PredefinedMenuItem::separator(),
        &quit,
    ])?;

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("SecureImage Forge")
        .with_icon(default_icon())
        .build()?;

    let _ = TRAY.set(TrayHandles {
        icon,
        show_id: show.id().0.clone(),
        update_id: update.id().0.clone(),
        quit_id: quit.id().0.clone(),
    });
    Ok(())
}

/// Drain pending menu events. Returns the resolved command, if any.
/// Call from the desktop event loop.
#[allow(dead_code)]
pub fn poll_event() -> Option<TrayCommand> {
    let handles = TRAY.get()?;
    let event = MenuEvent::receiver().try_recv().ok()?;
    let id = event.id().0.as_str();
    if id == handles.show_id {
        Some(TrayCommand::ShowWindow)
    } else if id == handles.update_id {
        Some(TrayCommand::CheckForUpdates)
    } else if id == handles.quit_id {
        Some(TrayCommand::Quit)
    } else {
        None
    }
}

/// Embed a 32×32 monochrome RGBA icon: filled square with a Klein-blue notch.
/// Avoids shipping a binary asset for now.
fn default_icon() -> tray_icon::Icon {
    const SIZE: u32 = 32;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let edge = x == 0 || y == 0 || x == SIZE - 1 || y == SIZE - 1;
            let notch = (x as i32 - y as i32).abs() < 4 && x > 6 && y > 6 && x < 26 && y < 26;
            if notch {
                rgba.extend_from_slice(&[0xFF, 0x3B, 0x30, 0xFF]); // signal red
            } else if edge {
                rgba.extend_from_slice(&[0x0A, 0x0A, 0x0A, 0xFF]);
            } else {
                rgba.extend_from_slice(&[0x00, 0x2F, 0xA7, 0xFF]); // klein
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, SIZE, SIZE).expect("static icon is valid")
}
