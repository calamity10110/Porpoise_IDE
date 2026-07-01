use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bus::EventBus;
use crate::config::AppConfig;
use crate::types::event::{AgentStatusKind, WorktreeStatus};
use crate::types::id::*;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: RwLock<AppConfig>,
    pub event_bus: EventBus,
    pub worktrees: RwLock<HashMap<WorktreeId, WorktreeState>>,
    pub agents: RwLock<HashMap<AgentId, AgentState>>,
    pub sessions: RwLock<HashMap<SessionId, SessionState>>,
}

impl AppState {
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

    pub fn event_bus(&self) -> &EventBus {
        &self.inner.event_bus
    }

    pub async fn config(&self) -> tokio::sync::RwLockReadGuard<'_, AppConfig> {
        self.inner.config.read().await
    }

    pub async fn config_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, AppConfig> {
        self.inner.config.write().await
    }

    pub async fn worktrees(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<WorktreeId, WorktreeState>> {
        self.inner.worktrees.read().await
    }

    pub async fn worktrees_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, HashMap<WorktreeId, WorktreeState>> {
        self.inner.worktrees.write().await
    }

    pub async fn agents(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<AgentId, AgentState>> {
        self.inner.agents.read().await
    }

    pub async fn agents_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, HashMap<AgentId, AgentState>> {
        self.inner.agents.write().await
    }
}

#[derive(Clone, Debug)]
pub struct WorktreeState {
    pub id: WorktreeId,
    pub path: std::path::PathBuf,
    pub name: String,
    pub status: WorktreeStatus,
    pub agent_id: Option<AgentId>,
}

#[derive(Clone, Debug)]
pub struct AgentState {
    pub id: AgentId,
    pub kind: String,
    pub pid: Option<u32>,
    pub status: AgentStatusKind,
    pub worktree_id: Option<WorktreeId>,
}

#[derive(Clone, Debug)]
pub struct SessionState {
    pub id: SessionId,
    pub worktree_id: WorktreeId,
    pub started_at: chrono::DateTime<chrono::Utc>,
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
