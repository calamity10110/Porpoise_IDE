use porpoise_core::error::Result;

use crate::{app::SshAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::SshArgs, _format: &OutputFormat) -> Result<String> {
    match args.action {
        SshAction::Connect { host, user } => {
            let body = daemon::call("ssh_connect", serde_json::json!({"host": host, "user": user})).await?;
            Ok(format!("connected: {body}"))
        }
        SshAction::Worktree { session_id } => {
            let body = daemon::call("ssh_worktree", serde_json::json!({"session_id": session_id})).await?;
            Ok(format!("{body}"))
        }
        SshAction::PortForward {
            session_id,
            local,
            remote,
        } => {
            let body = daemon::call(
                "ssh_port_forward",
                serde_json::json!({
                    "session_id": session_id, "local": local, "remote": remote
                }),
            )
            .await?;
            Ok(format!("{body}"))
        }
    }
}
