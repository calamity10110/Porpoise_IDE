use std::path::{Path, PathBuf};

use porpoise_core::error::{PorpoiseError, Result};
use porpoise_git::{GitEngine, cache::GitStatusCache};

pub async fn handle_status(repo_path: &str, cache: &GitStatusCache) -> Result<serde_json::Value> {
    let path = Path::new(repo_path);
    // Check cache first
    if let Some(cached) = cache.get(path) {
        return Ok(cached);
    }
    // Cache miss — spawn_blocking for git2
    let repo_path = repo_path.to_string();
    let cache_key = repo_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        let changes = engine.status()?;
        let json_changes: Vec<serde_json::Value> = changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "path": c.path.display().to_string(),
                    "status": format!("{:?}", c.status),
                })
            })
            .collect();
        Ok(serde_json::json!({ "changes": json_changes }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;

    // Cache the result
    cache.insert(PathBuf::from(cache_key), result.clone());
    Ok(result)
}

pub async fn handle_clone(url: &str, path: &str) -> Result<serde_json::Value> {
    let url = url.to_string();
    let path = path.to_string();
    let result = tokio::task::spawn_blocking(move || {
        let target = Path::new(&path);
        let engine = GitEngine::clone(&url, target)?;
        Ok(serde_json::json!({
            "url": url,
            "path": engine.path().display().to_string(),
            "status": "cloned"
        }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;
    Ok(result)
}

pub async fn handle_diff(repo_path: &str, staged: bool, cache: &GitStatusCache) -> Result<serde_json::Value> {
    let path = Path::new(repo_path);
    // Check cache first
    if let Some(cached) = cache.get(path) {
        return Ok(cached);
    }
    let repo_path = repo_path.to_string();
    let cache_key = repo_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        let diff = engine.diff(staged)?;
        Ok(serde_json::json!({ "diff": diff }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;

    cache.insert(PathBuf::from(cache_key), result.clone());
    Ok(result)
}

pub async fn handle_log(repo_path: &str, count: usize, cache: &GitStatusCache) -> Result<serde_json::Value> {
    let path = Path::new(repo_path);
    // Check cache first
    if let Some(cached) = cache.get(path) {
        return Ok(cached);
    }
    let repo_path = repo_path.to_string();
    let cache_key = repo_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        let entries = engine.log(count)?;
        let list: Vec<serde_json::Value> = entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "author": e.author,
                    "message": e.message,
                    "timestamp": e.timestamp,
                })
            })
            .collect();
        Ok(serde_json::json!({ "commits": list }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;

    cache.insert(PathBuf::from(cache_key), result.clone());
    Ok(result)
}

pub async fn handle_branch_list(repo_path: &str, cache: &GitStatusCache) -> Result<serde_json::Value> {
    let path = Path::new(repo_path);
    // Check cache first
    if let Some(cached) = cache.get(path) {
        return Ok(cached);
    }
    let repo_path = repo_path.to_string();
    let cache_key = repo_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        let branches = engine.branch_list()?;
        let list: Vec<serde_json::Value> = branches
            .iter()
            .map(|b| {
                serde_json::json!({
                    "name": b.name,
                    "is_head": b.is_head,
                })
            })
            .collect();
        Ok(serde_json::json!({ "branches": list }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;

    cache.insert(PathBuf::from(cache_key), result.clone());
    Ok(result)
}

pub async fn handle_branch_create(repo_path: &str, name: &str) -> Result<serde_json::Value> {
    let repo_path = repo_path.to_string();
    let name = name.to_string();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        engine.branch_create(&name)?;
        Ok(serde_json::json!({ "branch": name, "status": "created" }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;
    Ok(result)
}

pub async fn handle_branch_checkout(repo_path: &str, name: &str) -> Result<serde_json::Value> {
    let repo_path = repo_path.to_string();
    let name = name.to_string();
    let result = tokio::task::spawn_blocking(move || {
        let engine = GitEngine::open(Path::new(&repo_path))?;
        engine.branch_checkout(&name)?;
        Ok(serde_json::json!({ "branch": name, "status": "checked_out" }))
    })
    .await
    .map_err(|e| PorpoiseError::Internal(e.to_string()))??;
    Ok(result)
}
