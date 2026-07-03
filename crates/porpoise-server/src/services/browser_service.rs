use porpoise_core::error::Result;
use porpoise_browser::{HeadlessBrowser, BrowserEngine};

pub async fn handle_open(url: &str) -> Result<serde_json::Value> {
    let engine = HeadlessBrowser;
    match engine.navigate(url).await {
        Ok(result) => Ok(serde_json::json!({
            "url": result.url,
            "title": result.title,
            "page_id": result.page_id,
            "status": "loaded",
        })),
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
