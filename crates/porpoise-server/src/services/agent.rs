use porpoise_core::error::Result;

pub async fn handle_list() -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "agents": [] }))
}

pub async fn handle_detect() -> Result<serde_json::Value> {
    let manifests = porpoise_agent::AgentDetector::detect_all();
    let list: Vec<serde_json::Value> = manifests
        .iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "binary": m.binary,
                "kind": m.kind.to_string(),
                "detected": m.detected,
                "version": m.version,
            })
        })
        .collect();
    Ok(serde_json::json!({ "agents": list }))
}
