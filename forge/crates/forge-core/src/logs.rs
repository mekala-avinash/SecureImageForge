//! Persistent on-disk logs for builds. One file per build under
//! `<data_dir>/logs/<build_id>.log`. The orchestrator writes the buildkit
//! transcript here; `forge logs <id>` simply reads it back.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::Result;

#[async_trait::async_trait]
pub trait LogStore: Send + Sync {
    async fn write(&self, build_id: Uuid, content: &str) -> Result<PathBuf>;
    async fn read(&self, build_id: Uuid) -> Result<Option<String>>;
}

#[derive(Debug, Clone)]
pub struct FileLogStore {
    root: PathBuf,
}

impl FileLogStore {
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
}

#[async_trait::async_trait]
impl LogStore for FileLogStore {
    async fn write(&self, build_id: Uuid, content: &str) -> Result<PathBuf> {
        self.ensure_root()?;
        let path = self.path_for(build_id);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    async fn read(&self, build_id: Uuid) -> Result<Option<String>> {
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
        let store = FileLogStore::new(dir.path().join("logs"));
        let id = Uuid::new_v4();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            store.write(id, "hello world").await.unwrap();
            let got = store.read(id).await.unwrap().unwrap();
            assert_eq!(got, "hello world");
        });
    }

    #[test]
    fn missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let store = FileLogStore::new(dir.path().join("logs"));
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let got = runtime.block_on(async { store.read(Uuid::new_v4()).await.unwrap() });
        assert!(got.is_none());
    }
}
