use porpoise_core::error::Result;

use crate::{app::DaemonAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::DaemonArgs, _format: &OutputFormat) -> Result<String> {
    match args.action {
        DaemonAction::Start => {
            let body = daemon::call("daemon_start", serde_json::json!({})).await?;
            Ok(format!("daemon: {body}"))
        }
        DaemonAction::Stop => {
            daemon::call("daemon_stop", serde_json::json!({})).await?;
            Ok("daemon stopped".into())
        }
        DaemonAction::Status => {
            let body = daemon::call("health", serde_json::json!({})).await?;
            Ok(format!("daemon status: {body}"))
        }
    }
}
