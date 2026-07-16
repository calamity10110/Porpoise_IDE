use porpoise_core::{error::Result, state::AppState};

pub async fn handle_get(state: &AppState, key: &str) -> Result<serde_json::Value> {
    match key {
        "app" | "general" => {
            let config = state.config().await;
            Ok(serde_json::json!({
                "theme": "dark",
                "default_agent": "claude",
                "max_concurrent_agents": config.agent.max_concurrent_agents,
                "data_dir": config.core.data_dir.as_ref().map(|d| d.display().to_string()),
                "event_bus_capacity": config.core.event_bus_capacity,
            }))
        }
        "agent" => {
            let config = state.config().await;
            Ok(serde_json::json!({
                "max_concurrent_agents": config.agent.max_concurrent_agents,
            }))
        }
        "core" => {
            let config = state.config().await;
            Ok(serde_json::json!({
                "data_dir": config.core.data_dir.as_ref().map(|d| d.display().to_string()),
            }))
        }
        _ => Ok(serde_json::json!({ "value": null, "key": key })),
    }
}

pub async fn handle_set(_state: &AppState, key: &str, _value: serde_json::Value) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "key": key, "status": "set" }))
}

pub async fn handle_list(state: &AppState) -> Result<serde_json::Value> {
    let config = state.config().await;
    Ok(serde_json::json!({
        "keys": ["app", "agent", "core"],
        "max_concurrent_agents": config.agent.max_concurrent_agents,
        "data_dir": config.core.data_dir.as_ref().map(|d| d.display().to_string()),
    }))
}
