use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[derive(Clone)]
pub struct DbPool {
    pool: Pool<SqliteConnectionManager>,
}

impl DbPool {
    pub fn open(db_path: &Path) -> Result<Self, porpoise_core::PorpoiseError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| porpoise_core::PorpoiseError::Db(format!("cannot create data dir: {e}")))?;
        }

        let manager = SqliteConnectionManager::file(db_path);
        let pool = Pool::builder()
            .max_size(4)
            .build(manager)
            .map_err(|e| porpoise_core::PorpoiseError::Db(format!("pool creation failed: {e}")))?;

        {
            let conn = pool
                .get()
                .map_err(|e| porpoise_core::PorpoiseError::Db(format!("connection failed: {e}")))?;
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;",
            )
            .map_err(|e| porpoise_core::PorpoiseError::Db(format!("pragma setup failed: {e}")))?;
        }

        Ok(Self { pool })
    }

    pub fn get(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, porpoise_core::PorpoiseError> {
        self.pool
            .get()
            .map_err(|e| porpoise_core::PorpoiseError::Db(format!("connection pool error: {e}")))
    }

    pub fn inner(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }
}

impl std::fmt::Debug for DbPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbPool")
            .field("max_size", &self.pool.max_size())
            .finish()
    }
}
