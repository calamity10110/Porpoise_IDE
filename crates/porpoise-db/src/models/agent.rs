use porpoise_core::error::{PorpoiseError, Result};

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct AgentRow {
    pub id: String,
    pub worktree_id: String,
    pub kind: String,
    pub pid: Option<i32>,
    pub status: String,
    pub started_at: String,
    pub stopped_at: Option<String>,
}

impl AgentRow {
    pub fn insert(pool: &DbPool, row: &AgentRow) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO agents (id, worktree_id, kind, pid, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![row.id, row.worktree_id, row.kind, row.pid, row.status],
        )
        .map_err(|e| PorpoiseError::Db(format!("insert agent failed: {e}")))?;
        Ok(())
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn
            .prepare("SELECT id, worktree_id, kind, pid, status, started_at, stopped_at FROM agents WHERE id = ?1")
            .map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let mut rows = stmt
            .query_map(rusqlite::params![id], |row| {
                Ok(AgentRow {
                    id: row.get(0)?,
                    worktree_id: row.get(1)?,
                    kind: row.get(2)?,
                    pid: row.get(3)?,
                    status: row.get(4)?,
                    started_at: row.get(5)?,
                    stopped_at: row.get(6)?,
                })
            })
            .map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(PorpoiseError::Db(format!("row read failed: {e}"))),
            None => Ok(None),
        }
    }

    pub fn list_by_worktree(pool: &DbPool, worktree_id: &str) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, worktree_id, kind, pid, status, started_at, stopped_at FROM agents WHERE worktree_id = ?1",
            )
            .map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![worktree_id], |row| {
                Ok(AgentRow {
                    id: row.get(0)?,
                    worktree_id: row.get(1)?,
                    kind: row.get(2)?,
                    pid: row.get(3)?,
                    status: row.get(4)?,
                    started_at: row.get(5)?,
                    stopped_at: row.get(6)?,
                })
            })
            .map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }

    pub fn update_status(pool: &DbPool, id: &str, status: &str) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "UPDATE agents SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id],
        )
        .map_err(|e| PorpoiseError::Db(format!("update agent status failed: {e}")))?;
        Ok(())
    }

    pub fn stop_agent(pool: &DbPool, id: &str) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "UPDATE agents SET status = 'stopped', stopped_at = datetime('now') WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| PorpoiseError::Db(format!("stop agent failed: {e}")))?;
        Ok(())
    }
}
