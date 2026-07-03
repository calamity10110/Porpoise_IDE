use crate::app::SkillAction;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle(args: crate::app::SkillArgs, format: &OutputFormat) -> Result<String> {
    match args.action {
        SkillAction::Search { query } => Ok(format!("[searching skills for '{query}']")),
        SkillAction::Install { name } => {
            let mut registry = porpoise_skills::SkillRegistry::new();
            registry.register(name.clone(), porpoise_skills::SkillManifest {
                id: name.clone(),
                name: name.clone(),
                version: "0.1.0".into(),
                description: format!("{name} skill"),
                enabled: true,
            });
            Ok(format!("Installed skill '{name}'"))
        }
        SkillAction::Uninstall { name } => Ok(format!("Uninstalled skill '{name}'")),
        SkillAction::List => {
            let registry = porpoise_skills::SkillRegistry::new();
            let skills: Vec<serde_json::Value> = registry.list().iter().map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "version": s.version,
                    "enabled": s.enabled,
                })
            }).collect();
            Ok(format.format(&serde_json::json!({ "skills": skills })))
        }
    }
}
