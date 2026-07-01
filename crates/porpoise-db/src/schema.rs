pub const CREATE_SCHEMA_VERSION: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);";

pub const CREATE_WORKTREES: &str = "
CREATE TABLE IF NOT EXISTS worktrees (
    id          TEXT PRIMARY KEY,
    repo_path   TEXT NOT NULL,
    worktree_path TEXT NOT NULL,
    branch      TEXT NOT NULL,
    base_ref    TEXT,
    agent_id    TEXT,
    status      TEXT NOT NULL DEFAULT 'idle',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_SESSIONS: &str = "
CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    exit_code   INTEGER
);";

pub const CREATE_TERMINALS: &str = "
CREATE TABLE IF NOT EXISTS terminals (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    rows        INTEGER NOT NULL DEFAULT 24,
    cols        INTEGER NOT NULL DEFAULT 80,
    shell       TEXT NOT NULL DEFAULT 'bash',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_TERMINAL_HISTORY: &str = "
CREATE TABLE IF NOT EXISTS terminal_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    terminal_id TEXT NOT NULL REFERENCES terminals(id) ON DELETE CASCADE,
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    data        BLOB NOT NULL,
    row_start   INTEGER,
    row_end     INTEGER
);";

pub const CREATE_AGENTS: &str = "
CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    pid         INTEGER,
    status      TEXT NOT NULL DEFAULT 'spawning',
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    stopped_at  TEXT
);";

pub const CREATE_CONFIG: &str = "
CREATE TABLE IF NOT EXISTS config (
    key         TEXT PRIMARY KEY,
    value       BLOB NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_EVENT_LOG: &str = "
CREATE TABLE IF NOT EXISTS event_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type  TEXT NOT NULL,
    payload     TEXT NOT NULL,
    severity    TEXT NOT NULL DEFAULT 'info',
    timestamp   TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_sessions_worktree ON sessions(worktree_id);",
    "CREATE INDEX IF NOT EXISTS idx_terminals_session ON terminals(session_id);",
    "CREATE INDEX IF NOT EXISTS idx_terminal_history_time ON terminal_history(timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_terminal_history_terminal ON terminal_history(terminal_id);",
    "CREATE INDEX IF NOT EXISTS idx_agents_worktree ON agents(worktree_id);",
    "CREATE INDEX IF NOT EXISTS idx_event_log_type ON event_log(event_type);",
    "CREATE INDEX IF NOT EXISTS idx_event_log_time ON event_log(timestamp);",
];

pub const ALL_TABLES: &[&str] = &[
    CREATE_SCHEMA_VERSION,
    CREATE_WORKTREES,
    CREATE_SESSIONS,
    CREATE_TERMINALS,
    CREATE_TERMINAL_HISTORY,
    CREATE_AGENTS,
    CREATE_CONFIG,
    CREATE_EVENT_LOG,
];
