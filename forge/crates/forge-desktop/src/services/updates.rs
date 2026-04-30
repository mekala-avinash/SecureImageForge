//! Desktop wiring for the auto-updater. Pulls the manifest from the feed,
//! optionally verifies its cosign signature, and asks `forge-core::updater`
//! for a decision against the current binary version.

use std::sync::Arc;

use anyhow::{Context, Result};

use forge_core::process::TokioRunner;
use forge_core::toolchain::Toolchain;
use forge_core::updater::{UpdateDecision, UpdateManifest, Updater, UpdaterConfig};

use crate::state::{runtime, AppState};

/// Synchronous wrapper around the async fetch+decide flow so it can be called
/// from a Dioxus event handler.
pub fn check(state: &AppState) -> Result<UpdateDecision> {
    let cfg = state.config.updater.clone();
    let toolchain: Arc<Toolchain> = state.toolchain.clone();
    runtime().block_on(async move { check_async(toolchain, cfg).await })
}

async fn check_async(
    toolchain: Arc<Toolchain>,
    section: forge_core::config::UpdaterSection,
) -> Result<UpdateDecision> {
    let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
    let updater = Updater::new(
        runner,
        toolchain,
        UpdaterConfig {
            feed_url: section.feed_url.clone(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            channel: section.channel.clone(),
            cosign_key_path: section.cosign_key_path.clone(),
            allow_unsigned: section.allow_unsigned,
        },
    );

    let payload = reqwest::get(&section.feed_url)
        .await
        .with_context(|| format!("GET {}", section.feed_url))?
        .error_for_status()?
        .bytes()
        .await?;

    if !section.allow_unsigned {
        let sig = reqwest::get(format!("{}.sig", section.feed_url))
            .await
            .ok()
            .and_then(|r| r.error_for_status().ok());
        if let Some(resp) = sig {
            let bytes = resp.bytes().await?;
            updater.verify_manifest(&payload, &bytes).await?;
        } else {
            anyhow::bail!("update manifest is unsigned and allow_unsigned=false");
        }
    }

    let manifest: UpdateManifest = serde_json::from_slice(&payload)?;
    Ok(updater.decide(&manifest))
}
