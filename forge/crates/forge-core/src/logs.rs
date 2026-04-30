//! Persistent on-disk logs for builds. One file per build under
//! `<data_dir>/logs/<build_id>.log`. The orchestrator writes the buildkit
//! transcript here; `forge logs <id>` simply reads it back.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::Result;

#[derive(Debug, Clone)]
pub struct LogStore {
    root: PathBuf,
}

impl LogStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_for(&self, build_id: Uuid) -> PathBuf {
        self.root.join(format!("{build_id}.log"))
    }

    pub fn ensure_root(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }

    pub fn write(&self, build_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_root()?;
        let path = self.path_for(build_id);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    pub fn read(&self, build_id: Uuid) -> Result<Option<String>> {
        let path = self.path_for(build_id);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(std::fs::read_to_string(path)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_then_read_round_trip() {
        let dir = TempDir::new().unwrap();
        let store = LogStore::new(dir.path().join("logs"));
        let id = Uuid::new_v4();
        store.write(id, "hello world").unwrap();
        let got = store.read(id).unwrap().unwrap();
        assert_eq!(got, "hello world");
    }

    #[test]
    fn missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let store = LogStore::new(dir.path().join("logs"));
        let got = store.read(Uuid::new_v4()).unwrap();
        assert!(got.is_none());
    }
}
