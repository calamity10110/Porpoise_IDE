use porpoise_core::error::{PorpoiseError, Result};
use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct TerminalRow {
    pub id: String,
    pub session_id: String,
    pub rows: i32,
    pub cols: i32,
    pub shell: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct HistoryRow {
    pub id: i64,
    pub terminal_id: String,
    pub timestamp: String,
    pub data: Vec<u8>,
    pub row_start: Option<i32>,
    pub row_end: Option<i32>,
}

impl TerminalRow {
    pub fn insert(pool: &DbPool, row: &TerminalRow) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO terminals (id, session_id, rows, cols, shell) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![row.id, row.session_id, row.rows, row.cols, row.shell],
        ).map_err(|e| PorpoiseError::Db(format!("insert terminal failed: {e}")))?;
        Ok(())
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, rows, cols, shell, created_at FROM terminals WHERE id = ?1"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(TerminalRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                rows: row.get(2)?,
                cols: row.get(3)?,
                shell: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        match rows.next() {
            Some(Ok(row)) => Ok(Some(row)),
            Some(Err(e)) => Err(PorpoiseError::Db(format!("row read failed: {e}"))),
            None => Ok(None),
        }
    }

    pub fn list_by_session(pool: &DbPool, session_id: &str) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, rows, cols, shell, created_at FROM terminals WHERE session_id = ?1"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt.query_map(rusqlite::params![session_id], |row| {
            Ok(TerminalRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                rows: row.get(2)?,
                cols: row.get(3)?,
                shell: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }
}

impl HistoryRow {
    pub fn insert(pool: &DbPool, terminal_id: &str, data: &[u8], row_start: Option<i32>, row_end: Option<i32>) -> Result<i64> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO terminal_history (terminal_id, data, row_start, row_end) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![terminal_id, data, row_start, row_end],
        ).map_err(|e| PorpoiseError::Db(format!("insert history failed: {e}")))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn query_by_terminal(pool: &DbPool, terminal_id: &str, limit: i64) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, terminal_id, timestamp, data, row_start, row_end FROM terminal_history WHERE terminal_id = ?1 ORDER BY id DESC LIMIT ?2"
        ).map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt.query_map(rusqlite::params![terminal_id, limit], |row| {
            Ok(HistoryRow {
                id: row.get(0)?,
                terminal_id: row.get(1)?,
                timestamp: row.get(2)?,
                data: row.get(3)?,
                row_start: row.get(4)?,
                row_end: row.get(5)?,
            })
        }).map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }
}
