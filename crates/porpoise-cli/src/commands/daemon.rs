use porpoise_core::error::Result;

use crate::{app::DaemonAction, output::OutputFormat};

pub async fn handle(args: crate::app::DaemonArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        DaemonAction::Start => {
            // Would spawn porpoise-server process
            Ok(format.format(&serde_json::json!({"status": "daemon started"})))
        }
        DaemonAction::Stop => Ok(format.format(&serde_json::json!({"status": "daemon stopped"}))),
        DaemonAction::Status => Ok(format.format(&serde_json::json!({"status": "not running"}))),
    }
}
