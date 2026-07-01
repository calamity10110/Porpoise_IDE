use crate::app::AgentAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::AgentArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        AgentAction::List => {
            Ok(format.format(&serde_json::json!({"agents": []})))
        }
        AgentAction::Run { kind, worktree, prompt } => {
            Ok(format.format(&serde_json::json!({
                "kind": kind,
                "worktree": worktree,
                "prompt": prompt,
                "status": "launched"
            })))
        }
        AgentAction::Stop { id } => {
            Ok(format!("agent {id} stopped"))
        }
        AgentAction::Logs { id, lines } => {
            let count = lines.unwrap_or(50);
            Ok(format!("[agent {id} last {count} lines]"))
        }
    }
}
