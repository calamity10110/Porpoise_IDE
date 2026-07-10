use std::{collections::HashMap, path::Path, sync::Arc};

use porpoise_core::{
    error::{PorpoiseError, Result},
    types::id::AgentId,
};
use tokio::sync::RwLock;

use crate::{
    generic::GenericAgent,
    traits::{Agent, AgentHandle},
    types::{AgentInfo, AgentKind, AgentStatus},
};

struct AgentEntry {
    #[allow(dead_code)]
    agent: Box<dyn Agent>,
    handle: Option<Box<dyn AgentHandle>>,
    info: AgentInfo,
}

pub struct AgentPool {
    agents: Arc<RwLock<HashMap<AgentId, AgentEntry>>>,
    max_size: usize,
}

impl AgentPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    pub async fn spawn(&self, kind: AgentKind, worktree: &Path) -> Result<AgentId> {
        let count = self.agents.read().await.len();
        if count >= self.max_size {
            return Err(PorpoiseError::ResourceLimit(format!(
                "max agents ({}) reached",
                self.max_size
            )));
        }
        let agent: Box<dyn Agent> = match &kind {
            AgentKind::ClaudeCode => Box::new(GenericAgent::with_kind("claude", AgentKind::ClaudeCode)),
            AgentKind::Codex => Box::new(GenericAgent::with_kind("codex", AgentKind::Codex)),
            AgentKind::Gemini => Box::new(GenericAgent::with_kind("gemini", AgentKind::Gemini)),
            AgentKind::OpenCode => Box::new(GenericAgent::with_kind("opencode", AgentKind::OpenCode)),
            AgentKind::ZAI => Box::new(GenericAgent::with_kind("z", AgentKind::ZAI)),
            AgentKind::OpenAI => Box::new(GenericAgent::with_kind("openai", AgentKind::OpenAI)),
            AgentKind::Grok => Box::new(GenericAgent::with_kind("grok", AgentKind::Grok)),
            AgentKind::OpenRouter => Box::new(GenericAgent::with_kind("openrouter", AgentKind::OpenRouter)),
            AgentKind::Custom(name) => Box::new(GenericAgent::new(name)),
        };
        let handle = agent.spawn(worktree).await?;
        let pid = handle.pid();
        let id = AgentId::new();
        let info = AgentInfo {
            id,
            kind,
            pid,
            status: AgentStatus::Running,
            worktree_path: Some(worktree.to_path_buf()),
        };
        self.agents.write().await.insert(
            id,
            AgentEntry {
                agent,
                handle: Some(handle),
                info,
            },
        );
        Ok(id)
    }

    pub async fn list(&self) -> Vec<AgentInfo> {
        self.agents.read().await.values().map(|e| e.info.clone()).collect()
    }

    pub async fn shutdown(&self, id: AgentId) -> Result<()> {
        let mut agents = self.agents.write().await;
        if let Some(mut entry) = agents.remove(&id)
            && let Some(mut handle) = entry.handle.take()
        {
            handle.shutdown().await?;
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) -> Result<()> {
        let ids: Vec<AgentId> = self.agents.read().await.keys().copied().collect();
        for id in ids {
            self.shutdown(id).await.ok();
        }
        Ok(())
    }
}
