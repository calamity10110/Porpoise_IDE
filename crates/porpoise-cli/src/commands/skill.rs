use porpoise_core::error::Result;

use crate::{app::SkillAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::SkillArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        SkillAction::Search { query } => {
            let body = daemon::call("skill_search", serde_json::json!({"query": query})).await?;
            Ok(format.format(&body))
        }
        SkillAction::Install { name } => {
            daemon::call("skill_install", serde_json::json!({"name": name})).await?;
            Ok(format!("Installed skill '{name}'"))
        }
        SkillAction::Uninstall { name } => {
            daemon::call("skill_uninstall", serde_json::json!({"name": name})).await?;
            Ok(format!("Uninstalled skill '{name}'"))
        }
        SkillAction::List => {
            let body = daemon::call("skill_list", serde_json::json!({})).await?;
            Ok(format.format(&body))
        }
    }
}
