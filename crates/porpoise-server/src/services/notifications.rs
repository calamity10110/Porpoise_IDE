use std::sync::Arc;

use chrono::Utc;
use porpoise_core::{
    error::{PorpoiseError, Result},
    types::event::NotificationSeverity,
};
use porpoise_db::DbPool;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast};

/// A notification action button shown in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub action_type: String, // "open_url", "run_command", "dismiss"
    pub payload: Option<String>,
}

/// Rich notification with actions, grouping, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichNotification {
    pub record: NotificationRecord,
    pub actions: Vec<NotificationAction>,
    pub group_key: Option<String>,
    pub icon: Option<String>,
    pub link: Option<String>,
}

/// Delivery channel for notifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Desktop,
    Sound,
    Webhook { url: String },
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub id: String,
    pub title: String,
    pub body: String,
    pub severity: String,
    pub source: String,
    pub timestamp: i64,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationPreferences {
    pub agent_completion: bool,
    pub agent_errors: bool,
    pub agent_warnings: bool,
    pub terminal_bell: bool,
    pub desktop_notifications: bool,
    pub sound: bool,
}

impl NotificationPreferences {
    pub fn defaults() -> Self {
        Self {
            agent_completion: true,
            agent_errors: true,
            agent_warnings: false,
            terminal_bell: true,
            desktop_notifications: true,
            sound: false,
        }
    }
}

pub struct NotificationService {
    db: DbPool,
    prefs: Arc<Mutex<NotificationPreferences>>,
    /// SSE broadcast channel for real-time notification push to frontend.
    event_tx: broadcast::Sender<RichNotification>,
    /// HTTP client for webhook delivery.
    http: reqwest::Client,
}

impl NotificationService {
    pub fn new(db: DbPool) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            db,
            prefs: Arc::new(Mutex::new(NotificationPreferences::defaults())),
            event_tx,
            http: reqwest::Client::new(),
        }
    }

    /// Subscribe to the SSE notification stream.
    pub fn subscribe(&self) -> broadcast::Receiver<RichNotification> {
        self.event_tx.subscribe()
    }

    pub async fn notify(
        &self,
        title: &str,
        body: &str,
        severity: NotificationSeverity,
        source: &str,
    ) -> Result<NotificationRecord> {
        let now = Utc::now();
        let record = NotificationRecord {
            id: format!("notif_{}_{}", now.timestamp_millis(), uuid::Uuid::now_v7().simple()),
            title: title.to_string(),
            body: body.to_string(),
            severity: serde_json::to_string(&severity)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            source: source.to_string(),
            timestamp: now.timestamp_millis(),
            read: false,
        };

        self.store(&record).await?;

        let prefs = self.prefs.lock().await;
        if prefs.desktop_notifications && self.should_notify(severity.clone(), &prefs) {
            self.show_desktop_notification(&record);
        }
        if prefs.sound && self.should_notify(severity.clone(), &prefs) {
            self.play_notification_sound(&severity);
        }

        // Broadcast to SSE subscribers (ignore if no receivers)
        let rich = RichNotification {
            record: record.clone(),
            actions: vec![],
            group_key: None,
            icon: None,
            link: None,
        };
        let _ = self.event_tx.send(rich);

        Ok(record)
    }

    async fn store(&self, record: &NotificationRecord) -> Result<()> {
        let db = self.db.clone();
        let rec = record.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            conn.execute(
                "INSERT INTO notifications (id, title, message, level, source, created_at, read)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    rec.id,
                    rec.title,
                    rec.body,
                    rec.severity,
                    rec.source,
                    rec.timestamp,
                    rec.read as i32,
                ],
            )
            .map_err(|e| PorpoiseError::Internal(format!("insert notification: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn list_unread(&self) -> Result<Vec<NotificationRecord>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, message, level, source, created_at, read
                     FROM notifications WHERE read = 0 ORDER BY created_at DESC LIMIT 100",
                )
                .map_err(|e| PorpoiseError::Internal(format!("prepare: {e}")))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(NotificationRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        body: row.get(2)?,
                        severity: row.get(3)?,
                        source: row.get(4)?,
                        timestamp: row.get::<_, i64>(5)?,
                        read: row.get::<_, i32>(6)? != 0,
                    })
                })
                .map_err(|e| PorpoiseError::Internal(format!("query: {e}")))?;

            rows.filter_map(|r| r.ok()).map(Ok).collect()
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn list_all(&self, limit: usize) -> Result<Vec<NotificationRecord>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, message, level, source, created_at, read
                     FROM notifications ORDER BY created_at DESC LIMIT ?1",
                )
                .map_err(|e| PorpoiseError::Internal(format!("prepare: {e}")))?;

            let rows = stmt
                .query_map(rusqlite::params![limit as i64], |row| {
                    Ok(NotificationRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        body: row.get(2)?,
                        severity: row.get(3)?,
                        source: row.get(4)?,
                        timestamp: row.get::<_, i64>(5)?,
                        read: row.get::<_, i32>(6)? != 0,
                    })
                })
                .map_err(|e| PorpoiseError::Internal(format!("query: {e}")))?;

            rows.filter_map(|r| r.ok()).map(Ok).collect()
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn mark_read(&self, id: &str) -> Result<()> {
        let db = self.db.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            conn.execute("UPDATE notifications SET read = 1 WHERE id = ?1", rusqlite::params![id])
                .map_err(|e| PorpoiseError::Internal(format!("mark read: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn mark_all_read(&self) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            conn.execute("UPDATE notifications SET read = 1", [])
                .map_err(|e| PorpoiseError::Internal(format!("mark all read: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn clear(&self) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            conn.execute("DELETE FROM notifications", [])
                .map_err(|e| PorpoiseError::Internal(format!("clear: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn unread_count(&self) -> Result<usize> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let conn = db.get().map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM notifications WHERE read = 0", [], |row| {
                    row.get(0)
                })
                .map_err(|e| PorpoiseError::Internal(format!("count: {e}")))?;
            Ok(count as usize)
        })
        .await
        .map_err(|e| PorpoiseError::Internal(format!("spawn_blocking: {e}")))?
    }

    pub async fn set_preferences(&self, prefs: NotificationPreferences) {
        *self.prefs.lock().await = prefs;
    }

    pub async fn get_preferences(&self) -> NotificationPreferences {
        self.prefs.lock().await.clone()
    }

    pub fn prune_old(&self, keep_count: usize) -> Result<usize> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Db(format!("prune get conn: {e}")))?;
        let deleted = conn
            .execute(
                "DELETE FROM notifications WHERE id IN (
                    SELECT id FROM notifications ORDER BY created_at ASC
                    LIMIT MAX(0, (SELECT CAST(COUNT(*) AS INTEGER) - ?1 FROM notifications))
                )",
                rusqlite::params![keep_count as i64],
            )
            .map_err(|e| PorpoiseError::Db(format!("prune delete: {e}")))?;
        Ok(deleted)
    }

    fn should_notify(&self, severity: NotificationSeverity, prefs: &NotificationPreferences) -> bool {
        match severity {
            NotificationSeverity::Info => prefs.agent_completion,
            NotificationSeverity::Warning => prefs.agent_warnings,
            NotificationSeverity::Error => prefs.agent_errors,
        }
    }

    fn show_desktop_notification(&self, record: &NotificationRecord) {
        let summary = record.title.clone();
        let body = record.body.clone();
        let timeout = match record.severity.as_str() {
            "Error" => notify_rust::Timeout::Milliseconds(10000),
            "Warning" => notify_rust::Timeout::Milliseconds(7000),
            _ => notify_rust::Timeout::Milliseconds(5000),
        };

        tokio::task::spawn_blocking(move || {
            let _ = notify_rust::Notification::new()
                .summary(&summary)
                .body(&body)
                .timeout(timeout)
                .show();
        });
    }

    /// Play a system sound based on notification severity.
    fn play_notification_sound(&self, severity: &NotificationSeverity) {
        use std::process::Command;
        let sound = match severity {
            NotificationSeverity::Error => "SystemHand",
            NotificationSeverity::Warning => "SystemExclamation",
            NotificationSeverity::Info => "SystemNotification",
        };
        // Windows: use PowerShell to play system sound
        let _ = Command::new("powershell")
            .args([
                "-c",
                &format!(
                    "(New-Object Media.SoundPlayer 'C:\\Windows\\Media\\{}.wav').Play()",
                    sound
                ),
            ])
            .spawn();
    }

    /// Deliver a notification via webhook (HTTP POST JSON).
    pub async fn deliver_webhook(&self, url: &str, record: &NotificationRecord) -> Result<()> {
        let payload = serde_json::json!({
            "title": record.title,
            "body": record.body,
            "severity": record.severity,
            "source": record.source,
            "timestamp": record.timestamp,
            "id": record.id,
        });
        self.http
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| PorpoiseError::Internal(format!("webhook delivery failed: {e}")))?;
        Ok(())
    }

    /// Send a rich notification with actions, grouping, and optional webhook delivery.
    pub async fn notify_rich(
        &self,
        title: &str,
        body: &str,
        severity: NotificationSeverity,
        source: &str,
        actions: Vec<NotificationAction>,
        group_key: Option<String>,
        link: Option<String>,
        webhook_url: Option<&str>,
    ) -> Result<RichNotification> {
        let record = self.notify(title, body, severity.clone(), source).await?;

        // Deliver to webhook if configured
        if let Some(url) = webhook_url {
            if let Err(e) = self.deliver_webhook(url, &record).await {
                tracing::warn!(url, error=%e, "webhook delivery failed");
            }
        }

        let rich = RichNotification {
            record,
            actions,
            group_key,
            icon: None,
            link,
        };

        // Broadcast rich version to SSE
        let _ = self.event_tx.send(rich.clone());
        Ok(rich)
    }

    /// Get grouped unread notifications (groups by group_key).
    pub async fn list_grouped(&self) -> Result<Vec<(String, Vec<NotificationRecord>)>> {
        let all = self.list_unread().await?;
        let mut groups: std::collections::HashMap<String, Vec<NotificationRecord>> = std::collections::HashMap::new();
        for rec in all {
            let key = rec.source.clone(); // group by source as default
            groups.entry(key).or_default().push(rec);
        }
        Ok(groups.into_iter().collect())
    }

    pub async fn notify_agent_completion(
        &self,
        agent_name: &str,
        worktree: &str,
        exit_code: i32,
    ) -> Result<NotificationRecord> {
        let (title, body, severity) = if exit_code == 0 {
            (
                "Agent Completed".to_string(),
                format!("{agent_name} finished in {worktree}"),
                NotificationSeverity::Info,
            )
        } else {
            (
                "Agent Failed".to_string(),
                format!("{agent_name} exited with code {exit_code} in {worktree}"),
                NotificationSeverity::Error,
            )
        };
        self.notify(&title, &body, severity, "agent").await
    }

    pub async fn notify_agent_error(&self, agent_name: &str, error: &str) -> Result<NotificationRecord> {
        self.notify(
            "Agent Error",
            &format!("{agent_name}: {error}"),
            NotificationSeverity::Error,
            "agent",
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use porpoise_db::DbPool;
    use tempfile::TempDir;

    use super::*;

    fn make_svc() -> (NotificationService, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = DbPool::open(&dir.path().join("test.db")).unwrap();
        porpoise_db::migration::run_migrations(&db).unwrap();
        (NotificationService::new(db), dir)
    }

    #[tokio::test]
    async fn test_notify_and_list() {
        let (svc, _dir) = make_svc();

        let prefs = NotificationPreferences {
            desktop_notifications: false,
            ..NotificationPreferences::defaults()
        };
        svc.set_preferences(prefs).await;

        svc.notify("Test", "body", NotificationSeverity::Info, "system")
            .await
            .unwrap();
        svc.notify("Test2", "body2", NotificationSeverity::Warning, "system")
            .await
            .unwrap();

        let unread = svc.list_unread().await.unwrap();
        assert_eq!(unread.len(), 2);

        svc.mark_all_read().await.unwrap();
        let unread = svc.list_unread().await.unwrap();
        assert_eq!(unread.len(), 0);
    }

    #[tokio::test]
    async fn test_unread_count() {
        let (svc, _dir) = make_svc();
        svc.set_preferences(NotificationPreferences {
            desktop_notifications: false,
            ..NotificationPreferences::defaults()
        })
        .await;

        assert_eq!(svc.unread_count().await.unwrap(), 0);
        svc.notify("A", "a", NotificationSeverity::Info, "s").await.unwrap();
        assert_eq!(svc.unread_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_agent_completion_notification() {
        let (svc, _dir) = make_svc();
        svc.set_preferences(NotificationPreferences {
            desktop_notifications: false,
            ..NotificationPreferences::defaults()
        })
        .await;

        let rec = svc.notify_agent_completion("claude", "/tmp/wt", 0).await.unwrap();
        assert_eq!(rec.source, "agent");
        assert!(rec.title.contains("Completed"));

        let rec = svc.notify_agent_completion("claude", "/tmp/wt", 1).await.unwrap();
        assert!(rec.title.contains("Failed"));
    }
}
