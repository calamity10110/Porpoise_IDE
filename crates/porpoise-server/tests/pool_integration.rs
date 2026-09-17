//! Cross-crate integration tests for AgentPool (porpoise-agent).
//!
//! Uses mock agent injection to test pool logic (LRU, capacity, touch, shutdown)
//! without requiring real agent binaries.

use porpoise_agent::pool::AgentPool;
use porpoise_agent::types::AgentKind;

#[tokio::test]
async fn test_pool_create_and_list() {
    let pool = AgentPool::new(5);
    let agents = pool.list().await;
    assert!(agents.is_empty(), "new pool should have no agents");
}

#[tokio::test]
async fn test_pool_custom_idle_timeout() {
    let pool = AgentPool::with_idle_timeout(3, 60);
    assert_eq!(pool.list().await.len(), 0);
}

#[tokio::test]
async fn test_pool_spawn_and_list_mock() {
    let pool = AgentPool::new(5);
    let id = pool.inject_mock(AgentKind::ClaudeCode).await;
    let agents = pool.list().await;
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].id, id);
    assert_eq!(agents[0].kind, AgentKind::ClaudeCode);
}

#[tokio::test]
async fn test_pool_shutdown_mock() {
    let pool = AgentPool::new(5);
    let id = pool.inject_mock(AgentKind::Codex).await;
    assert_eq!(pool.list().await.len(), 1);

    pool.shutdown(id).await.unwrap();
    assert_eq!(pool.list().await.len(), 0);
}

#[tokio::test]
async fn test_pool_shutdown_all_mock() {
    let pool = AgentPool::new(5);
    pool.inject_mock(AgentKind::ClaudeCode).await;
    pool.inject_mock(AgentKind::Codex).await;
    pool.inject_mock(AgentKind::Gemini).await;
    assert_eq!(pool.list().await.len(), 3);

    pool.shutdown_all().await.unwrap();
    assert_eq!(pool.list().await.len(), 0);
}

#[tokio::test]
async fn test_pool_touch_updates_last_used() {
    let pool = AgentPool::new(5);
    let id = pool.inject_mock_at(AgentKind::ClaudeCode, 1000).await;

    let before = pool.list().await[0].last_used_at.unwrap();
    assert_eq!(before, 1000);

    pool.touch(&id).await;

    let after = pool.list().await[0].last_used_at.unwrap();
    assert!(after > 1000, "touch should update last_used_at");
}

#[tokio::test]
async fn test_pool_lru_eviction_on_full() {
    let pool = AgentPool::new(2);

    // Inject two agents with different timestamps
    let id_old = pool.inject_mock_at(AgentKind::ClaudeCode, 100).await;
    let id_new = pool.inject_mock_at(AgentKind::Codex, 200).await;
    assert_eq!(pool.list().await.len(), 2);

    // Inject a third — should evict the oldest (id_old)
    let id_third = pool.inject_mock_at(AgentKind::Gemini, 300).await;
    let agents = pool.list().await;
    assert_eq!(agents.len(), 2, "pool should still be at max_size");

    let ids: Vec<_> = agents.iter().map(|a| a.id).collect();
    assert!(!ids.contains(&id_old), "oldest agent should have been evicted");
    assert!(ids.contains(&id_new), "newer agent should remain");
    assert!(ids.contains(&id_third), "newest agent should remain");

    pool.shutdown_all().await.unwrap();
}

#[tokio::test]
async fn test_pool_lru_eviction_oldest_first() {
    let pool = AgentPool::new(1);

    // First agent with old timestamp
    let id1 = pool.inject_mock_at(AgentKind::ClaudeCode, 50).await;
    assert_eq!(pool.list().await.len(), 1);

    // Second agent — should evict id1 (only slot, LRU)
    let id2 = pool.inject_mock_at(AgentKind::Codex, 100).await;
    assert_eq!(pool.list().await.len(), 1);
    assert_ne!(id1, id2, "new agent should have different id");

    let remaining = pool.list().await;
    assert_eq!(remaining[0].id, id2, "only the newest agent should remain");

    pool.shutdown(id2).await.unwrap();
}

#[tokio::test]
async fn test_pool_multiple_kinds() {
    let pool = AgentPool::new(10);

    pool.inject_mock(AgentKind::ClaudeCode).await;
    pool.inject_mock(AgentKind::Codex).await;
    pool.inject_mock(AgentKind::Gemini).await;
    pool.inject_mock(AgentKind::OpenCode).await;
    pool.inject_mock(AgentKind::ZAI).await;

    let agents = pool.list().await;
    assert_eq!(agents.len(), 5);

    let kinds: Vec<_> = agents.iter().map(|a| &a.kind).collect();
    assert!(kinds.contains(&&AgentKind::ClaudeCode));
    assert!(kinds.contains(&&AgentKind::Codex));
    assert!(kinds.contains(&&AgentKind::Gemini));
    assert!(kinds.contains(&&AgentKind::OpenCode));
    assert!(kinds.contains(&&AgentKind::ZAI));

    pool.shutdown_all().await.unwrap();
}
