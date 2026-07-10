use porpoise_core::error::{PorpoiseError, Result};

use crate::pool::DbPool;

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub updated_at: String,
}

impl ConfigEntry {
    pub fn get(pool: &DbPool, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = pool.get()?;
        let mut stmt = conn
            .prepare("SELECT value FROM config WHERE key = ?1")
            .map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let mut rows = stmt
            .query_map(rusqlite::params![key], |row| row.get::<_, Vec<u8>>(0))
            .map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            Some(Err(e)) => Err(PorpoiseError::Db(format!("row read failed: {e}"))),
            None => Ok(None),
        }
    }

    pub fn set(pool: &DbPool, key: &str, value: &[u8]) -> Result<()> {
        let conn = pool.get()?;
        conn.execute(
            "INSERT INTO config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            rusqlite::params![key, value],
        )
        .map_err(|e| PorpoiseError::Db(format!("set config failed: {e}")))?;
        Ok(())
    }

    pub fn list(pool: &DbPool) -> Result<Vec<Self>> {
        let conn = pool.get()?;
        let mut stmt = conn
            .prepare("SELECT key, value, updated_at FROM config ORDER BY key")
            .map_err(|e| PorpoiseError::Db(format!("prepare failed: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ConfigEntry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })
            .map_err(|e| PorpoiseError::Db(format!("query failed: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| PorpoiseError::Db(format!("row read failed: {e}")))?);
        }
        Ok(result)
    }

    pub fn delete(pool: &DbPool, key: &str) -> Result<()> {
        let conn = pool.get()?;
        conn.execute("DELETE FROM config WHERE key = ?1", rusqlite::params![key])
            .map_err(|e| PorpoiseError::Db(format!("delete config failed: {e}")))?;
        Ok(())
    }
}
