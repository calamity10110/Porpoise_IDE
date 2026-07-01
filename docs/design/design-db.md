# Module Design: porpoise-db

> SQLite persistence layer — all durable state lives here.

---

## Purpose

`porpoise-db` provides the database schema, migration system, and CRUD operations for all persistent Porpoise data: worktrees, sessions, terminals, terminal history, agents, configuration entries, and audit events.

**What problem it solves:** Without persistence, all Porpoise state would be lost on restart — worktree metadata, terminal scrollback, agent session coordinates. SQLite provides a self-contained, zero-configuration, cross-platform database that requires no external services.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `rusqlite` (or `sqlx`) | 0.32+ | SQLite bindings with compile-time checked queries (sqlx) or simple API (rusqlite) |
| `r2d2` | 0.8 | Connection pooling |
| `r2d2_sqlite` | 0.25 | r2d2 adapter for rusqlite |
| `serde` / `serde_json` | 1.x | JSON serialization for config values in DB |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Import — `PorpoiseError::Db*`, DB config types |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| Database URL/path | `AppConfig::DbConfig.url` | Config loading | `PathBuf` / connection string |
| SQL migration files | Embedded in binary | `include_str!()` at compile time | SQL text |
| CRUD parameters | Service crates (server, runtime, etc.) | Function calls with typed params | Typed structs |
| Terminal output bytes | `porpoise-terminal` / `porpoise-runtime` | `insert_history()` | `Vec<u8>` with row metadata |
| Config key-value writes | Server / CLI | `set_config(key, value)` | String key + BLOB value |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| Worktree records | Server worktree service | `query()` / `query_one()` | `WorktreeRow` struct |
| Session history | Server terminal service | `query()` | `Vec<SessionRow>` |
| Terminal scrollback | Server terminal service | `query_history()` | `Vec<HistoryRow>` |
| Agent metadata | Server agent service | `query()` | `Vec<AgentRow>` |
| Config values | Server config service | `get_config(key)` | `Option<Vec<u8>>` |
| Migration status | Server startup | `current_version()` | `u32` schema version |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| SQLite database file | Process state |
| Connection pool | Event bus |
| Schema definitions | Business logic invariants |
| Migration scripts | Authentication |
| CRUD query logic | Session tokens |
| WAL mode configuration | Any in-memory state |

---

## Program Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Database Initialization Flow                      │
│                                                                      │
│  Server starts                                                       │
│    │                                                                 │
│    ▼                                                                 │
│  discover_config_path() → get DbConfig.url                           │
│    │                                                                 │
│    ▼                                                                 │
│  DbPool::open() ───────────────────────────────┐                    │
│    │                                            │                    │
│    ├── Create data dir if not exists            │                    │
│    ├── Open SQLite file with WAL mode           │                    │
│    ├── Set busy_timeout (5s)                    │                    │
│    ├── Set foreign_keys = ON                    │                    │
│    ├── Acquire pool (4 connections)            │                    │
│    │                                            │                    │
│    ▼                                            ▼                    │
│  Migration::run()                              DbPool ready          │
│    │                                            │                    │
│    ├── Check schema_version from pragma         │                    │
│    ├── Apply pending migrations in order        │                    │
│    ├── Update schema_version                    │                    │
│    └── Return current version                   │                    │
│                                                 ▼                    │
│                                       Server uses DbPool             │
│                                       for all CRUD operations        │
└─────────────────────────────────────────────────────────────────────┘
```

### CRUD Lifecycle

```
Service needs data → acquire connection from pool → execute query → return result
                                 │
                      ┌──────────┴──────────┐
                      ▼                      ▼
                 Read query              Write query
                 (SELECT)                (INSERT/UPDATE/DELETE)
                      │                      │
                      ▼                      ▼
              Deserialize row             Check rows_affected
              → return struct             → return Ok/()

              Connection returned to pool on drop (RAII)
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | `PorpoiseError::Db*`, `DbConfig` | core → db |
| porpoise-server | Direct call | `DbPool` owned by server, accessed by services | server ↔ db |
| porpoise-runtime | Via server | Process metadata storage | server → db |
| porpoise-terminal | Via server | Scrollback persistence | server → db |
| porpoise-agent | Via server | Agent session persistence | server → db |
| porpoise-cli | Indirect | All CLI queries go through server IPC | cli → server → db |
| porpoise-app | Indirect | Same as CLI but via Tauri | app → server → db |

**Important:** Only `porpoise-server` holds the `DbPool` reference. Other crates never import or call porpoise-db directly. This ensures the server is the sole database authority, preventing concurrent access issues.

---

## Schema

```sql
-- Schema version tracking
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);

-- Worktrees
CREATE TABLE worktrees (
    id          TEXT PRIMARY KEY,
    repo_path   TEXT NOT NULL,
    worktree_path TEXT NOT NULL,
    branch      TEXT NOT NULL,
    base_ref    TEXT,
    agent_id    TEXT,
    status      TEXT NOT NULL DEFAULT 'idle',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    exit_code   INTEGER
);

-- Terminals
CREATE TABLE terminals (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    rows        INTEGER NOT NULL DEFAULT 24,
    cols        INTEGER NOT NULL DEFAULT 80,
    shell       TEXT NOT NULL DEFAULT 'bash',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Terminal scrollback (append-only)
CREATE TABLE terminal_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    terminal_id TEXT NOT NULL REFERENCES terminals(id) ON DELETE CASCADE,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    data        BLOB NOT NULL,
    row_start   INTEGER,
    row_end     INTEGER
);

-- Agents
CREATE TABLE agents (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    pid         INTEGER,
    status      TEXT NOT NULL DEFAULT 'spawning',
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    stopped_at  TEXT
);

-- Config key-value store
CREATE TABLE config (
    key         TEXT PRIMARY KEY,
    value       BLOB NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Audit/event log
CREATE TABLE event_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type  TEXT NOT NULL,
    payload     TEXT NOT NULL,
    severity    TEXT NOT NULL DEFAULT 'info',  -- info, warn, error
    timestamp   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sessions_worktree ON sessions(worktree_id);
CREATE INDEX idx_terminals_session ON terminals(session_id);
CREATE INDEX idx_terminal_history_time ON terminal_history(timestamp);
CREATE INDEX idx_terminal_history_terminal ON terminal_history(terminal_id);
CREATE INDEX idx_agents_worktree ON agents(worktree_id);
CREATE INDEX idx_event_log_type ON event_log(event_type);
CREATE INDEX idx_event_log_time ON event_log(timestamp);
```

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| WAL mode | Concurrent reads never block; writes don't block reads. Critical for server responsiveness |
| Foreign keys ON | Referential integrity enforced by database, not application code |
| ISO-8601 text dates | Human-readable, sortable, no timezone ambiguity vs INTEGER timestamps |
| BLOB for terminal history | Terminal output is raw bytes (might include escape sequences); TEXT would mangle them |
| ROWID auto-increment for history | Append-only insert pattern; integer PK is smaller and faster than UUID for high-volume data |
| Connection pool | Prevents concurrent request queue buildup; 4 connections handles expected load |
| Migration system | Schema changes are versioned, repeatable, and tested. No ad-hoc ALTER TABLE in production |

---

*The database is append-heavy (terminal history) and read-moderate (worktree/session queries). Monitor WAL file size for long-running sessions — periodic checkpoint is essential. See [ROADMAP.md](../ROADMAP.md) Phase 8 for performance tuning.*
