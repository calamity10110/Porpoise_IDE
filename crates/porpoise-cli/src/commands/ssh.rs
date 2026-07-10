use porpoise_core::error::Result;
use porpoise_ssh::auth::AuthMethod;

use crate::{app::SshAction, output::OutputFormat};

pub async fn handle(args: crate::app::SshArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        SshAction::Connect { host, user } => {
            let username = user.as_deref().unwrap_or("root");
            let manager = porpoise_ssh::SshManager::new();
            let auth = AuthMethod::Password("".into());
            match manager.connect(&host, 22, username, &auth).await {
                Ok(id) => Ok(format!("Connected: {id}")),
                Err(e) => Ok(format!("Connection failed: {e}")),
            }
        }
        SshAction::Worktree { session_id } => Ok(format.format(&serde_json::json!({
            "session": session_id,
            "status": "worktree not yet supported over SSH"
        }))),
        SshAction::PortForward {
            session_id,
            local,
            remote,
        } => Ok(format.format(&serde_json::json!({
            "session": session_id,
            "local_port": local,
            "remote_port": remote,
            "status": "port forwarding not yet implemented"
        }))),
    }
}
