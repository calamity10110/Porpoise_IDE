use porpoise_browser::{BrowserEngine, HeadlessBrowser};
use porpoise_core::error::Result;

pub async fn handle_open(url: &str) -> Result<serde_json::Value> {
    let engine = HeadlessBrowser::new();
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

pub async fn handle_snapshot(url: &str) -> Result<serde_json::Value> {
    let engine = HeadlessBrowser::new();
    if !url.is_empty() {
        let _ = engine.navigate(url).await;
    }
    match engine.snapshot().await {
        Ok(bytes) => {
            let html = String::from_utf8_lossy(&bytes).to_string();
            Ok(serde_json::json!({
                "url": url,
                "html": html,
                "status": "snapshot_ok",
                "size": bytes.len(),
            }))
        }
        Err(e) => Ok(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}
