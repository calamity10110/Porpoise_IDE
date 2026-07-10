use porpoise_core::{error::Result, state::AppState, types::id::TerminalId};

pub async fn handle_create(_state: &AppState) -> Result<serde_json::Value> {
    let id = TerminalId::new();
    Ok(serde_json::json!({ "id": id.to_string(), "status": "created" }))
}
