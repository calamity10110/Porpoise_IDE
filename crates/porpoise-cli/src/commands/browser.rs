use crate::app::BrowserAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::BrowserArgs, _format: &OutputFormat) -> Result<String> {
    match args.action {
        BrowserAction::Open { url } => Ok(format!("browser opened {url}")),
        BrowserAction::Snapshot { page_id } => Ok(format!("snapshot of page {page_id}")),
        BrowserAction::Click { page_id, selector } => {
            Ok(format!("clicked {selector} on page {page_id}"))
        }
        BrowserAction::Fill { page_id, selector, value } => {
            Ok(format!("filled {selector} with '{value}' on page {page_id}"))
        }
    }
}
