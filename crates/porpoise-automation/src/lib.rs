use std::collections::HashMap;
use std::sync::Arc;

use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDef {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<StepDef>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepDef {
    AgentCall {
        id: String,
        agent_kind: String,
        prompt: String,
        depends_on: Vec<String>,
        timeout_secs: Option<u64>,
    },
    WebAction {
        id: String,
        url: String,
        action: WebActionType,
        depends_on: Vec<String>,
    },
    ComputerAction {
        id: String,
        action: ComputerActionType,
        depends_on: Vec<String>,
    },
    ApiCall {
        id: String,
        url: String,
        method: String,
        headers: HashMap<String, String>,
        body: Option<String>,
        depends_on: Vec<String>,
    },
    CredentialLookup {
        id: String,
        credential_name: String,
        output_var: String,
        depends_on: Vec<String>,
    },
    Delay {
        id: String,
        duration_secs: u64,
        depends_on: Vec<String>,
    },
    Condition {
        id: String,
        expression: String,
        if_true: Vec<StepDef>,
        if_false: Vec<StepDef>,
        depends_on: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebActionType {
    Navigate,
    Click { selector: String },
    Fill { selector: String, value: String },
    Select { selector: String, option: String },
    Extract { selector: String, attribute: Option<String> },
    Screenshot,
    WaitForSelector { selector: String, timeout_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComputerActionType {
    Click { x: i32, y: i32 },
    Type { text: String },
    KeyPress { key: String },
    Screenshot,
    Scroll { x: i32, y: i32, delta_x: i32, delta_y: i32 },
    Wait { duration_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowState {
    pub step_results: HashMap<String, serde_json::Value>,
    pub variables: HashMap<String, String>,
    pub errors: HashMap<String, String>,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum WorkflowStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed(String),
}

pub struct WorkflowEngine {
    workflows: Arc<RwLock<HashMap<String, WorkflowState>>>,
}

impl Default for WorkflowEngine {
    fn default() -> Self { Self::new() }
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self { workflows: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn register(&self, def: WorkflowDef) -> Result<String> {
        let id = def.id.clone();
        let state = WorkflowState::default();
        self.workflows.write().await.insert(id.clone(), state);
        Ok(id)
    }

    pub async fn execute(&self, def: &WorkflowDef) -> Result<WorkflowState> {
        let mut state = WorkflowState { status: WorkflowStatus::Running, ..Default::default() };

        let ordered = self.topological_sort(def)?;
        for step_id in &ordered {
            let step = def.steps.iter().find(|s| s.id() == *step_id)
                .ok_or_else(|| PorpoiseError::Internal(format!("step {step_id} not found")))?;

            match self.execute_step(step, &mut state, def).await {
                Ok(result) => {
                    state.step_results.insert(step_id.clone(), result);
                }
                Err(e) => {
                    state.errors.insert(step_id.clone(), e.to_string());
                    state.status = WorkflowStatus::Failed(e.to_string());
                    return Ok(state);
                }
            }
        }

        state.status = WorkflowStatus::Completed;
        Ok(state)
    }

    async fn execute_step(&self, step: &StepDef, state: &mut WorkflowState, _def: &WorkflowDef) -> Result<serde_json::Value> {
        match step {
            StepDef::AgentCall { id, agent_kind, prompt, .. } => {
                let resolved = resolve_template(prompt, &state.variables);
                let _ = (agent_kind, id);
                Ok(serde_json::json!({ "step": id, "agent": agent_kind, "prompt": resolved }))
            }
            StepDef::WebAction { id, url, action, .. } => {
                let _ = (url, action);
                Ok(serde_json::json!({ "step": id, "url": url, "action": format!("{action:?}") }))
            }
            StepDef::ComputerAction { id, action, .. } => {
                let _ = action;
                Ok(serde_json::json!({ "step": id, "computer_action": format!("{action:?}") }))
            }
            StepDef::ApiCall { id, url, method, body, .. } => {
                let _ = (url, method, body);
                Ok(serde_json::json!({ "step": id, "called": url }))
            }
            StepDef::CredentialLookup { id, credential_name, output_var, .. } => {
                state.variables.insert(output_var.clone(), format!("<cred:{credential_name}>"));
                Ok(serde_json::json!({ "step": id, "lookup": credential_name }))
            }
            StepDef::Delay { id, duration_secs, .. } => {
                tokio::time::sleep(std::time::Duration::from_secs(*duration_secs)).await;
                Ok(serde_json::json!({ "step": id, "delayed_ms": duration_secs * 1000 }))
            }
            StepDef::Condition { id, expression, .. } => {
                let _ = expression;
                Ok(serde_json::json!({ "step": id, "condition": expression }))
            }
        }
    }

    pub fn load_yaml(yaml: &str) -> Result<WorkflowDef> {
        serde_yaml::from_str(yaml)
            .map_err(|e| PorpoiseError::Config(format!("workflow parse: {e}")))
    }

    fn topological_sort(&self, def: &WorkflowDef) -> Result<Vec<String>> {
        let mut visited = HashMap::new();
        let mut result = Vec::new();

        fn visit(
            id: &str,
            def: &WorkflowDef,
            visited: &mut HashMap<String, bool>,
            result: &mut Vec<String>,
        ) -> Result<()> {
            match visited.get(id) {
                Some(&true) => Ok(()),
                Some(&false) => Err(PorpoiseError::Internal(format!("cycle detected at {id}"))),
                None => {
                    visited.insert(id.to_string(), false);
                    if let Some(step) = def.steps.iter().find(|s| s.id() == id) {
                        for dep in step.depends_on() {
                            visit(dep, def, visited, result)?;
                        }
                    }
                    visited.insert(id.to_string(), true);
                    result.push(id.to_string());
                    Ok(())
                }
            }
        }

        for step in &def.steps {
            visit(step.id(), def, &mut visited, &mut result)?;
        }

        Ok(result)
    }
}

fn resolve_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("${{{k}}}"), v);
    }
    result
}

impl StepDef {
    pub fn id(&self) -> &str {
        match self {
            StepDef::AgentCall { id, .. } => id,
            StepDef::WebAction { id, .. } => id,
            StepDef::ComputerAction { id, .. } => id,
            StepDef::ApiCall { id, .. } => id,
            StepDef::CredentialLookup { id, .. } => id,
            StepDef::Delay { id, .. } => id,
            StepDef::Condition { id, .. } => id,
        }
    }

    pub fn depends_on(&self) -> &[String] {
        match self {
            StepDef::AgentCall { depends_on, .. } => depends_on,
            StepDef::WebAction { depends_on, .. } => depends_on,
            StepDef::ComputerAction { depends_on, .. } => depends_on,
            StepDef::ApiCall { depends_on, .. } => depends_on,
            StepDef::CredentialLookup { depends_on, .. } => depends_on,
            StepDef::Delay { depends_on, .. } => depends_on,
            StepDef::Condition { depends_on, .. } => depends_on,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_workflow() -> WorkflowDef {
        WorkflowDef {
            id: "wf-test".into(),
            name: "Test Workflow".into(),
            description: None,
            env: HashMap::new(),
            steps: vec![
                StepDef::Delay { id: "step-1".into(), duration_secs: 0, depends_on: vec![] },
                StepDef::AgentCall {
                    id: "step-2".into(), agent_kind: "claude".into(),
                    prompt: "Do something".into(), depends_on: vec!["step-1".into()],
                    timeout_secs: None,
                },
                StepDef::WebAction {
                    id: "step-3".into(), url: "https://example.com".into(),
                    action: WebActionType::Navigate, depends_on: vec!["step-2".into()],
                },
            ],
        }
    }

    #[test]
    fn test_load_yaml() {
        let yaml = r#"
id: wf-test
name: Test
env: {}
steps:
  - type: Delay
    id: step-1
    duration_secs: 0
    depends_on: []
  - type: AgentCall
    id: step-2
    agent_kind: claude
    prompt: "Hello"
    depends_on: [step-1]
"#;
        let wf = WorkflowEngine::load_yaml(yaml).unwrap();
        assert_eq!(wf.steps.len(), 2);
    }

    #[test]
    fn test_topological_sort() {
        let wf = sample_workflow();
        let engine = WorkflowEngine::new();
        let order = engine.topological_sort(&wf).unwrap();
        assert_eq!(order, vec!["step-1", "step-2", "step-3"]);
    }

    #[tokio::test]
    async fn test_execute() {
        let wf = sample_workflow();
        let engine = WorkflowEngine::new();
        let state = engine.execute(&wf).await.unwrap();
        assert_eq!(state.status, WorkflowStatus::Completed);
        assert!(state.step_results.contains_key("step-1"));
        assert!(state.step_results.contains_key("step-2"));
        assert!(state.step_results.contains_key("step-3"));
    }

    #[test]
    fn test_detect_cycle() {
        let wf = WorkflowDef {
            id: "wf-cycle".into(),
            name: "Cycle".into(),
            description: None,
            env: HashMap::new(),
            steps: vec![
                StepDef::Delay { id: "a".into(), duration_secs: 0, depends_on: vec!["b".into()] },
                StepDef::Delay { id: "b".into(), duration_secs: 0, depends_on: vec!["a".into()] },
            ],
        };
        let engine = WorkflowEngine::new();
        let result = engine.topological_sort(&wf);
        assert!(result.is_err());
    }
}
