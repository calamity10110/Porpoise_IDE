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
    idle_timeout_secs: u64,
}

impl AgentPool {
    pub fn new(max_size: usize) -> Self {
        Self::with_idle_timeout(max_size, 900)
    }

    pub fn with_idle_timeout(max_size: usize, idle_timeout_secs: u64) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            idle_timeout_secs,
        }
    }

    pub async fn spawn(&self, kind: AgentKind, worktree: &Path) -> Result<AgentId> {
        // Try LRU eviction if pool is full
        let count = self.agents.read().await.len();
        if count >= self.max_size {
            let oldest_id = {
                let agents = self.agents.read().await;
                agents
                    .iter()
                    .min_by_key(|(_, e)| e.info.last_used_at.unwrap_or(0))
                    .map(|(id, _)| *id)
            };
            if let Some(id) = oldest_id {
                tracing::info!(agent = %id, "evicting agent (LRU)");
                self.shutdown(id).await.ok();
            } else {
                return Err(PorpoiseError::ResourceLimit(format!(
                    "max agents ({}) reached",
                    self.max_size
                )));
            }
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let info = AgentInfo {
            id,
            kind,
            pid,
            status: AgentStatus::Running,
            worktree_path: Some(worktree.to_path_buf()),
            last_used_at: Some(now),
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

    pub async fn touch(&self, id: &AgentId) {
        let mut agents = self.agents.write().await;
        if let Some(entry) = agents.get_mut(id) {
            entry.info.last_used_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            );
        }
    }

    pub fn spawn_idle_cleanup(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let pool = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let to_evict: Vec<AgentId> = {
                    let agents = pool.agents.read().await;
                    agents
                        .iter()
                        .filter(|(_, e)| {
                            let last = e.info.last_used_at.unwrap_or(now);
                            now - last > pool.idle_timeout_secs as i64
                        })
                        .map(|(id, _)| *id)
                        .collect()
                };
                for id in to_evict {
                    tracing::info!(agent = %id, "evicting idle agent");
                    pool.shutdown(id).await.ok();
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = AgentPool::new(5);
        let agents = pool.list().await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_pool_with_idle_timeout() {
        let pool = AgentPool::with_idle_timeout(5, 1);
        let agents = pool.list().await;
        assert!(agents.is_empty());
    }
}
