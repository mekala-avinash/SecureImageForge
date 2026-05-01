use std::path::Path;
use anyhow::Result;
use forge_core::config::Config;

pub fn save_config(data_dir: &Path, config: &Config) -> Result<()> {
    let cfg_path = data_dir.join("config.toml");
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&cfg_path, content)?;
    Ok(())
}
