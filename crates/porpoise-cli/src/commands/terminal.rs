use porpoise_core::error::Result;

use crate::{app::TerminalAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::TerminalArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        TerminalAction::Create { worktree, shell } => {
            let body = daemon::call("terminal_create", serde_json::json!({
                "worktree_id": worktree, "shell": shell
            })).await?;
            Ok(format.format(&body))
        }
        TerminalAction::List { worktree } => {
            let body = daemon::call("terminal_list", serde_json::json!({
                "worktree_id": worktree
            })).await?;
            Ok(format.format(&body))
        }
        TerminalAction::Send { id, text, enter } => {
            let data = if enter { format!("{text}\n") } else { text };
            daemon::call("terminal_send", serde_json::json!({"id": id, "data": data})).await?;
            Ok(format!("sent to {id}"))
        }
        TerminalAction::Read { id } => {
            let body = daemon::call("terminal_read", serde_json::json!({"id": id, "max_bytes": 4096})).await?;
            Ok(format!("{body}"))
        }
        TerminalAction::Resize { id, rows, cols } => {
            daemon::call("terminal_resize", serde_json::json!({"id": id, "rows": rows, "cols": cols})).await?;
            Ok(format!("resized {id} to {rows}x{cols}"))
        }
        TerminalAction::Close { id } => {
            daemon::call("terminal_close", serde_json::json!({"id": id})).await?;
            Ok(format!("terminal {id} closed"))
        }
    }
}
