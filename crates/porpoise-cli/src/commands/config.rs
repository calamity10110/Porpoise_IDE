use porpoise_core::error::Result;

use crate::{app::ConfigAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::ConfigArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        ConfigAction::Get { key } => {
            let body = daemon::call("config_get", serde_json::json!({"key": key})).await?;
            Ok(format.format(&body))
        }
        ConfigAction::Set { key, value } => {
            daemon::call("config_set", serde_json::json!({"key": key, "value": value})).await?;
            Ok(format!("set {key} = {value}"))
        }
        ConfigAction::List => {
            let body = daemon::call("config_list", serde_json::json!({})).await?;
            Ok(format.format(&body))
        }
        ConfigAction::Edit => Ok("edit config via PORPOISE_CONFIG file".into()),
    }
}
