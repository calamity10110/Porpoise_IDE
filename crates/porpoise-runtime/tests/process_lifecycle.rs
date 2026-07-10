use porpoise_core::bus::EventBus;
use porpoise_runtime::{ProcessKind, ProcessManager};

#[tokio::test]
async fn test_spawn_and_list() {
    let bus = EventBus::new(16);
    let mgr = ProcessManager::new(bus);

    #[cfg(unix)]
    let (cmd, args) = ("sleep", vec!["10"]);
    #[cfg(windows)]
    let (cmd, args) = ("ping", vec!["-n", "10", "127.0.0.1"]);

    let handle = mgr.spawn(ProcessKind::Other, cmd, &args, None, vec![]).await.unwrap();

    assert!(handle.pid > 0);
    assert_eq!(handle.kind, ProcessKind::Other);

    let list = mgr.list().await;
    assert_eq!(list.len(), 1);

    mgr.kill(handle.id).await.unwrap();
    let list = mgr.list().await;
    assert_eq!(list.len(), 0);
}

#[tokio::test]
async fn test_spawn_with_cwd() {
    let bus = EventBus::new(16);
    let mgr = ProcessManager::new(bus);
    let dir = tempfile::TempDir::new().unwrap();

    #[cfg(unix)]
    let (cmd, args) = ("pwd", vec![]);
    #[cfg(windows)]
    let (cmd, args) = ("cmd", vec!["/C", "cd"]);

    let handle = mgr
        .spawn(ProcessKind::Other, cmd, &args, Some(dir.path()), vec![])
        .await
        .unwrap();

    assert!(handle.pid > 0);
    mgr.kill(handle.id).await.unwrap();
}

#[tokio::test]
async fn test_shutdown_all() {
    let bus = EventBus::new(16);
    let mgr = ProcessManager::new(bus);

    #[cfg(unix)]
    let (cmd, args) = ("sleep", vec!["10"]);
    #[cfg(windows)]
    let (cmd, args) = ("ping", vec!["-n", "10", "127.0.0.1"]);

    for _ in 0..3 {
        mgr.spawn(ProcessKind::Other, cmd, &args, None, vec![]).await.unwrap();
    }

    assert_eq!(mgr.list().await.len(), 3);
    mgr.shutdown_all(std::time::Duration::from_secs(1)).await.unwrap();
    assert_eq!(mgr.list().await.len(), 0);
}

#[tokio::test]
async fn test_kill_nonexistent() {
    let bus = EventBus::new(16);
    let mgr = ProcessManager::new(bus);

    let fake_id = porpoise_core::types::id::ProcessId::new();
    let result = mgr.kill(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spawn_invalid_command() {
    let bus = EventBus::new(16);
    let mgr = ProcessManager::new(bus);

    let result = mgr
        .spawn(ProcessKind::Other, "nonexistent-binary-xyz", &[], None, vec![])
        .await;
    assert!(result.is_err());
}
