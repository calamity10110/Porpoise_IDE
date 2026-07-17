use std::sync::Arc;

use chrono::Utc;
use porpoise_core::{
    error::{PorpoiseError, Result},
    types::event::NotificationSeverity,
};
use porpoise_db::DbPool;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

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
}

impl NotificationService {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            prefs: Arc::new(Mutex::new(NotificationPreferences::defaults())),
        }
    }

    pub async fn notify(
        &self,
        title: &str,
        body: &str,
        severity: NotificationSeverity,
        source: &str,
    ) -> Result<NotificationRecord> {
        let record = NotificationRecord {
            id: format!("notif_{}", Utc::now().timestamp_millis()),
            title: title.to_string(),
            body: body.to_string(),
            severity: serde_json::to_string(&severity)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            source: source.to_string(),
            timestamp: Utc::now().timestamp_millis(),
            read: false,
        };

        self.store(&record).await?;

        let prefs = self.prefs.lock().await;
        if prefs.desktop_notifications && self.should_notify(severity.clone(), &prefs) {
            self.show_desktop_notification(&record);
        }

        Ok(record)
    }

    async fn store(&self, record: &NotificationRecord) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
        conn.execute(
            "INSERT INTO notifications (id, title, message, level, source, created_at, read)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                record.id,
                record.title,
                record.body,
                record.severity,
                record.source,
                record.timestamp,
                record.read as i32,
            ],
        )
        .map_err(|e| PorpoiseError::Internal(format!("insert notification: {e}")))?;
        Ok(())
    }

    pub async fn list_unread(&self) -> Result<Vec<NotificationRecord>> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
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
    }

    pub async fn list_all(&self, limit: usize) -> Result<Vec<NotificationRecord>> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
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
    }

    pub async fn mark_read(&self, id: &str) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
        conn.execute("UPDATE notifications SET read = 1 WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| PorpoiseError::Internal(format!("mark read: {e}")))?;
        Ok(())
    }

    pub async fn mark_all_read(&self) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
        conn.execute("UPDATE notifications SET read = 1", [])
            .map_err(|e| PorpoiseError::Internal(format!("mark all read: {e}")))?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
        conn.execute("DELETE FROM notifications", [])
            .map_err(|e| PorpoiseError::Internal(format!("clear: {e}")))?;
        Ok(())
    }

    pub async fn unread_count(&self) -> Result<usize> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Internal(format!("db pool: {e}")))?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notifications WHERE read = 0", [], |row| {
                row.get(0)
            })
            .map_err(|e| PorpoiseError::Internal(format!("count: {e}")))?;
        Ok(count as usize)
    }

    pub async fn set_preferences(&self, prefs: NotificationPreferences) {
        *self.prefs.lock().await = prefs;
    }

    pub async fn get_preferences(&self) -> NotificationPreferences {
        self.prefs.lock().await.clone()
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
