use porpoise_core::error::{PorpoiseError, Result};
use rusqlite::Connection;

use crate::{pool::DbPool, schema};

pub trait Migration {
    fn version(&self) -> u32;
    fn description(&self) -> &'static str;
    fn up(&self, conn: &Connection) -> Result<()>;
}

struct InitialSchema;

impl Migration for InitialSchema {
    fn version(&self) -> u32 {
        1
    }
    fn description(&self) -> &'static str {
        "initial schema"
    }

    fn up(&self, conn: &Connection) -> Result<()> {
        for table in schema::ALL_TABLES {
            conn.execute(table, [])
                .map_err(|e| PorpoiseError::DbMigration(format!("create table failed: {e}")))?;
        }
        for idx in schema::CREATE_INDEXES {
            conn.execute(idx, [])
                .map_err(|e| PorpoiseError::DbMigration(format!("create index failed: {e}")))?;
        }
        conn.execute("PRAGMA foreign_keys = ON", []).ok();
        conn.execute("PRAGMA journal_mode = WAL", []).ok();
        Ok(())
    }
}

struct MigrationV2;

impl Migration for MigrationV2 {
    fn version(&self) -> u32 {
        2
    }
    fn description(&self) -> &'static str {
        "consolidation tables + generation_id"
    }

    fn up(&self, conn: &Connection) -> Result<()> {
        conn.execute(schema::CREATE_NOTIFICATIONS, [])
            .map_err(|e| PorpoiseError::DbMigration(format!("create notifications: {e}")))?;
        conn.execute(schema::CREATE_SCROLLBACK, [])
            .map_err(|e| PorpoiseError::DbMigration(format!("create scrollback: {e}")))?;
        conn.execute(schema::CREATE_AGENT_SESSIONS, [])
            .map_err(|e| PorpoiseError::DbMigration(format!("create agent_sessions: {e}")))?;
        conn.execute(schema::CREATE_SERVER_METADATA, [])
            .map_err(|e| PorpoiseError::DbMigration(format!("create server_metadata: {e}")))?;

        // ALTER TABLE errors if column exists; catch gracefully
        let alter_result = conn.execute("ALTER TABLE sessions ADD COLUMN generation_id TEXT", []);
        if let Err(e) = alter_result {
            let msg = e.to_string();
            if !msg.contains("duplicate column") && !msg.contains("already exists") {
                return Err(PorpoiseError::DbMigration(format!("alter sessions: {e}")));
            }
        }

        for idx in &[
            "CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at);",
            "CREATE INDEX IF NOT EXISTS idx_scrollback_terminal ON scrollback(terminal_id);",
            "CREATE INDEX IF NOT EXISTS idx_scrollback_generation ON scrollback(generation_id);",
            "CREATE INDEX IF NOT EXISTS idx_agent_sessions_agent ON agent_sessions(agent_id);",
            "CREATE INDEX IF NOT EXISTS idx_sessions_generation ON sessions(generation_id);",
        ] {
            conn.execute(idx, [])
                .map_err(|e| PorpoiseError::DbMigration(format!("create index: {e}")))?;
        }

        Ok(())
    }
}

fn all_migrations() -> Vec<Box<dyn Migration>> {
    vec![Box::new(InitialSchema), Box::new(MigrationV2)]
}

pub fn run_migrations(pool: &DbPool) -> Result<u32> {
    let conn = pool
        .get()
        .map_err(|e| PorpoiseError::DbMigration(format!("failed to get connection: {e}")))?;

    let current: u32 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let migrations = all_migrations();
    let mut latest = current;

    for migration in migrations {
        if migration.version() > current {
            migration.up(&conn)?;
            conn.execute(
                "INSERT INTO schema_version (version, description) VALUES (?1, ?2)",
                rusqlite::params![migration.version(), migration.description()],
            )
            .map_err(|e| PorpoiseError::DbMigration(format!("version insert failed: {e}")))?;
            latest = migration.version();
        }
    }

    Ok(latest)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn create_pool() -> DbPool {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        DbPool::open(&db_path).unwrap()
    }

    #[test]
    fn test_migration_v2_creates_tables() {
        let pool = create_pool();
        run_migrations(&pool).unwrap();

        let conn = pool.get().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(
            tables.contains(&"notifications".to_string()),
            "notifications table missing"
        );
        assert!(tables.contains(&"scrollback".to_string()), "scrollback table missing");
        assert!(
            tables.contains(&"agent_sessions".to_string()),
            "agent_sessions table missing"
        );
        assert!(
            tables.contains(&"server_metadata".to_string()),
            "server_metadata table missing"
        );
    }

    #[test]
    fn test_migration_v2_idempotent() {
        let pool = create_pool();
        run_migrations(&pool).unwrap();
        run_migrations(&pool).unwrap();
    }

    #[test]
    fn test_generation_id_column_added() {
        let pool = create_pool();
        run_migrations(&pool).unwrap();

        let conn = pool.get().unwrap();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(sessions)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(
            cols.contains(&"generation_id".to_string()),
            "generation_id column missing on sessions"
        );
    }
}
