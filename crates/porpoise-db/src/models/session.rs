use porpoise_core::error::{PorpoiseError, Result};
use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    pub worktree_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub exit_code: Option<i32>,
}

impl SessionRow {
    pub fn insert(pool: &DbPool, row: &SessionRow) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO sessions (id, worktree_id, started_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![row.id, row.worktree_id, row.started_at],
        ).map_err(|e| PorpoiseError::Db(format!("insert session failed: {e}")))?;
        Ok(())
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, worktree_id, started_at, ended_at, exit_code FROM sessions WHERE id = ?1"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                worktree_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                exit_code: row.get(4)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(PorpoiseError::Db(format!("row read failed: {e}"))),
            None => Ok(None),
        }
    }

    pub fn list_by_worktree(pool: &DbPool, worktree_id: &str) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, worktree_id, started_at, ended_at, exit_code FROM sessions WHERE worktree_id = ?1 ORDER BY started_at DESC"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt.query_map(rusqlite::params![worktree_id], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                worktree_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                exit_code: row.get(4)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }

    pub fn end_session(pool: &DbPool, id: &str, exit_code: i32) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "UPDATE sessions SET ended_at = datetime('now'), exit_code = ?1 WHERE id = ?2",
            rusqlite::params![exit_code, id],
        ).map_err(|e| PorpoiseError::Db(format!("end session failed: {e}")))?;
        Ok(())
    }
}
