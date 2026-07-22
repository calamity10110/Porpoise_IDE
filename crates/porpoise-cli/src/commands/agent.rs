use porpoise_core::error::Result;

use crate::{app::AgentAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::AgentArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        AgentAction::List => {
            let body = daemon::call("agent_list", serde_json::json!({})).await?;
            Ok(format.format(&body))
        }
        AgentAction::Run { kind, worktree, prompt } => {
            let body = daemon::call(
                "agent_run",
                serde_json::json!({
                    "kind": kind, "worktree": worktree, "prompt": prompt
                }),
            )
            .await?;
            Ok(format!("started agent: {body}"))
        }
        AgentAction::Stop { id } => {
            daemon::call("agent_stop", serde_json::json!({"id": id})).await?;
            Ok(format!("agent {id} stopped"))
        }
        AgentAction::Logs { id, lines } => {
            let body = daemon::call("agent_logs", serde_json::json!({"id": id, "lines": lines})).await?;
            Ok(format!("{body}"))
        }
    }
}
