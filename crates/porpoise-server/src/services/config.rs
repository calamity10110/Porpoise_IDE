use porpoise_core::error::Result;
use porpoise_core::state::AppState;

pub async fn handle_get(_state: &AppState, _key: &str) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "value": null }))
}
