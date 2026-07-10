use porpoise_core::error::Result;

use crate::{app::WorktreeAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::WorktreeArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        WorktreeAction::Create { name, repo, agent, prompt } => {
            let body = daemon::call("worktree_create", serde_json::json!({
                "name": name, "repo": repo, "agent": agent, "prompt": prompt
            })).await?;
            Ok(format.format(&body))
        }
        WorktreeAction::List => {
            let body = daemon::call("worktree_list", serde_json::json!({})).await?;
            Ok(format.format(&body))
        }
        WorktreeAction::Show { name } => {
            let body = daemon::call("worktree_list", serde_json::json!({})).await?;
            Ok(format.format(&serde_json::json!({ "name": name, "worktrees": body })))
        }
        WorktreeAction::Rm { name } => {
            daemon::call("worktree_rm", serde_json::json!({"name": name})).await?;
            Ok(format!("worktree {name} removed"))
        }
        WorktreeAction::Prune { dry_run } => {
            let _ = dry_run;
            daemon::call("worktree_prune", serde_json::json!({"dry_run": dry_run})).await?;
            Ok(format!("prune completed (dry_run={dry_run})"))
        }
    }
}
