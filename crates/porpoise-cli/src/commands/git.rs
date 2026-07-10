use porpoise_core::error::Result;

use crate::{app::GitAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::GitArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        GitAction::Status { repo_path } => {
            let body = daemon::call("git_status", serde_json::json!({"path": repo_path})).await?;
            Ok(format.format(&body))
        }
        GitAction::Diff { repo_path, staged } => {
            let body = daemon::call("git_diff", serde_json::json!({"path": repo_path, "staged": staged})).await?;
            Ok(format!("{body}"))
        }
        GitAction::Log { repo_path, count } => {
            let body = daemon::call("git_log", serde_json::json!({"path": repo_path, "count": count})).await?;
            Ok(format!("{body}"))
        }
        GitAction::Clone { url, path } => {
            let body = daemon::call("git_clone", serde_json::json!({"url": url, "path": path})).await?;
            Ok(format.format(&body))
        }
        GitAction::Branch { repo_path } => {
            let body = daemon::call("git_branch", serde_json::json!({"path": repo_path})).await?;
            Ok(format!("{body}"))
        }
    }
}
