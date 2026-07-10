use porpoise_browser::{BrowserEngine, HeadlessBrowser};
use porpoise_core::error::Result;

use crate::{app::BrowserAction, output::OutputFormat};

pub async fn handle(args: crate::app::BrowserArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        BrowserAction::Open { url } => {
            let engine = HeadlessBrowser;
            let result = engine.navigate(&url).await?;
            Ok(format.format(&serde_json::json!({
                "url": result.url,
                "title": result.title,
                "status": format!("{:?}", result.status),
            })))
        }
        BrowserAction::Snapshot { page_id } => Ok(format!("[snapshot of page {page_id}]")),
        BrowserAction::Click { page_id, selector } => Ok(format!("[clicked {selector} on page {page_id}]")),
        BrowserAction::Fill {
            page_id,
            selector,
            value,
        } => Ok(format!("[filled {selector} with '{value}' on page {page_id}]")),
    }
}
