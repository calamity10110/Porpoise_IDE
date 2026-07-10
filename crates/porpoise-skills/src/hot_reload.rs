use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::sync::RwLock;

use crate::pipeline::CompilationPipeline;

/// Tracks file modification times for hot-reload detection.
struct CachedModule {
    skill_id: String,
    path: PathBuf,
    last_modified: SystemTime,
    #[allow(dead_code)]
    wasm_bytes: Vec<u8>,
}

/// Watches skill plugin files for changes and recompiles them on modification.
///
/// Maintains a cache of compiled modules keyed by skill ID. When a source
/// file's modification time changes, the module is recompiled and the cache
/// entry is replaced. Callers poll `check_for_changes()` to trigger reloads.
pub struct HotReloadManager {
    cache: Arc<RwLock<HashMap<String, CachedModule>>>,
    pipeline: Arc<CompilationPipeline>,
}

impl HotReloadManager {
    pub fn new(pipeline: CompilationPipeline) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            pipeline: Arc::new(pipeline),
        }
    }

    pub async fn add(&self, skill_id: &str, path: &Path) -> Result<()> {
        let modified = self.get_modified(path)?;
        let compiled = self.pipeline.compile_from_file(skill_id, path)?;
        let wasm_bytes = compiled
            .module
            .serialize()
            .map_err(|e| PorpoiseError::Wasm(format!("serialize: {e}")))?
            .to_vec();

        self.cache.write().await.insert(
            skill_id.to_string(),
            CachedModule {
                skill_id: skill_id.to_string(),
                path: path.to_path_buf(),
                last_modified: modified,
                wasm_bytes,
            },
        );
        Ok(())
    }

    pub async fn remove(&self, skill_id: &str) {
        self.cache.write().await.remove(skill_id);
    }

    /// Checks all cached modules for file modifications. Returns the IDs
    /// of skills that were reloaded.
    pub async fn check_for_changes(&self) -> Result<Vec<String>> {
        let entries: Vec<(String, PathBuf, SystemTime)> = {
            let cache = self.cache.read().await;
            cache
                .values()
                .map(|m| (m.skill_id.clone(), m.path.clone(), m.last_modified))
                .collect()
        };

        let mut reloaded = Vec::new();

        for (skill_id, path, last_known) in entries {
            let current = match self.get_modified(&path) {
                Ok(t) => t,
                Err(_) => continue,
            };

            if current > last_known {
                match self.pipeline.compile_from_file(&skill_id, &path) {
                    Ok(compiled) => {
                        let wasm = compiled
                            .module
                            .serialize()
                            .map_err(|e| PorpoiseError::Wasm(format!("serialize: {e}")))?
                            .to_vec();
                        let mut cache = self.cache.write().await;
                        if let Some(entry) = cache.get_mut(&skill_id) {
                            entry.last_modified = current;
                            entry.wasm_bytes = wasm;
                        }
                        reloaded.push(skill_id);
                    }
                    Err(e) => {
                        tracing::warn!("hot-reload failed for {skill_id}: {e}");
                    }
                }
            }
        }

        Ok(reloaded)
    }

    /// Returns the list of currently cached skill IDs.
    pub async fn cached_ids(&self) -> Vec<String> {
        self.cache.read().await.keys().cloned().collect()
    }

    /// Returns the cached WASM bytes for a skill.
    pub async fn get_bytes(&self, skill_id: &str) -> Option<Vec<u8>> {
        self.cache.read().await.get(skill_id).map(|m| m.wasm_bytes.clone())
    }

    fn get_modified(&self, path: &Path) -> Result<SystemTime> {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(|e| PorpoiseError::Plugin(format!("stat {path:?}: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    const SAMPLE_WAT: &str = r#"
        (module
            (func (export "run") (result i32) i32.const 42)
        )
    "#;

    #[tokio::test]
    async fn test_add_and_cached() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("skill.wat");
        std::fs::write(&path, SAMPLE_WAT).unwrap();

        let pipeline = CompilationPipeline::new().unwrap();
        let mgr = HotReloadManager::new(pipeline);

        mgr.add("test", &path).await.unwrap();

        let cached = mgr.cached_ids().await;
        assert!(cached.contains(&"test".to_string()));

        let bytes = mgr.get_bytes("test").await;
        assert!(bytes.is_some());
    }

    #[tokio::test]
    async fn test_detect_change() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("skill.wat");
        std::fs::write(&path, SAMPLE_WAT).unwrap();

        let pipeline = CompilationPipeline::new().unwrap();
        let mgr = HotReloadManager::new(pipeline);

        mgr.add("test", &path).await.unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));

        let updated_wat = r#"
            (module
                (func (export "run") (result i32) i32.const 99)
            )
        "#;
        std::fs::write(&path, updated_wat).unwrap();

        let reloaded = mgr.check_for_changes().await.unwrap();
        assert!(reloaded.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_no_change_no_reload() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("skill.wat");
        std::fs::write(&path, SAMPLE_WAT).unwrap();

        let pipeline = CompilationPipeline::new().unwrap();
        let mgr = HotReloadManager::new(pipeline);

        mgr.add("test", &path).await.unwrap();
        let reloaded = mgr.check_for_changes().await.unwrap();
        assert!(reloaded.is_empty());
    }
}
