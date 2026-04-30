//! xtask — repo automation entry point.
//!
//! Phase 0 ships the command surface; full implementations land alongside the
//! tooling adapters they correspond to.

use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Repo automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand, Debug)]
enum Task {
    /// Run `cargo deny check` to enforce the Apache-2.0/MIT license policy.
    LicenseAudit,
    /// Fetch and verify rootless buildkitd binaries for the supported platforms.
    BundleBuildkit {
        /// Platforms in `os/arch` form (e.g. linux/amd64).
        #[arg(long, default_values_t = vec![
            "linux/amd64".to_string(),
            "linux/arm64".to_string(),
            "darwin/amd64".to_string(),
            "darwin/arm64".to_string(),
        ])]
        platforms: Vec<String>,
    },
    /// Build release bundles (Phase 4 — placeholder for now).
    Dist,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Task::LicenseAudit => license_audit(),
        Task::BundleBuildkit { platforms } => bundle_buildkit(&platforms),
        Task::Dist => dist(),
    }
}

fn license_audit() -> Result<()> {
    let status = Command::new("cargo")
        .args(["deny", "check"])
        .status()
        .context("cargo deny not installed; run `cargo install cargo-deny`")?;
    if !status.success() {
        bail!("license audit failed");
    }
    Ok(())
}

fn bundle_buildkit(platforms: &[String]) -> Result<()> {
    println!("[xtask] buildkit bundling deferred to Phase 1");
    for p in platforms {
        println!("  - target: {p}");
    }
    Ok(())
}

fn dist() -> Result<()> {
    println!("[xtask] dist bundling deferred to Phase 4");
    Ok(())
}
