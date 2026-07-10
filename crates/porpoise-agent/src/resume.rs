use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use porpoise_core::error::{PorpoiseError, Result};
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
    conn: rusqlite::Connection,
}

impl SessionStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| PorpoiseError::Agent(format!("open session store: {e}")))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_sessions (
                id          TEXT PRIMARY KEY,
                agent_kind  TEXT NOT NULL,
                worktree    TEXT NOT NULL,
                prompt      TEXT,
                output_log  TEXT,
                started_at  TEXT NOT NULL,
                ended_at    TEXT,
                status      TEXT NOT NULL DEFAULT 'active'
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_status ON agent_sessions(status);
            CREATE INDEX IF NOT EXISTS idx_sessions_kind ON agent_sessions(agent_kind);",
        )
        .map_err(|e| PorpoiseError::Agent(format!("create sessions table: {e}")))?;

        Ok(Self { conn })
    }

    pub fn save_session(&self, record: &SessionRecord) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO agent_sessions
                    (id, agent_kind, worktree, prompt, output_log, started_at, ended_at, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    record.session_id,
                    record.agent_kind,
                    record.worktree_path.to_string_lossy(),
                    record.prompt,
                    record.output_log_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    record.started_at,
                    record.ended_at,
                    serde_json::to_string(&record.status).unwrap_or_default(),
                ],
            )
            .map_err(|e| PorpoiseError::Agent(format!("save session: {e}")))?;
        Ok(())
    }

    pub fn load_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, agent_kind, worktree, prompt, output_log, started_at, ended_at, status
                 FROM agent_sessions WHERE id = ?1",
            )
            .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

        let result = stmt.query_row(rusqlite::params![session_id], |row| {
            let status_str: String = row.get(7)?;
            let status = serde_json::from_str(&status_str).unwrap_or(SessionStatus::Active);
            Ok(SessionRecord {
                session_id: row.get(0)?,
                agent_kind: row.get(1)?,
                worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                prompt: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                output_log_path: row.get::<_, Option<String>>(4)?.map(PathBuf::from),
                started_at: row.get(5)?,
                ended_at: row.get(6)?,
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
        let status_str = serde_json::to_string(&status).unwrap_or_default();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, agent_kind, worktree, prompt, output_log, started_at, ended_at, status
                 FROM agent_sessions WHERE status = ?1 ORDER BY started_at DESC",
            )
            .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![status_str], |row| {
                let status_json: String = row.get(7)?;
                let parsed_status: SessionStatus = serde_json::from_str(&status_json).unwrap_or(SessionStatus::Active);
                Ok(SessionRecord {
                    session_id: row.get(0)?,
                    agent_kind: row.get(1)?,
                    worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                    prompt: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    output_log_path: row.get::<_, Option<String>>(4)?.map(PathBuf::from),
                    started_at: row.get(5)?,
                    ended_at: row.get(6)?,
                    status: parsed_status,
                })
            })
            .map_err(|e| PorpoiseError::Agent(format!("query: {e}")))?;

        rows.filter_map(|r| r.ok()).map(Ok).collect()
    }

    pub fn update_status(&self, session_id: &str, status: SessionStatus) -> Result<()> {
        let status_str = serde_json::to_string(&status).unwrap_or_default();
        let ended_at = if status == SessionStatus::Completed || status == SessionStatus::Failed {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        self.conn
            .execute(
                "UPDATE agent_sessions SET status = ?1, ended_at = COALESCE(?2, ended_at) WHERE id = ?3",
                rusqlite::params![status_str, ended_at, session_id],
            )
            .map_err(|e| PorpoiseError::Agent(format!("update status: {e}")))?;
        Ok(())
    }

    pub fn create_session(&self, kind: &AgentKind, worktree: &Path, prompt: &str) -> Result<SessionRecord> {
        let now = Utc::now().to_rfc3339();
        let session_id = format!("sess_{}", chrono::Utc::now().timestamp_millis());

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
    let mut stmt = store
        .conn
        .prepare(
            "SELECT id, agent_kind, worktree, prompt, output_log, started_at, ended_at, status
         FROM agent_sessions ORDER BY started_at DESC LIMIT ?1",
        )
        .map_err(|e| PorpoiseError::Agent(format!("prepare: {e}")))?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            let status_json: String = row.get(7)?;
            let status: SessionStatus = serde_json::from_str(&status_json).unwrap_or(SessionStatus::Active);
            Ok(SessionRecord {
                session_id: row.get(0)?,
                agent_kind: row.get(1)?,
                worktree_path: PathBuf::from(row.get::<_, String>(2)?),
                prompt: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                output_log_path: row.get::<_, Option<String>>(4)?.map(PathBuf::from),
                started_at: row.get(5)?,
                ended_at: row.get(6)?,
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
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_and_load_session() {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::open(&dir.path().join("sessions.db")).unwrap();

        let record = store
            .create_session(&AgentKind::ClaudeCode, Path::new("/tmp/wt"), "fix bug")
            .unwrap();

        assert_eq!(record.status, SessionStatus::Active);

        let loaded = store.load_session(&record.session_id).unwrap().unwrap();
        assert_eq!(loaded.agent_kind, "claude");
        assert_eq!(loaded.prompt, "fix bug");
    }

    #[test]
    fn test_update_status() {
        let dir = TempDir::new().unwrap();
        let store = SessionStore::open(&dir.path().join("sessions.db")).unwrap();

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
        let dir = TempDir::new().unwrap();
        let store = SessionStore::open(&dir.path().join("sessions.db")).unwrap();

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
        let dir = TempDir::new().unwrap();
        let store = SessionStore::open(&dir.path().join("sessions.db")).unwrap();

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
