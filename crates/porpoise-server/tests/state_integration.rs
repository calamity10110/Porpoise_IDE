//! Cross-crate integration tests for AppState (porpoise-core).

use porpoise_core::{
    bus::EventBus,
    config::AppConfig,
    state::{AgentState, AppState, SessionState, WorktreeState},
    types::{
        event::{AgentStatusKind, WorktreeStatus},
        id::*,
    },
};

#[tokio::test]
async fn test_full_state_lifecycle() {
    let bus = EventBus::new(64);
    let state = AppState::new(AppConfig::default(), bus);

    // Create a worktree
    let wt_id = WorktreeId::new();
    let wt = WorktreeState {
        id: wt_id,
        path: "/tmp/test-wt".into(),
        name: "test-wt".into(),
        status: WorktreeStatus::Idle,
        agent_id: None,
    };
    state.worktrees_mut().await.insert(wt_id, wt);

    // Create an agent and attach to worktree
    let agent_id = AgentId::new();
    let agent = AgentState {
        id: agent_id,
        kind: "claude".into(),
        pid: Some(12345),
        status: AgentStatusKind::Running,
        worktree_id: Some(wt_id),
    };
    state.agents_mut().await.insert(agent_id, agent);

    // Update worktree status
    {
        let mut wts = state.worktrees_mut().await;
        let wt = wts.get_mut(&wt_id).unwrap();
        wt.status = WorktreeStatus::Running { agent: agent_id };
        wt.agent_id = Some(agent_id);
    }

    // Create a session
    let sess_id = SessionId::new();
    let session = SessionState {
        id: sess_id,
        worktree_id: wt_id,
        started_at: chrono::Utc::now(),
        ended_at: None,
    };
    state.sessions_mut().await.insert(sess_id, session);

    // Verify all state
    assert_eq!(state.worktrees().await.len(), 1);
    assert_eq!(state.agents().await.len(), 1);
    assert_eq!(state.sessions().await.len(), 1);

    let wt = state.worktrees().await.get(&wt_id).cloned().unwrap();
    assert!(matches!(wt.status, WorktreeStatus::Running { .. }));
    assert_eq!(wt.agent_id, Some(agent_id));
}

#[tokio::test]
async fn test_state_concurrent_access() {
    let bus = EventBus::new(64);
    let state = AppState::new(AppConfig::default(), bus);

    // Spawn multiple tasks writing to different maps concurrently
    let s1 = state.clone();
    let h1 = tokio::spawn(async move {
        for i in 0..50 {
            let id = WorktreeId::new();
            let wt = WorktreeState {
                id,
                path: format!("/tmp/wt-{i}").into(),
                name: format!("wt-{i}"),
                status: WorktreeStatus::Idle,
                agent_id: None,
            };
            s1.worktrees_mut().await.insert(id, wt);
        }
    });

    let s2 = state.clone();
    let h2 = tokio::spawn(async move {
        for i in 0..50 {
            let id = AgentId::new();
            let agent = AgentState {
                id,
                kind: format!("agent-{i}"),
                pid: None,
                status: AgentStatusKind::Spawning,
                worktree_id: None,
            };
            s2.agents_mut().await.insert(id, agent);
        }
    });

    let s3 = state.clone();
    let h3 = tokio::spawn(async move {
        for _i in 0..50 {
            let id = SessionId::new();
            let session = SessionState {
                id,
                worktree_id: WorktreeId::new(),
                started_at: chrono::Utc::now(),
                ended_at: None,
            };
            s3.sessions_mut().await.insert(id, session);
        }
    });

    let _ = tokio::join!(h1, h2, h3);

    assert_eq!(state.worktrees().await.len(), 50);
    assert_eq!(state.agents().await.len(), 50);
    assert_eq!(state.sessions().await.len(), 50);
}
