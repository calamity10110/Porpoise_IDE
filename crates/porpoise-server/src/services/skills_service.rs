use porpoise_core::error::Result;

pub async fn handle_list() -> Result<serde_json::Value> {
    let registry = porpoise_skills::SkillRegistry::new();
    let skills: Vec<serde_json::Value> = registry.list().iter().map(|s| {
        serde_json::json!({
            "id": s.id,
            "name": s.name,
            "version": s.version,
            "enabled": s.enabled,
        })
    }).collect();
    Ok(serde_json::json!({ "skills": skills }))
}
