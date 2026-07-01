use porpoise_core::error::Result;
use porpoise_core::state::AppState;
use porpoise_core::types::id::TerminalId;

pub async fn handle_create(_state: &AppState) -> Result<serde_json::Value> {
    let id = TerminalId::new();
    Ok(serde_json::json!({ "id": id.to_string(), "status": "created" }))
}
