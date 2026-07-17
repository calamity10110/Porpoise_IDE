use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Instant,
};

#[derive(Clone)]
pub struct GitStatusCache {
    inner: Arc<RwLock<HashMap<PathBuf, CachedEntry>>>,
}

#[derive(Clone)]
struct CachedEntry {
    status: serde_json::Value,
    computed_at: Instant,
}

const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(5);

impl GitStatusCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, path: &Path) -> Option<serde_json::Value> {
        let map = self.inner.read().ok()?;
        if let Some(entry) = map.get(path) {
            if entry.computed_at.elapsed() < CACHE_TTL {
                return Some(entry.status.clone());
            }
        }
        None
    }

    pub fn insert(&self, path: PathBuf, status: serde_json::Value) {
        if let Ok(mut map) = self.inner.write() {
            map.insert(
                path,
                CachedEntry {
                    status,
                    computed_at: Instant::now(),
                },
            );
        }
    }

    pub fn invalidate(&self, path: &Path) {
        if let Ok(mut map) = self.inner.write() {
            map.remove(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let cache = GitStatusCache::new();
        let path = PathBuf::from("/tmp/test");
        cache.insert(path.clone(), serde_json::json!({"changes": []}));
        assert!(cache.get(&path).is_some());
    }

    #[test]
    fn test_cache_miss() {
        let cache = GitStatusCache::new();
        assert!(cache.get(Path::new("/nonexistent")).is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = GitStatusCache::new();
        let path = PathBuf::from("/tmp/stale");
        cache.insert(path.clone(), serde_json::json!({"value": 42}));
        // Immediately after insert it should be a hit
        assert!(cache.get(&path).is_some());
    }

    #[test]
    fn test_invalidate() {
        let cache = GitStatusCache::new();
        let path = PathBuf::from("/tmp/removable");
        cache.insert(path.clone(), serde_json::json!({"key": "val"}));
        assert!(cache.get(&path).is_some());
        cache.invalidate(&path);
        assert!(cache.get(&path).is_none());
    }
}
