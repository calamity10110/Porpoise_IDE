use porpoise_core::error::Result;

use crate::{app::TerminalAction, output::OutputFormat};

pub async fn handle(args: crate::app::TerminalArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        TerminalAction::Create { worktree, shell } => Ok(format.format(&serde_json::json!({
            "id": "tm_placeholder",
            "worktree": worktree,
            "shell": shell.unwrap_or_else(|| "bash".into())
        }))),
        TerminalAction::List { worktree } => Ok(format.format(&serde_json::json!({
            "worktree": worktree,
            "terminals": []
        }))),
        TerminalAction::Send { id, text, enter } => {
            let sent = if enter { format!("{text}\n") } else { text };
            Ok(format!("sent to {id}: {sent}"))
        }
        TerminalAction::Read { id } => Ok(format!("[terminal {id} output]")),
        TerminalAction::Resize { id, rows, cols } => Ok(format!("resized {id} to {rows}x{cols}")),
        TerminalAction::Close { id } => Ok(format!("terminal {id} closed")),
    }
}
