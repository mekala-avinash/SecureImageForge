//! Shared application state injected via Dioxus context. Holds repo + log
//! store + toolchain + a handle to the orchestrator so views can dispatch
//! work without ever touching subprocess code directly.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dioxus::prelude::*;

use forge_core::config::Config;
use forge_core::logs::LogStore;
use forge_core::repo::BuildRepo;
use forge_core::storage::Storage;
use forge_core::toolchain::Toolchain;

#[derive(Clone)]
pub struct AppState {
    pub data_dir: PathBuf,
    pub config: Arc<Config>,
    pub repo: Arc<dyn BuildRepo>,
    pub logs: Arc<dyn LogStore>,
    pub provenance: Arc<dyn forge_core::provenance::ProvenanceRepo>,
    pub toolchain: Arc<Toolchain>,
}

pub fn init_state() -> Result<AppState> {
    let data_dir = data_dir();
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    let cfg_path = data_dir.join("config.toml");
    let mut config = Config::load(&cfg_path)?;
    let db_path = data_dir.join("forge.sqlite");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let storage = runtime.block_on(Storage::open(&db_path))?;

    if config.buildkit.managed {
        let manager = forge_core::runtime::RuntimeManager::new(config.vendor.prefix.clone());
        match runtime.block_on(manager.ensure_running()) {
            Ok(addr) => {
                tracing::info!("managed buildkit started at {}", addr);
                config.buildkit.addr = addr;
            }
            Err(e) => {
                tracing::warn!("failed to start managed buildkit: {}; falling back to {}", e, config.buildkit.addr);
            }
        }
    }

    let config = Arc::new(config);

    // Spin up a long-lived runtime that we leak into a static handle so
    // async ops dispatched from UI callbacks share one executor.
    services::install_runtime(runtime);

    let repo: Arc<dyn BuildRepo> = Arc::new(forge_core::repo::SqliteBuildRepo::new(storage.clone()));
    let logs: Arc<dyn LogStore> = Arc::new(forge_core::logs::FileLogStore::new(data_dir.join("logs")));
    let provenance: Arc<dyn forge_core::provenance::ProvenanceRepo> = Arc::new(forge_core::provenance::SqliteProvenanceRepo::new(storage.clone()));
    let toolchain = Arc::new(Toolchain::new(config.vendor.prefix.clone()));

    Ok(AppState {
        data_dir,
        config,
        repo,
        logs,
        provenance,
        toolchain,
    })
}

fn data_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("FORGE_DATA_DIR") {
        return PathBuf::from(p);
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".forge"))
        .unwrap_or_else(|| PathBuf::from(".forge"))
}

pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}

mod services {
    use std::sync::OnceLock;
    use tokio::runtime::Runtime;

    static RUNTIME: OnceLock<Runtime> = OnceLock::new();

    pub fn install_runtime(rt: Runtime) {
        // First call wins; if state is re-initialized in tests this just
        // becomes a silent no-op, which is what we want.
        let _ = RUNTIME.set(rt);
    }

    pub fn handle() -> &'static Runtime {
        RUNTIME
            .get()
            .expect("forge-desktop runtime not initialized")
    }
}

pub use self::services::handle as runtime;
