//! Service layer used by views. Wraps `forge-core` async APIs in spawn-and-wait
//! helpers that are easy to call from synchronous Dioxus event handlers.

pub mod builds;
pub mod orchestration;
pub mod tray;
pub mod updates;
