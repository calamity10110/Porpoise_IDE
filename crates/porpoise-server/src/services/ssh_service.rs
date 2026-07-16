use porpoise_core::error::Result;
use porpoise_ssh::{SshManager, auth::AuthMethod};

pub async fn handle_connect(
    host: &str,
    port: u16,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
) -> Result<serde_json::Value> {
    let manager = SshManager::new();
    let auth = match (password, key_path) {
        (Some(pw), _) => AuthMethod::Password(pw.into()),
        (_, Some(path)) => AuthMethod::KeyFile(path.into(), None),
        _ => AuthMethod::Agent,
    };
    match manager.connect(host, port, user, &auth).await {
        Ok(id) => Ok(serde_json::json!({ "session_id": id, "status": "connected", "host": host })),
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

pub async fn handle_worktree(host: &str, port: u16, user: &str, repo: &str, branch: &str) -> Result<serde_json::Value> {
    let auth = AuthMethod::Agent;
    let config = porpoise_ssh::reconnect::ReconnectConfig::default();
    match porpoise_ssh::SshWorktreeManager::connect(host, port, user, auth, config).await {
        Ok(manager) => match manager.create_worktree(repo, branch, &format!("./wt-{branch}")).await {
            Ok(rt) => Ok(serde_json::json!({
                "status": "created",
                "branch": branch,
                "name": rt.name,
                "host": rt.host,
                "remote_path": rt.remote_path.display().to_string(),
            })),
            Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
        },
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

pub async fn handle_port_forward(
    host: &str,
    port: u16,
    user: &str,
    target_host: &str,
    target_port: u16,
) -> Result<serde_json::Value> {
    let manager = SshManager::new();
    let auth = AuthMethod::Agent;
    match manager.connect(host, port, user, &auth).await {
        Ok(id) => Ok(serde_json::json!({
            "session_id": id,
            "status": "forwarding",
            "target": format!("{target_host}:{target_port}"),
        })),
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
