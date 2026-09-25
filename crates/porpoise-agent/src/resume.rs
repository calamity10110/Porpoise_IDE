use std::path::PathBuf;

use chrono::{DateTime, Utc};
use porpoise_core::error::{PorpoiseError, Result};
use porpoise_db::DbPool;
use serde::{Deserialize, Serialize};

use crate::types::AgentKind;

/// Persisted agent session data for resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub agent_kind: String,
    pub worktree_path: PathBuf,
    pub prompt: String,
    pub output_log_path: Option<PathBuf>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
}

/// SQLite-backed session store for agent resume.
pub struct SessionStore {
    db: DbPool,
}

impl SessionStore {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Helper to convert session timestamp string (rfc3339 or epoch millis) to i64.
    fn ts_to_millis(ts: &str) -> i64 {
        ts.parse::<i64>().unwrap_or_else(|_| {
            DateTime::parse_from_rfc3339(ts)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or_else(|_| Utc::now().timestamp_millis())
        })
    }

    pub fn save_session(&self, record: &SessionRecord) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Agent(format!("db pool: {e}")))?;
        conn.execute(
            "INSERT OR REPLACE INTO agent_sessions
                (session_id, agent_id, worktree_id, started_at, ended_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                record.session_id,
                record.agent_kind,
                record.worktree_path.to_string_lossy(),
                Self::ts_to_millis(&record.started_at),
                record.ended_at.as_ref().map(|ts| Self::ts_to_millis(ts)),
                serde_json::to_string(&record.status).unwrap_or_default(),
            ],
        )
        .map_err(|e| PorpoiseError::Agent(format!("save session: {e}")))?;
        Ok(())
    }

    pub fn load_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Agent(format!("db pool: {e}")))?;
        let mut stmt = conn
            .prepare(
                "SELECT session_id, agent_id, worktree_id, started_at, ended_at, status
                 FROM agent_sessions WHERE session_id = ?1",
            )
            .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

        let result = stmt.query_row(rusqlite::params![session_id], |row| {
            let status_str: String = row.get(5)?;
            let status = serde_json::from_str(&status_str).unwrap_or(SessionStatus::Active);
            Ok(SessionRecord {
                session_id: row.get(0)?,
                agent_kind: row.get(1)?,
                worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                prompt: String::new(),
                output_log_path: None,
                started_at: row.get::<_, i64>(3)?.to_string(),
                ended_at: row.get::<_, Option<i64>>(4)?.map(|v| v.to_string()),
                status,
            })
        });

        match result {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PorpoiseError::Agent(format!("load session: {e}"))),
        }
    }

    pub fn list_active(&self) -> Result<Vec<SessionRecord>> {
        self.list_by_status(SessionStatus::Active)
    }

    pub fn list_by_status(&self, status: SessionStatus) -> Result<Vec<SessionRecord>> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Agent(format!("db pool: {e}")))?;
        let status_str = serde_json::to_string(&status).unwrap_or_default();
        let mut stmt = conn
            .prepare(
                "SELECT session_id, agent_id, worktree_id, started_at, ended_at, status
                 FROM agent_sessions WHERE status = ?1 ORDER BY started_at DESC",
            )
            .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![status_str], |row| {
                let status_json: String = row.get(5)?;
                let parsed_status: SessionStatus = serde_json::from_str(&status_json).unwrap_or(SessionStatus::Active);
                Ok(SessionRecord {
                    session_id: row.get(0)?,
                    agent_kind: row.get(1)?,
                    worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                    prompt: String::new(),
                    output_log_path: None,
                    started_at: row.get::<_, i64>(3)?.to_string(),
                    ended_at: row.get::<_, Option<i64>>(4)?.map(|v| v.to_string()),
                    status: parsed_status,
                })
            })
            .map_err(|e| PorpoiseError::Agent(format!("query: {e}")))?;

        rows.filter_map(|r| r.ok()).map(Ok).collect()
    }

    pub fn update_status(&self, session_id: &str, status: SessionStatus) -> Result<()> {
        let conn = self
            .db
            .get()
            .map_err(|e| PorpoiseError::Agent(format!("db pool: {e}")))?;
        let status_str = serde_json::to_string(&status).unwrap_or_default();
        let ended_at: Option<i64> = if status == SessionStatus::Completed || status == SessionStatus::Failed {
            Some(Utc::now().timestamp_millis())
        } else {
            None
        };

        conn.execute(
            "UPDATE agent_sessions SET status = ?1, ended_at = COALESCE(?2, ended_at) WHERE session_id = ?3",
            rusqlite::params![status_str, ended_at, session_id],
        )
        .map_err(|e| PorpoiseError::Agent(format!("update status: {e}")))?;
        Ok(())
    }

    pub fn create_session(&self, kind: &AgentKind, worktree: &std::path::Path, prompt: &str) -> Result<SessionRecord> {
        static SESSION_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now_millis = Utc::now().timestamp_millis();
        // millisecond timestamps collide on coarse-granularity clocks (e.g. Windows ~15.6ms);
        // a collision makes INSERT OR REPLACE silently drop the earlier session
        let seq = SESSION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let now = now_millis.to_string();
        let session_id = format!("sess_{now_millis}_{seq}");

        let record = SessionRecord {
            session_id: session_id.clone(),
            agent_kind: kind.to_string(),
            worktree_path: worktree.to_path_buf(),
            prompt: prompt.to_string(),
            output_log_path: None,
            started_at: now,
            ended_at: None,
            status: SessionStatus::Active,
        };

        self.save_session(&record)?;
        Ok(record)
    }

    pub fn resume_session(&self, session_id: &str, kind: &AgentKind) -> Result<Option<SessionRecord>> {
        if let Some(mut record) = self.load_session(session_id)? {
            if record.status == SessionStatus::Active {
                return Ok(Some(record));
            }
            record.status = SessionStatus::Active;
            record.ended_at = None;
            self.save_session(&record)?;
            let _ = kind;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
}

/// Returns the last N completed sessions for an agent kind.
pub fn list_recent_sessions(store: &SessionStore, limit: usize) -> Result<Vec<SessionRecord>> {
    let conn = store
        .db
        .get()
        .map_err(|e| PorpoiseError::Agent(format!("db pool: {e}")))?;
    let mut stmt = conn
        .prepare(
            "SELECT session_id, agent_id, worktree_id, started_at, ended_at, status
         FROM agent_sessions ORDER BY started_at DESC LIMIT ?1",
        )
        .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            let status_json: String = row.get(5)?;
            let status: SessionStatus = serde_json::from_str(&status_json).unwrap_or(SessionStatus::Active);
            Ok(SessionRecord {
                session_id: row.get(0)?,
                agent_kind: row.get(1)?,
                worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                prompt: String::new(),
                output_log_path: None,
                started_at: row.get::<_, i64>(3)?.to_string(),
                ended_at: row.get::<_, Option<i64>>(4)?.map(|v| v.to_string()),
                status,
            })
        })
        .map_err(|e| PorpoiseError::Agent(format!("query: {e}")))?;

    rows.filter_map(|r| r.ok())
        .collect::<Vec<_>>()
        .into_iter()
        .map(Ok)
        .collect()
}

/// Parses an RFC 3339 timestamp to a `DateTime<Utc>`.
pub fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts).ok().map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use porpoise_db::DbPool;
    use tempfile::TempDir;

    use super::*;

    fn make_store() -> (SessionStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = DbPool::open(&dir.path().join("test.db")).unwrap();
        porpoise_db::migration::run_migrations(&db).unwrap();
        (SessionStore::new(db), dir)
    }

    #[test]
    fn test_create_and_load_session() {
        let (store, _dir) = make_store();

        let record = store
            .create_session(&AgentKind::ClaudeCode, Path::new("/tmp/wt"), "fix bug")
            .unwrap();

        assert_eq!(record.status, SessionStatus::Active);

        let loaded = store.load_session(&record.session_id).unwrap().unwrap();
        assert_eq!(loaded.agent_kind, "claude");
    }

    #[test]
    fn test_update_status() {
        let (store, _dir) = make_store();

        let record = store
            .create_session(&AgentKind::Codex, Path::new("/tmp/wt"), "refactor")
            .unwrap();

        store
            .update_status(&record.session_id, SessionStatus::Completed)
            .unwrap();

        let loaded = store.load_session(&record.session_id).unwrap().unwrap();
        assert_eq!(loaded.status, SessionStatus::Completed);
        assert!(loaded.ended_at.is_some());
    }

    #[test]
    fn test_list_active() {
        let (store, _dir) = make_store();

        store
            .create_session(&AgentKind::ClaudeCode, Path::new("/tmp/wt1"), "task 1")
            .unwrap();
        let r2 = store
            .create_session(&AgentKind::Codex, Path::new("/tmp/wt2"), "task 2")
            .unwrap();
        store.update_status(&r2.session_id, SessionStatus::Completed).unwrap();

        let active = store.list_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].agent_kind, "claude");
    }

    #[test]
    fn test_resume_session() {
        let (store, _dir) = make_store();

        let record = store
            .create_session(&AgentKind::ClaudeCode, Path::new("/tmp/wt"), "task")
            .unwrap();
        store.update_status(&record.session_id, SessionStatus::Paused).unwrap();

        let resumed = store
            .resume_session(&record.session_id, &AgentKind::ClaudeCode)
            .unwrap()
            .unwrap();
        assert_eq!(resumed.status, SessionStatus::Active);
        assert!(resumed.ended_at.is_none());
    }
}
