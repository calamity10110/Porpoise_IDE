use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    bus::EventBus,
    config::AppConfig,
    types::{
        event::{AgentStatusKind, WorktreeStatus},
        id::*,
    },
};

/// Shared application state, accessible across all service tasks.
///
/// Wraps interior-mutable state in `Arc<RwLock<>>` so multiple Tokio tasks can
/// read and write concurrently. Clone is O(1) and produces a handle to the same
/// underlying state.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: RwLock<AppConfig>,
    pub event_bus: EventBus,
    pub worktrees: RwLock<HashMap<WorktreeId, WorktreeState>>,
    pub agents: RwLock<HashMap<AgentId, AgentState>>,
    #[allow(dead_code)]
    pub sessions: RwLock<HashMap<SessionId, SessionState>>,
}

impl AppState {
    /// Creates a new `AppState` with the given config and event bus.
    ///
    /// All internal maps are initially empty — populate them via the `_mut()`
    /// accessors as services start up.
    pub fn new(config: AppConfig, event_bus: EventBus) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config: RwLock::new(config),
                event_bus,
                worktrees: RwLock::new(HashMap::new()),
                agents: RwLock::new(HashMap::new()),
                sessions: RwLock::new(HashMap::new()),
            }),
        }
    }

    /// Returns a reference to the shared event bus for publishing/subscribing.
    pub fn event_bus(&self) -> &EventBus {
        &self.inner.event_bus
    }

    /// Acquires a read guard on the application config.
    pub async fn config(&self) -> tokio::sync::RwLockReadGuard<'_, AppConfig> {
        self.inner.config.read().await
    }

    /// Acquires a write guard on the application config.
    pub async fn config_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, AppConfig> {
        self.inner.config.write().await
    }

    /// Acquires a read guard on the worktree map.
    pub async fn worktrees(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<WorktreeId, WorktreeState>> {
        self.inner.worktrees.read().await
    }

    /// Acquires a write guard on the worktree map.
    pub async fn worktrees_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, HashMap<WorktreeId, WorktreeState>> {
        self.inner.worktrees.write().await
    }

    /// Acquires a read guard on the agent map.
    pub async fn agents(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<AgentId, AgentState>> {
        self.inner.agents.read().await
    }

    /// Acquires a write guard on the agent map.
    pub async fn agents_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, HashMap<AgentId, AgentState>> {
        self.inner.agents.write().await
    }
}

/// Runtime state for a single worktree.
#[derive(Clone, Debug)]
pub struct WorktreeState {
    /// Unique worktree identifier.
    pub id: WorktreeId,
    /// Filesystem path to the worktree directory.
    pub path: std::path::PathBuf,
    /// Human-readable name (e.g. "fix-auth").
    pub name: String,
    /// Current lifecycle status.
    pub status: WorktreeStatus,
    /// The agent assigned to this worktree, if any.
    pub agent_id: Option<AgentId>,
}

/// Runtime state for a single agent process.
#[derive(Clone, Debug)]
pub struct AgentState {
    /// Unique agent identifier.
    pub id: AgentId,
    /// Agent kind string (e.g. "claude", "codex").
    pub kind: String,
    /// OS process ID, if the agent is running.
    pub pid: Option<u32>,
    /// Current agent lifecycle status.
    pub status: AgentStatusKind,
    /// The worktree this agent is attached to, if any.
    pub worktree_id: Option<WorktreeId>,
}

/// Runtime state for a single session.
#[derive(Clone, Debug)]
pub struct SessionState {
    /// Unique session identifier.
    pub id: SessionId,
    /// The worktree this session belongs to.
    pub worktree_id: WorktreeId,
    /// When the session started.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// When the session ended, if completed.
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::EventBus;

    #[tokio::test]
    async fn test_app_state_creation() {
        let bus = EventBus::new(16);
        let state = AppState::new(AppConfig::default(), bus);
        let wts = state.worktrees().await;
        assert!(wts.is_empty());
    }

    #[tokio::test]
    async fn test_app_state_config_access() {
        let bus = EventBus::new(16);
        let state = AppState::new(AppConfig::default(), bus);
        let cfg = state.config().await;
        assert_eq!(cfg.core.event_bus_capacity, 1024);
    }

    #[tokio::test]
    async fn test_app_state_mutate_worktrees() {
        let bus = EventBus::new(16);
        let state = AppState::new(AppConfig::default(), bus);

        let id = WorktreeId::new();
        let wt = WorktreeState {
            id,
            path: "/tmp/test".into(),
            name: "test".into(),
            status: WorktreeStatus::Idle,
            agent_id: None,
        };

        state.worktrees_mut().await.insert(id, wt);
        let wts = state.worktrees().await;
        assert_eq!(wts.len(), 1);
    }
}
