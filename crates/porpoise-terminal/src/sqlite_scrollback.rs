use std::path::Path;

use porpoise_core::error::{PorpoiseError, Result};
use rusqlite::{Connection, params};

use crate::types::OutputLine;

/// SQLite-backed scrollback persistence.
///
/// Stores terminal output lines in a SQLite database table `scrollback`.
/// Each row contains the terminal ID, line text, OSC flag, and timestamp.
/// Provides batched writes and efficient range queries by timestamp.
pub struct SqliteScrollbackStore {
    conn: Connection,
}

impl SqliteScrollbackStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn =
            Connection::open(db_path).map_err(|e| PorpoiseError::Terminal(format!("open scrollback db: {e}")))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scrollback (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                terminal_id TEXT NOT NULL,
                text        TEXT NOT NULL,
                is_osc      INTEGER NOT NULL DEFAULT 0,
                timestamp   INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_scrollback_terminal ON scrollback(terminal_id);
            CREATE INDEX IF NOT EXISTS idx_scrollback_time ON scrollback(timestamp);",
        )
        .map_err(|e| PorpoiseError::Terminal(format!("create scrollback table: {e}")))?;

        Ok(Self { conn })
    }

    pub fn store_line(&self, terminal_id: &str, line: &OutputLine) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO scrollback (terminal_id, text, is_osc, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![terminal_id, line.text, line.is_osc as i32, line.timestamp],
            )
            .map_err(|e| PorpoiseError::Terminal(format!("insert scrollback: {e}")))?;
        Ok(())
    }

    pub fn store_batch(&mut self, terminal_id: &str, lines: &[OutputLine]) -> Result<()> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| PorpoiseError::Terminal(format!("begin tx: {e}")))?;

        for line in lines {
            tx.execute(
                "INSERT INTO scrollback (terminal_id, text, is_osc, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![terminal_id, line.text, line.is_osc as i32, line.timestamp],
            )
            .map_err(|e| PorpoiseError::Terminal(format!("batch insert: {e}")))?;
        }

        tx.commit()
            .map_err(|e| PorpoiseError::Terminal(format!("commit: {e}")))?;
        Ok(())
    }

    pub fn load(&self, terminal_id: &str, limit: usize) -> Result<Vec<OutputLine>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT text, is_osc, timestamp FROM scrollback WHERE terminal_id = ?1 ORDER BY timestamp DESC LIMIT ?2",
            )
            .map_err(|e| PorpoiseError::Terminal(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map(params![terminal_id, limit as i64], |row| {
                Ok(OutputLine {
                    text: row.get(0)?,
                    is_osc: row.get::<_, i32>(1)? != 0,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| PorpoiseError::Terminal(format!("query: {e}")))?;

        let mut lines: Vec<OutputLine> = rows.filter_map(|r| r.ok()).collect();
        lines.reverse();
        Ok(lines)
    }

    pub fn clear(&self, terminal_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM scrollback WHERE terminal_id = ?1", params![terminal_id])
            .map_err(|e| PorpoiseError::Terminal(format!("clear: {e}")))?;
        Ok(())
    }

    pub fn line_count(&self, terminal_id: &str) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM scrollback WHERE terminal_id = ?1",
                params![terminal_id],
                |row| row.get(0),
            )
            .map_err(|e| PorpoiseError::Terminal(format!("count: {e}")))?;
        Ok(count as usize)
    }

    pub fn prune(&self, terminal_id: &str, keep_count: usize) -> Result<usize> {
        let deleted = self
            .conn
            .execute(
                "DELETE FROM scrollback WHERE terminal_id = ?1 AND id NOT IN (
                    SELECT id FROM scrollback WHERE terminal_id = ?1
                    ORDER BY id DESC LIMIT ?2
                )",
                params![terminal_id, keep_count as i64],
            )
            .map_err(|e| PorpoiseError::Terminal(format!("prune: {e}")))?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_store_and_load() {
        let dir = TempDir::new().unwrap();
        let mut store = SqliteScrollbackStore::open(&dir.path().join("test.db")).unwrap();

        let line1 = OutputLine {
            text: "hello".into(),
            is_osc: false,
            timestamp: 1000,
        };
        let line2 = OutputLine {
            text: "world".into(),
            is_osc: false,
            timestamp: 2000,
        };

        store.store_batch("tm_001", &[line1, line2]).unwrap();
        let loaded = store.load("tm_001", 100).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].text, "hello");
        assert_eq!(loaded[1].text, "world");
    }

    #[test]
    fn test_clear_and_count() {
        let dir = TempDir::new().unwrap();
        let store = SqliteScrollbackStore::open(&dir.path().join("test.db")).unwrap();

        store
            .store_line(
                "tm_001",
                &OutputLine {
                    text: "a".into(),
                    is_osc: false,
                    timestamp: 1,
                },
            )
            .unwrap();
        assert_eq!(store.line_count("tm_001").unwrap(), 1);
        store.clear("tm_001").unwrap();
        assert_eq!(store.line_count("tm_001").unwrap(), 0);
    }

    #[test]
    fn test_prune() {
        let dir = TempDir::new().unwrap();
        let store = SqliteScrollbackStore::open(&dir.path().join("test.db")).unwrap();

        for i in 0..100 {
            store
                .store_line(
                    "tm_001",
                    &OutputLine {
                        text: format!("line {i}"),
                        is_osc: false,
                        timestamp: i,
                    },
                )
                .unwrap();
        }
        assert_eq!(store.line_count("tm_001").unwrap(), 100);
        let deleted = store.prune("tm_001", 50).unwrap();
        assert_eq!(deleted, 50);
        assert_eq!(store.line_count("tm_001").unwrap(), 50);
    }
}
