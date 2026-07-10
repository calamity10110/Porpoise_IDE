use porpoise_core::error::Result;

use crate::{app::WorktreeAction, output::OutputFormat};

pub async fn handle(args: crate::app::WorktreeArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        WorktreeAction::Create {
            name,
            repo,
            agent,
            prompt,
        } => {
            // Would call RelayClient here in production
            Ok(format.format(&serde_json::json!({
                "id": "wt_placeholder",
                "name": name,
                "repo": repo,
                "agent": agent,
                "prompt": prompt,
                "status": "created"
            })))
        }
        WorktreeAction::List => Ok(format.format(&serde_json::json!({
            "worktrees": []
        }))),
        WorktreeAction::Show { name } => Ok(format.format(&serde_json::json!({
            "name": name,
            "status": "unknown",
            "message": "not yet connected to daemon"
        }))),
        WorktreeAction::Rm { name } => Ok(format!("worktree {name} removed")),
        WorktreeAction::Prune { dry_run } => {
            if dry_run {
                Ok("dry run: would prune 0 orphan worktrees".into())
            } else {
                Ok("pruned 0 orphan worktrees".into())
            }
        }
    }
}
