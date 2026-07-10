use porpoise_core::error::Result;
use porpoise_ssh::{SshManager, auth::AuthMethod};

pub async fn handle_connect(host: &str, port: u16, user: &str) -> Result<serde_json::Value> {
    let manager = SshManager::new();
    let auth = AuthMethod::Password("".into());
    match manager.connect(host, port, user, &auth).await {
        Ok(id) => Ok(serde_json::json!({ "session_id": id, "status": "connected" })),
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
