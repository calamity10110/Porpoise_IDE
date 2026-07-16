use porpoise_core::error::Result;
use porpoise_skills::SkillRegistry;

pub async fn handle_list() -> Result<serde_json::Value> {
    let registry = SkillRegistry::new();
    let skills: Vec<serde_json::Value> = registry
        .list()
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "version": s.version,
                "enabled": s.enabled,
            })
        })
        .collect();
    Ok(serde_json::json!({ "skills": skills }))
}

pub async fn handle_search(query: &str) -> Result<serde_json::Value> {
    let registry = SkillRegistry::new();
    let results: Vec<serde_json::Value> = registry
        .list()
        .iter()
        .filter(|s| {
            let q = query.to_lowercase();
            s.name.to_lowercase().contains(&q) || s.id.to_lowercase().contains(&q)
        })
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "name": s.name,
                "version": s.version,
                "enabled": s.enabled,
            })
        })
        .collect();
    Ok(serde_json::json!({ "results": results, "query": query }))
}

pub async fn handle_install(id: &str) -> Result<serde_json::Value> {
    Ok(serde_json::json!({ "id": id, "status": "install_simulated" }))
}

pub async fn handle_uninstall(id: &str) -> Result<serde_json::Value> {
    let mut registry = SkillRegistry::new();
    registry.uninstall(id);
    Ok(serde_json::json!({ "id": id, "status": "uninstalled" }))
}
