use porpoise_core::error::Result;

use crate::{app::BrowserAction, daemon, output::OutputFormat};

pub async fn handle(args: crate::app::BrowserArgs, _format: &OutputFormat) -> Result<String> {
    match args.action {
        BrowserAction::Open { url } => {
            let body = daemon::call("browser_open", serde_json::json!({"url": url})).await?;
            Ok(format!("opened: {body}"))
        }
        BrowserAction::Snapshot { page_id } => {
            let body = daemon::call("browser_snapshot", serde_json::json!({"page_id": page_id})).await?;
            Ok(format!("{body}"))
        }
        BrowserAction::Click { page_id, selector } => {
            let _ = page_id; let _ = selector;
            Ok("clicked".into())
        }
        BrowserAction::Fill { page_id, selector, value } => {
            let _ = page_id; let _ = selector; let _ = value;
            Ok("filled".into())
        }
    }
}
