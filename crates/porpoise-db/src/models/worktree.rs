use porpoise_core::error::{PorpoiseError, Result};
use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct WorktreeRow {
    pub id: String,
    pub repo_path: String,
    pub worktree_path: String,
    pub branch: String,
    pub base_ref: Option<String>,
    pub agent_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl WorktreeRow {
    pub fn insert(pool: &DbPool, row: &WorktreeRow) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO worktrees (id, repo_path, worktree_path, branch, base_ref, agent_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                row.id, row.repo_path, row.worktree_path, row.branch,
                row.base_ref, row.agent_id, row.status,
            ],
        ).map_err(|e| PorpoiseError::Db(format!("insert worktree failed: {e}")))?;
        Ok(())
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, repo_path, worktree_path, branch, base_ref, agent_id, status, created_at, updated_at
             FROM worktrees WHERE id = ?1"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(WorktreeRow {
                id: row.get(0)?,
                repo_path: row.get(1)?,
                worktree_path: row.get(2)?,
                branch: row.get(3)?,
                base_ref: row.get(4)?,
                agent_id: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(PorpoiseError::Db(format!("row read failed: {e}"))),
            None => Ok(None),
        }
    }

    pub fn list(pool: &DbPool) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, repo_path, worktree_path, branch, base_ref, agent_id, status, created_at, updated_at
             FROM worktrees ORDER BY created_at DESC"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt.query_map([], |row| {
            Ok(WorktreeRow {
                id: row.get(0)?,
                repo_path: row.get(1)?,
                worktree_path: row.get(2)?,
                branch: row.get(3)?,
                base_ref: row.get(4)?,
                agent_id: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }

    pub fn update_status(pool: &DbPool, id: &str, status: &str) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "UPDATE worktrees SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![status, id],
        ).map_err(|e| PorpoiseError::Db(format!("update status failed: {e}")))?;
        Ok(())
    }

    pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
        let conn = pool.get()?;
        conn.execute("DELETE FROM worktrees WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| PorpoiseError::Db(format!("delete failed: {e}")))?;
        Ok(())
    }
}
