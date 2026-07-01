use crate::app::SshAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::SshArgs, _format: &OutputFormat) -> Result<String> {
    match args.action {
        SshAction::Connect { host, user } => {
            let user_str = user.as_deref().unwrap_or("root");
            Ok(format!("connecting to {user_str}@{host}..."))
        }
        SshAction::Worktree { session_id } => {
            Ok(format!("creating worktree on session {session_id}"))
        }
        SshAction::PortForward { session_id, local, remote } => {
            Ok(format!("forwarding localhost:{local} to remote:{remote} via {session_id}"))
        }
    }
}
