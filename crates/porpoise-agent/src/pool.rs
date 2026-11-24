use std::{collections::HashMap, path::Path, sync::Arc};

use porpoise_core::{
    error::{PorpoiseError, Result},
    types::id::AgentId,
};
use tokio::sync::RwLock;

use crate::{
    adapters::AdapterRegistry,
    generic::GenericAgent,
    traits::{Agent, AgentHandle},
    types::{AgentInfo, AgentKind, AgentStatus},
};

/// The binary a given agent kind runs. Adapter kinds must agree with
/// their adapter's `binary_name`.
fn binary_for_kind(kind: &AgentKind) -> String {
    match kind {
        AgentKind::ClaudeCode => "claude",
        AgentKind::Codex => "codex",
        AgentKind::Gemini => "gemini",
        AgentKind::OpenCode => "opencode",
        AgentKind::Aider => "aider",
        AgentKind::ZAI => "z",
        AgentKind::OpenAI => "openai",
        AgentKind::Grok => "grok",
        AgentKind::OpenRouter => "openrouter",
        AgentKind::Custom(name) => name.as_str(),
    }
    .to_string()
}

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
        self.spawn_inner(kind, worktree, Vec::new(), None).await
    }

    /// Spawn `kind` in `worktree` and deliver `prompt` to the agent.
    ///
    /// When an adapter knows the binary (claude, codex, aider, opencode)
    /// the prompt travels in the CLI arguments that adapter builds;
    /// otherwise it is written to the agent's stdin as a fallback.
    pub async fn spawn_with_prompt(&self, kind: AgentKind, worktree: &Path, prompt: Option<&str>) -> Result<AgentId> {
        let registry = AdapterRegistry::with_defaults();
        let (args, stdin_prompt) = match registry.get(&kind) {
            Some(adapter) => (adapter.build_args(worktree, prompt, &[]), None),
            None => (Vec::new(), prompt),
        };
        self.spawn_inner(kind, worktree, args, stdin_prompt).await
    }

    async fn spawn_inner(
        &self,
        kind: AgentKind,
        worktree: &Path,
        args: Vec<String>,
        stdin_prompt: Option<&str>,
    ) -> Result<AgentId> {
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
        let binary = binary_for_kind(&kind);
        let agent: Box<dyn Agent> = Box::new(GenericAgent::with_kind(&binary, kind.clone()).with_args(args));
        let mut handle = agent.spawn(worktree).await?;
        if let Some(prompt) = stdin_prompt
            && !prompt.is_empty()
        {
            // Line-oriented agents (cmd, sh, REPLs) wait for a newline
            // before acting on what they read from stdin.
            let mut line = prompt.to_string();
            if !line.ends_with('\n') {
                line.push('\n');
            }
            if let Err(e) = handle.send_input(&line).await {
                tracing::warn!(agent_kind = %kind, "prompt could not be written to stdin: {e}");
            }
        }
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

    /// Remove an agent's process handle from the pool so a caller can read
    /// its output exclusively (an AG-UI run does this). The `info` entry
    /// stays behind, so `list` and `shutdown` still see the agent.
    pub async fn take_handle(&self, id: AgentId) -> Option<Box<dyn AgentHandle>> {
        self.agents.write().await.get_mut(&id)?.handle.take()
    }

    /// Give a handle back after the run ends, if the entry is still there.
    /// Returns `true` when the pool took ownership again; on `false` the
    /// caller must shut the handle down itself.
    pub async fn restore_handle(&self, id: AgentId, handle: Box<dyn AgentHandle>) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(entry) = agents.get_mut(&id)
            && entry.handle.is_none()
        {
            entry.handle = Some(handle);
            true
        } else {
            false
        }
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

    /// An AG-UI run takes the handle out of the pool while it streams and
    /// gives it back afterwards; a stopped agent's entry disappears under
    /// it, in which case the caller keeps ownership.
    #[tokio::test]
    async fn take_and_restore_handle_round_trip() {
        let pool = AgentPool::new(4);
        // A bare binary with no adapter: an interactive shell stays alive
        // with piped stdin, giving us a real handle to move around.
        let kind = AgentKind::Custom(if cfg!(windows) { "cmd" } else { "sh" }.into());
        let id = pool
            .spawn_with_prompt(kind, &std::env::temp_dir(), None)
            .await
            .expect("spawn");

        let handle = pool.take_handle(id).await.expect("handle present");
        assert!(pool.take_handle(id).await.is_none(), "a handle can only be taken once");
        assert!(
            pool.restore_handle(id, handle).await,
            "restore succeeds while the entry exists"
        );

        // After shutdown the entry is gone: restore must refuse the handle.
        pool.shutdown(id).await.expect("shutdown");
        let spare = pool
            .spawn_with_prompt(
                AgentKind::Custom(if cfg!(windows) { "cmd" } else { "sh" }.into()),
                &std::env::temp_dir(),
                None,
            )
            .await
            .expect("spawn");
        let handle = pool.take_handle(spare).await.expect("handle present");
        pool.shutdown(spare).await.expect("shutdown");
        assert!(
            !pool.restore_handle(spare, handle).await,
            "restore refuses once the entry is gone"
        );
    }
}

// ── Test helpers: mock agent injection ──────────────────────────────
#[cfg(feature = "test-helpers")]
pub mod test_helpers {
    use super::*;
    use crate::types::AgentOutput;

    /// A no-op agent for testing pool logic without spawning real processes.
    pub struct MockAgent {
        kind: AgentKind,
    }

    impl MockAgent {
        pub fn new(kind: AgentKind) -> Self {
            Self { kind }
        }
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        fn kind(&self) -> AgentKind {
            self.kind.clone()
        }
        fn binary_name(&self) -> &str {
            "mock"
        }
        async fn spawn(&self, _worktree: &Path) -> Result<Box<dyn AgentHandle>> {
            Ok(Box::new(MockHandle))
        }
    }

    /// A no-op handle that always reports as not running.
    pub struct MockHandle;

    #[async_trait::async_trait]
    impl AgentHandle for MockHandle {
        fn pid(&self) -> Option<u32> {
            Some(0)
        }
        async fn read_output(&mut self) -> Result<Option<AgentOutput>> {
            Ok(None)
        }
        async fn send_input(&mut self, _text: &str) -> Result<()> {
            Ok(())
        }
        async fn interrupt(&mut self) -> Result<()> {
            Ok(())
        }
        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
        fn is_running(&self) -> bool {
            false
        }
    }

    impl AgentPool {
        /// Inject a mock agent directly into the pool (bypasses real process spawn).
        /// Respects max_size with LRU eviction, same as `spawn`.
        /// Returns the assigned AgentId.
        pub async fn inject_mock(&self, kind: AgentKind) -> AgentId {
            // LRU eviction if pool is full (mirrors spawn logic)
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
                    self.shutdown(id).await.ok();
                }
            }

            let id = AgentId::new();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let info = AgentInfo {
                id,
                kind,
                pid: Some(0),
                status: AgentStatus::Running,
                worktree_path: None,
                last_used_at: Some(now),
            };
            self.agents.write().await.insert(
                id,
                AgentEntry {
                    agent: Box::new(MockAgent::new(info.kind.clone())),
                    handle: Some(Box::new(MockHandle)),
                    info,
                },
            );
            id
        }

        /// Inject a mock agent with a specific last_used_at timestamp (for LRU testing).
        /// Respects max_size with LRU eviction.
        pub async fn inject_mock_at(&self, kind: AgentKind, last_used_at: i64) -> AgentId {
            // LRU eviction if pool is full
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
                    self.shutdown(id).await.ok();
                }
            }

            let id = AgentId::new();
            let info = AgentInfo {
                id,
                kind,
                pid: Some(0),
                status: AgentStatus::Running,
                worktree_path: None,
                last_used_at: Some(last_used_at),
            };
            self.agents.write().await.insert(
                id,
                AgentEntry {
                    agent: Box::new(MockAgent::new(info.kind.clone())),
                    handle: Some(Box::new(MockHandle)),
                    info,
                },
            );
            id
        }
    }
}
