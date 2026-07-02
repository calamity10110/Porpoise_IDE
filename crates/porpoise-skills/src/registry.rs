use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

pub struct SkillRegistry {
    skills: HashMap<String, SkillManifest>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, id: String, manifest: SkillManifest) {
        self.skills.insert(id, manifest);
    }

    pub fn list(&self) -> Vec<&SkillManifest> {
        self.skills.values().collect()
    }

    pub fn enable(&mut self, id: &str) {
        if let Some(skill) = self.skills.get_mut(id) {
            skill.enabled = true;
        }
    }

    pub fn disable(&mut self, id: &str) {
        if let Some(skill) = self.skills.get_mut(id) {
            skill.enabled = false;
        }
    }

    pub fn uninstall(&mut self, id: &str) {
        self.skills.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&SkillManifest> {
        self.skills.get(id)
    }
}
