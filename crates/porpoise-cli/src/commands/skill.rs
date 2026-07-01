use crate::app::SkillAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::SkillArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        SkillAction::Search { query } => Ok(format!("searching skills for '{query}'...")),
        SkillAction::Install { name } => Ok(format!("installing skill '{name}'...")),
        SkillAction::Uninstall { name } => Ok(format!("uninstalling skill '{name}'...")),
        SkillAction::List => {
            Ok(format.format(&serde_json::json!({"skills": []})))
        }
    }
}
