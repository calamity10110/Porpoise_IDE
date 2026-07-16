use std::path::Path;

use porpoise_core::error::Result;
use porpoise_git::GitEngine;

pub async fn handle_status(repo_path: &str) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
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
}

pub async fn handle_clone(url: &str, path: &str) -> Result<serde_json::Value> {
    let target = Path::new(path);
    let engine = GitEngine::clone(url, target)?;
    Ok(serde_json::json!({
        "url": url,
        "path": engine.path().display().to_string(),
        "status": "cloned"
    }))
}

pub async fn handle_diff(repo_path: &str, staged: bool) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
    let diff = engine.diff(staged)?;
    Ok(serde_json::json!({ "diff": diff }))
}

pub async fn handle_log(repo_path: &str, count: usize) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
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
}

pub async fn handle_branch_list(repo_path: &str) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
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
}

pub async fn handle_branch_create(repo_path: &str, name: &str) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
    engine.branch_create(name)?;
    Ok(serde_json::json!({ "branch": name, "status": "created" }))
}

pub async fn handle_branch_checkout(repo_path: &str, name: &str) -> Result<serde_json::Value> {
    let engine = GitEngine::open(Path::new(repo_path))?;
    engine.branch_checkout(name)?;
    Ok(serde_json::json!({ "branch": name, "status": "checked_out" }))
}
