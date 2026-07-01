use crate::app::ConfigAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::ConfigArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        ConfigAction::Get { key } => Ok(format!("{key} = (not set)")),
        ConfigAction::Set { key, value } => Ok(format!("{key} = {value}")),
        ConfigAction::List => {
            Ok(format.format(&serde_json::json!({"entries": {}})))
        }
        ConfigAction::Edit => Ok("opening config in editor...".into()),
    }
}
