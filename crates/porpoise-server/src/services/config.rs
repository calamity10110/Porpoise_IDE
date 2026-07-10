use porpoise_core::{error::Result, state::AppState};

pub async fn handle_get(_state: &AppState, _key: &str) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "value": null }))
}
