use porpoise_core::error::Result;

use crate::{app::GitAction, output::OutputFormat};

pub async fn handle(args: crate::app::GitArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        GitAction::Status { repo_path } => Ok(format.format(&serde_json::json!({
            "repo": repo_path,
            "branch": "main",
            "changes": []
        }))),
        GitAction::Diff { repo_path, staged } => Ok(format!("[diff for {repo_path} staged={staged}]")),
        GitAction::Log { repo_path, count: _ } => Ok(format!("[git log for {repo_path}]")),
        GitAction::Clone { url, path } => Ok(format!("cloned {url} to {path}")),
        GitAction::Branch { repo_path } => Ok(format!("[branches for {repo_path}]")),
    }
}
