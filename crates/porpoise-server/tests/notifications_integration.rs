//! Cross-crate integration tests for NotificationService (porpoise-server + porpoise-db).

use porpoise_core::types::event::NotificationSeverity;
use porpoise_db::{DbPool, migration::run_migrations};
use porpoise_server::services::notifications::{NotificationPreferences, NotificationService};

async fn setup() -> (NotificationService, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().unwrap();
    let db = DbPool::open(&dir.path().join("test.db")).unwrap();
    run_migrations(&db).unwrap();
    let svc = NotificationService::new(db);
    // Disable desktop notifications in tests
    svc.set_preferences(NotificationPreferences {
        desktop_notifications: false,
        ..NotificationPreferences::defaults()
    })
    .await;
    (svc, dir)
}

#[tokio::test]
async fn test_notify_and_list_unread() {
    let (svc, _dir) = setup().await;

    svc.notify("Test Title", "test body", NotificationSeverity::Info, "test")
        .await
        .unwrap();

    let unread = svc.list_unread().await.unwrap();
    assert_eq!(unread.len(), 1);
    assert_eq!(unread[0].title, "Test Title");
    assert_eq!(unread[0].body, "test body");
    assert_eq!(unread[0].source, "test");
    assert!(!unread[0].read);
}

#[tokio::test]
async fn test_mark_read_and_list_all() {
    let (svc, _dir) = setup().await;

    svc.notify("A", "body_a", NotificationSeverity::Info, "s")
        .await
        .unwrap();
    svc.notify("B", "body_b", NotificationSeverity::Warning, "s")
        .await
        .unwrap();
    svc.notify("C", "body_c", NotificationSeverity::Error, "s")
        .await
        .unwrap();

    let all = svc.list_all(100).await.unwrap();
    assert_eq!(all.len(), 3);

    // Mark first as read
    svc.mark_read(&all[0].id).await.unwrap();

    let unread = svc.list_unread().await.unwrap();
    assert_eq!(unread.len(), 2);

    // Verify the marked one is no longer in unread
    let unread_ids: Vec<_> = unread.iter().map(|n| &n.id).collect();
    assert!(!unread_ids.contains(&&all[0].id));
}

#[tokio::test]
async fn test_mark_all_read() {
    let (svc, _dir) = setup().await;

    svc.notify("X", "x", NotificationSeverity::Info, "s").await.unwrap();
    svc.notify("Y", "y", NotificationSeverity::Warning, "s").await.unwrap();

    assert_eq!(svc.unread_count().await.unwrap(), 2);

    svc.mark_all_read().await.unwrap();

    assert_eq!(svc.unread_count().await.unwrap(), 0);
    let unread = svc.list_unread().await.unwrap();
    assert!(unread.is_empty());
}

#[tokio::test]
async fn test_unread_count() {
    let (svc, _dir) = setup().await;

    assert_eq!(svc.unread_count().await.unwrap(), 0);

    svc.notify("A", "a", NotificationSeverity::Info, "s").await.unwrap();
    assert_eq!(svc.unread_count().await.unwrap(), 1);

    svc.notify("B", "b", NotificationSeverity::Error, "s").await.unwrap();
    assert_eq!(svc.unread_count().await.unwrap(), 2);
}

#[tokio::test]
async fn test_clear() {
    let (svc, _dir) = setup().await;

    svc.notify("A", "a", NotificationSeverity::Info, "s").await.unwrap();
    svc.notify("B", "b", NotificationSeverity::Warning, "s").await.unwrap();

    assert_eq!(svc.list_all(100).await.unwrap().len(), 2);

    svc.clear().await.unwrap();

    assert_eq!(svc.list_all(100).await.unwrap().len(), 0);
    assert_eq!(svc.unread_count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_list_all_with_limit() {
    let (svc, _dir) = setup().await;

    for i in 0..5 {
        svc.notify(&format!("N{i}"), "body", NotificationSeverity::Info, "s")
            .await
            .unwrap();
    }

    let all = svc.list_all(3).await.unwrap();
    assert_eq!(all.len(), 3, "list_all(3) should return at most 3");

    let all = svc.list_all(100).await.unwrap();
    assert_eq!(all.len(), 5, "list_all(100) should return all 5");
}

#[tokio::test]
async fn test_severity_stored_correctly() {
    let (svc, _dir) = setup().await;

    svc.notify("I", "i", NotificationSeverity::Info, "s").await.unwrap();
    svc.notify("W", "w", NotificationSeverity::Warning, "s").await.unwrap();
    svc.notify("E", "e", NotificationSeverity::Error, "s").await.unwrap();

    let all = svc.list_all(100).await.unwrap();
    let severities: Vec<_> = all.iter().map(|n| n.severity.as_str()).collect();
    assert!(severities.contains(&"Info"));
    assert!(severities.contains(&"Warning"));
    assert!(severities.contains(&"Error"));
}
