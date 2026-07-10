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

fn all_migrations() -> Vec<Box<dyn Migration>> {
    vec![Box::new(InitialSchema)]
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
