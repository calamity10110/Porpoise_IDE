use porpoise_core::error::Result;
use porpoise_git::GitEngine;

pub async fn handle_status(repo_path: &str) -> Result<serde_json::Value> {
    let engine = GitEngine::open(std::path::Path::new(repo_path))?;
    let changes = engine.status()?;
    let json_changes: Vec<serde_json::Value> = changes.iter().map(|c| {
        serde_json::json!({
            "path": c.path.display().to_string(),
            "status": format!("{:?}", c.status),
        })
    }).collect();
    Ok(serde_json::json!({ "changes": json_changes }))
}

pub async fn handle_clone(url: &str, path: &str) -> Result<serde_json::Value> {
    let target = std::path::Path::new(path);
    let engine = GitEngine::clone(url, target)?;
    Ok(serde_json::json!({
        "url": url,
        "path": engine.path().display().to_string(),
        "status": "cloned"
    }))
}
