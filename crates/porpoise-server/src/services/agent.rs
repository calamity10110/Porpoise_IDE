use porpoise_core::error::Result;
use porpoise_core::state::AppState;

pub async fn handle_list(_state: &AppState) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "agents": [] }))
}
