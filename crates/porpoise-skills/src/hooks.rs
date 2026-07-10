use std::{collections::HashMap, sync::Arc};

use porpoise_core::{
    error::Result,
    types::id::{AgentId, TerminalId},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Hook types that plugins can subscribe to.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    OnAgentStart,
    OnAgentOutput,
    OnAgentExit,
    OnTerminalCreate,
    OnTerminalOutput,
    OnTerminalClose,
    OnWorktreeCreate,
    OnWorktreeDelete,
    OnConfigReload,
}

/// Context passed to a hook callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub hook_type: HookType,
    pub agent_id: Option<String>,
    pub terminal_id: Option<String>,
    pub worktree_path: Option<String>,
    pub payload: serde_json::Value,
}

/// Result of running hooks for an event.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookResults {
    pub executed: Vec<String>,
    pub errors: Vec<String>,
}

/// A registered hook callback function.
pub type HookCallback = Arc<dyn Fn(&HookContext) -> Result<()> + Send + Sync>;

type HookEntry = (String, HookCallback);
type HookMap = HashMap<HookType, Vec<HookEntry>>;

pub struct HookRegistry {
    hooks: Arc<RwLock<HookMap>>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, hook_type: HookType, skill_id: &str, callback: HookCallback) {
        let mut hooks = self.hooks.write().await;
        hooks
            .entry(hook_type)
            .or_default()
            .push((skill_id.to_string(), callback));
    }

    pub async fn unregister_skill(&self, skill_id: &str) {
        let mut hooks = self.hooks.write().await;
        for callbacks in hooks.values_mut() {
            callbacks.retain(|(id, _)| id != skill_id);
        }
    }

    pub async fn fire(&self, ctx: &HookContext) -> HookResults {
        let hooks = self.hooks.read().await;
        let callbacks = hooks.get(&ctx.hook_type);

        let mut results = HookResults::default();

        if let Some(callbacks) = callbacks {
            for (skill_id, callback) in callbacks {
                results.executed.push(skill_id.clone());
                if let Err(e) = callback(ctx) {
                    results.errors.push(format!("{skill_id}: {e}"));
                }
            }
        }

        results
    }

    pub async fn list_hooks(&self) -> Vec<(HookType, Vec<String>)> {
        let hooks = self.hooks.read().await;
        hooks
            .iter()
            .map(|(ht, callbacks)| (ht.clone(), callbacks.iter().map(|(id, _)| id.clone()).collect()))
            .collect()
    }

    pub async fn clear(&self) {
        self.hooks.write().await.clear();
    }
}

/// Convenience functions for firing common hook types.
impl HookRegistry {
    pub async fn fire_agent_start(&self, agent_id: AgentId, worktree_path: &str) -> HookResults {
        self.fire(&HookContext {
            hook_type: HookType::OnAgentStart,
            agent_id: Some(agent_id.to_string()),
            terminal_id: None,
            worktree_path: Some(worktree_path.to_string()),
            payload: serde_json::json!({}),
        })
        .await
    }

    pub async fn fire_agent_output(&self, agent_id: AgentId, output: &str) -> HookResults {
        self.fire(&HookContext {
            hook_type: HookType::OnAgentOutput,
            agent_id: Some(agent_id.to_string()),
            terminal_id: None,
            worktree_path: None,
            payload: serde_json::json!({ "output": output }),
        })
        .await
    }

    pub async fn fire_terminal_create(&self, terminal_id: TerminalId) -> HookResults {
        self.fire(&HookContext {
            hook_type: HookType::OnTerminalCreate,
            agent_id: None,
            terminal_id: Some(terminal_id.to_string()),
            worktree_path: None,
            payload: serde_json::json!({}),
        })
        .await
    }

    pub async fn fire_terminal_output(&self, terminal_id: TerminalId, data: &[u8]) -> HookResults {
        let text = String::from_utf8_lossy(data).to_string();
        self.fire(&HookContext {
            hook_type: HookType::OnTerminalOutput,
            agent_id: None,
            terminal_id: Some(terminal_id.to_string()),
            worktree_path: None,
            payload: serde_json::json!({ "data": text }),
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use porpoise_core::PorpoiseError;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[tokio::test]
    async fn test_register_and_fire() {
        let registry = HookRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        registry
            .register(
                HookType::OnAgentStart,
                "test-skill",
                Arc::new(move |_ctx| {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
            )
            .await;

        let results = registry.fire_agent_start(AgentId::new(), "/tmp/wt").await;

        assert_eq!(results.executed.len(), 1);
        assert_eq!(results.errors.len(), 0);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = HookRegistry::new();

        registry
            .register(HookType::OnTerminalCreate, "skill-a", Arc::new(|_ctx| Ok(())))
            .await;
        registry
            .register(HookType::OnTerminalCreate, "skill-b", Arc::new(|_ctx| Ok(())))
            .await;

        let hooks = registry.list_hooks().await;
        assert_eq!(hooks[0].1.len(), 2);

        registry.unregister_skill("skill-a").await;

        let hooks = registry.list_hooks().await;
        assert_eq!(hooks[0].1.len(), 1);
    }

    #[tokio::test]
    async fn test_hook_error_collection() {
        let registry = HookRegistry::new();

        registry
            .register(HookType::OnAgentOutput, "good-skill", Arc::new(|_ctx| Ok(())))
            .await;
        registry
            .register(
                HookType::OnAgentOutput,
                "bad-skill",
                Arc::new(|_ctx| Err(PorpoiseError::Plugin("boom".into()))),
            )
            .await;

        let results = registry.fire_agent_output(AgentId::new(), "hello").await;

        assert_eq!(results.executed.len(), 2);
        assert_eq!(results.errors.len(), 1);
        assert!(results.errors[0].contains("bad-skill"));
    }
}
