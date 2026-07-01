# porpoise-core: Core Type System

> Foundation types, traits, errors, and configuration for the Porpoise system.
> Every other crate depends on `porpoise-core` for its domain primitives.

---

## 1. Identity Types (Newtype Pattern)

All entity IDs are UUID v7 newtypes with `Display`, `FromStr`, `Serialize`, and `Deserialize`. UUID v7 is time-ordered, enabling chronological sorting without a separate timestamp column.

```rust
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

/// A time-ordered UUID v7 identifier for a worktree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorktreeId(Uuid);

impl WorktreeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl fmt::Display for WorktreeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wt_{}", self.0)
    }
}

impl FromStr for WorktreeId {
    type Err = PorpoiseError;
    fn from_str(s: &str) -> Result<Self> {
        let inner = s.strip_prefix("wt_")
            .ok_or_else(|| PorpoiseError::invalid_id("WorktreeId", s))?;
        let uuid = Uuid::from_str(inner)
            .map_err(|e| PorpoiseError::invalid_id("WorktreeId", s))?;
        Ok(Self(uuid))
    }
}

// Same pattern for: TerminalId("tm_"), AgentId("ag_"), SessionId("ss_"),
//                    PageId("pg_"), CorrelationId("cr_")
```

**ID Prefix Convention:**

| Type | Prefix | Example |
|------|--------|---------|
| `WorktreeId` | `wt_` | `wt_0192ecf0-0a7c-7e80-b55a-1f2e3c4d5e6f` |
| `TerminalId` | `tm_` | `tm_0192ecf0-0a7c-7e81-b55a-2f3e4c5d6e7f` |
| `AgentId` | `ag_` | `ag_0192ecf0-0a7c-7e82-b55a-3f4e5c6d7e8f` |
| `SessionId` | `ss_` | `ss_0192ecf0-0a7c-7e83-b55a-4f5e6c7d8e9f` |
| `PageId` | `pg_` | `pg_0192ecf0-0a7c-7e84-b55a-5f6e7c8d9e0f` |
| `CorrelationId` | `cr_` | `cr_0192ecf0-0a7c-7e85-b55a-6f7e8c9d0e1f` |

---

## 2. Error System

```rust
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error enum for all Porpoise operations.
/// Each variant wraps a specific error domain.
#[derive(Error, Debug)]
pub enum PorpoiseError {
    // ── Configuration ──
    #[error("configuration error: {0}")]
    Config(String),
    
    #[error("config file not found at {0}")]
    ConfigNotFound(PathBuf),
    
    #[error("config parse error: {0}")]
    ConfigParse(String),
    
    // ── Database ──
    #[error("database error: {0}")]
    Db(String),
    
    #[error("migration error: {0}")]
    DbMigration(String),
    
    // ── Runtime / Process ──
    #[error("runtime error: {0}")]
    Runtime(String),
    
    #[error("process {pid} exited with code {code}")]
    ProcessExit { pid: u32, code: i32 },
    
    #[error("PTY allocation failed: {0}")]
    PtyError(String),
    
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    // ── Git ──
    #[error("git error: {0}")]
    Git(String),
    
    #[error("no such branch: {0}")]
    GitNoSuchBranch(String),
    
    #[error("merge conflict in {0}")]
    GitMergeConflict(PathBuf),
    
    // ── SSH ──
    #[error("SSH connection failed to {host}: {reason}")]
    SshConnect { host: String, reason: String },
    
    #[error("SSH authentication failed: {0}")]
    SshAuth(String),
    
    // ── Agent ──
    #[error("agent error: {0}")]
    Agent(String),
    
    #[error("agent {id} not found")]
    AgentNotFound { id: String },
    
    #[error("no agent of kind {kind} detected on PATH")]
    AgentNotDetected { kind: String },
    
    // ── Network ──
    #[error("network error: {0}")]
    Network(String),
    
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    
    #[error("request timeout after {0}ms")]
    Timeout(u64),
    
    // ── IPC / Relay ──
    #[error("IPC error: {0}")]
    Ipc(String),
    
    #[error("IPC connection refused")]
    IpcConnectionRefused,
    
    #[error("protocol version mismatch: server={server} client={client}")]
    IpcVersionMismatch { server: u8, client: u8 },
    
    // ── Terminal ──
    #[error("terminal error: {0}")]
    Terminal(String),
    
    // ── Browser ──
    #[error("browser error: {0}")]
    Browser(String),
    
    // ── Plugin ──
    #[error("plugin error: {0}")]
    Plugin(String),
    
    #[error("WASM runtime error: {0}")]
    Wasm(String),
    
    #[error("plugin {id} denied capability: {capability}")]
    PluginCapabilityDenied { id: String, capability: String },
    
    // ── Validation ──
    #[error("invalid ID format for {type_name}: {value}")]
    InvalidId { type_name: &'static str, value: String },
    
    #[error("validation error: {0}")]
    Validation(String),
    
    // ── Internal ──
    #[error("internal error: {0}")]
    Internal(String),
    
    #[error("not yet implemented: {0}")]
    Unimplemented(&'static str),
}

/// Convenience constructors
impl PorpoiseError {
    pub(crate) fn invalid_id(type_name: &'static str, value: impl fmt::Display) -> Self {
        Self::InvalidId { type_name, value: value.to_string() }
    }
}

/// Domain-specific Result alias
pub type Result<T> = std::result::Result<T, PorpoiseError>;
```

### Error Handling Pattern

```rust
// Domain crates define their own error-to-PorpoiseError conversion:
impl From<git2::Error> for PorpoiseError {
    fn from(e: git2::Error) -> Self {
        Self::Git(e.message().to_string())
    }
}

impl From<rusqlite::Error> for PorpoiseError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e.to_string())
    }
}

impl From<tokio::sync::broadcast::error::SendError<SystemEvent>> for PorpoiseError {
    fn from(e: tokio::sync::broadcast::error::SendError<SystemEvent>) -> Self {
        Self::Internal(format!("event bus send failed: {}", e))
    }
}
```

---

## 3. Event System

```rust
use tokio::sync::broadcast;

/// All system events flow through this enum.
/// Match on variants to react to state changes.
#[derive(Clone, Debug)]
pub enum SystemEvent {
    Worktree(WorktreeEvent),
    Terminal(TerminalEvent),
    Agent(AgentEvent),
    Git(GitEvent),
    Ssh(SshEvent),
    Browser(BrowserEvent),
    System(SystemEventKind),
}

#[derive(Clone, Debug)]
pub enum WorktreeEvent {
    Created { id: WorktreeId, path: PathBuf, name: String },
    Deleted { id: WorktreeId, path: PathBuf },
    StatusChanged { id: WorktreeId, status: WorktreeStatus },
    AgentAttached { id: WorktreeId, agent: AgentId },
    AgentDetached { id: WorktreeId },
}

#[derive(Clone, Debug)]
pub enum TerminalEvent {
    Created { id: TerminalId, session_id: SessionId, worktree_id: WorktreeId },
    Output { id: TerminalId, data: Vec<u8>, timestamp: i64 },
    Resized { id: TerminalId, rows: u16, cols: u16 },
    Closed { id: TerminalId, exit_code: Option<i32> },
    Bell { id: TerminalId },
}

#[derive(Clone, Debug)]
pub enum AgentEvent {
    Spawned { id: AgentId, kind: String, pid: u32 },
    Output { id: AgentId, text: String, kind: OutputKind },
    StatusChanged { id: AgentId, status: AgentStatusKind },
    Error { id: AgentId, message: String },
    Exited { id: AgentId, exit_code: i32 },
    HookFired { id: AgentId, hook_type: String, payload: String },
}

#[derive(Clone, Debug)]
pub enum OutputKind {
    Thinking,
    Code,
    ToolCall,
    ToolResult,
    Text,
    Error,
}

#[derive(Clone, Debug)]
pub enum AgentStatusKind {
    Spawning,
    Running,
    Thinking,
    WaitingForInput,
    Error,
    Completed,
    Killed,
}

#[derive(Clone, Debug)]
pub enum GitEvent {
    CloneStarted { repo: String },
    CloneCompleted { path: PathBuf },
    StatusChanged { repo: PathBuf, changes: usize },
    BranchCreated { repo: PathBuf, branch: String },
    WorktreeCreated { repo: PathBuf, path: PathBuf },
    WorktreeDeleted { path: PathBuf },
    FetchCompleted { repo: PathBuf },
}

#[derive(Clone, Debug)]
pub enum SshEvent {
    Connected { host: String },
    Disconnected { host: String },
    Reconnected { host: String },
    PortForwardStarted { host: String, local_port: u16 },
    PortForwardStopped { host: String, local_port: u16 },
}

#[derive(Clone, Debug)]
pub enum BrowserEvent {
    PageLoaded { page_id: PageId, url: String },
    SnapshotTaken { page_id: PageId, path: PathBuf },
    ElementClicked { page_id: PageId, selector: String },
}

#[derive(Clone, Debug)]
pub enum SystemEventKind {
    Startup,
    Shutdown,
    ConfigReloaded,
    LowMemory { available_mb: u64 },
    Error { message: String },
    Notification { title: String, body: String, severity: NotificationSeverity },
}

#[derive(Clone, Debug)]
pub enum NotificationSeverity { Info, Warning, Error }

/// Event bus — the central pub/sub mechanism.
/// Components subscribe to receive events.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    /// Create a new event bus with the given channel capacity.
    /// 1024 is a reasonable default — events are small and short-lived.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }
    
    /// Subscribe to all system events.
    /// Returns a Receiver that can be awaited with `recv()`.
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }
    
    /// Publish an event to all subscribers.
    /// If all receivers have been dropped, the event is silently discarded.
    pub fn publish(&self, event: impl Into<SystemEvent>) {
        let _ = self.tx.send(event.into());
    }
}

/// Event handling trait — implement to react to events.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event. Return Ok(()) or an error.
    async fn handle(&self, event: &SystemEvent) -> Result<()>;
    
    /// Return the event types this handler is interested in.
    /// Used to filter events before dispatch (future optimization).
    fn interested_in(&self) -> Vec<String> {
        vec![]  // default: all events
    }
}
```

---

## 4. Configuration

```rust
use serde::Deserialize;

/// Top-level configuration — loaded from TOML, environment variables, and CLI args.
/// Merged in order: defaults → config file → env vars → CLI flags.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub cli: CliConfig,
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub ssh: SshConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoreConfig {
    /// Data directory for SQLite DB, logs, and state
    pub data_dir: Option<PathBuf>,
    /// Maximum number of concurrent events in the bus
    pub event_bus_capacity: usize,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            data_dir: None,  // resolved to XDG/platform default
            event_bus_capacity: 1024,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliConfig {
    pub default_output_format: OutputFormat,
    pub pager: bool,
    pub color: ColorChoice,
}

#[derive(Debug, Clone, Deserialize)]
pub enum OutputFormat { Plain, Json, JsonPretty, Yaml }

#[derive(Debug, Clone, Deserialize)]
pub enum ColorChoice { Auto, Always, Never }

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_output_format: OutputFormat::Plain,
            pager: true,
            color: ColorChoice::Auto,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub url: Option<String>,       // overrides default "sqlite://data_dir/porpoise.db"
    pub pool_size: u32,
    pub wal_mode: bool,
    pub busy_timeout_ms: u64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: None,
            pool_size: 4,
            wal_mode: true,
            busy_timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub max_concurrent_processes: u32,
    pub process_timeout_secs: u64,
    pub health_check_interval_secs: u64,
    pub pty_buffer_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_processes: 32,
            process_timeout_secs: 86400,  // 24h
            health_check_interval_secs: 30,
            pty_buffer_size: 65536,       // 64KB
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub default_shell: String,
    pub max_concurrent_agents: u32,
    pub agent_timeout_secs: u64,
    pub allowed_agents: Vec<String>,  // empty = allow all
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitConfig {
    pub default_branch: String,
    pub fetch_on_open: bool,
    pub auto_prune_worktrees: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshConfig {
    pub connect_timeout_secs: u64,
    pub keepalive_interval_secs: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrowserConfig {
    pub default_width: u32,
    pub default_height: u32,
    pub user_agent: Option<String>,
}

/// Configuration discovery path:
/// 1. PORPOISE_CONFIG env var (explicit path)
/// 2. $XDG_CONFIG_HOME/porpoise/config.toml (Linux/macOS)
/// 3. ~/.config/porpoise/config.toml (fallback)
/// 4. %APPDATA%/porpoise/config.toml (Windows)
pub fn discover_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("PORPOISE_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    
    #[cfg(target_os = "linux")]
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().map(|h| h.join(".config"))
        .ok_or_else(|| PorpoiseError::Config("cannot determine home directory".into()))?);
    
    #[cfg(target_os = "macos")]
    let base = home_dir()?.join(".config");  // same as Linux on macOS
    
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .map_err(|_| PorpoiseError::Config("APPDATA not set".into()))?;
    
    Ok(base.join("porpoise").join("config.toml"))
}
```

---

## 5. State Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state — accessed via Arc<RwLock<>> across all service tasks.
/// The Mutex is async (tokio::sync::RwLock), not std::sync::Mutex,
/// because state reads may hold the lock across await points.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: RwLock<AppConfig>,
    pub event_bus: EventBus,
    pub worktrees: RwLock<HashMap<WorktreeId, WorktreeState>>,
    pub agents: RwLock<HashMap<AgentId, AgentState>>,
    pub sessions: RwLock<HashMap<SessionId, SessionState>>,
    pub db: sqlx::Pool<Sqlite>,
}

impl AppState {
    pub fn new(config: AppConfig, db: sqlx::Pool<Sqlite>, event_bus: EventBus) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config: RwLock::new(config),
                event_bus,
                worktrees: RwLock::new(HashMap::new()),
                agents: RwLock::new(HashMap::new()),
                sessions: RwLock::new(HashMap::new()),
                db,
            }),
        }
    }
    
    pub fn event_bus(&self) -> &EventBus {
        &self.inner.event_bus
    }
    
    pub async fn config(&self) -> tokio::sync::RwLockReadGuard<'_, AppConfig> {
        self.inner.config.read().await
    }
    
    pub async fn config_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, AppConfig> {
        self.inner.config.write().await
    }
    
    pub fn db(&self) -> &sqlx::Pool<Sqlite> {
        &self.inner.db
    }
}
```

---

## 6. Traits

```rust
/// A typed CLI command that can be dispatched to a handler.
#[async_trait]
pub trait Command: Send + Sync {
    type Output: Serialize;
    
    fn name(&self) -> &'static str;
    async fn execute(self, state: &AppState) -> Result<Self::Output>;
}

/// Persistent state serialization/deserialization.
#[async_trait]
pub trait StatePersister: Send + Sync {
    type State: Serialize + DeserializeOwned;
    
    async fn save(&self, state: &Self::State) -> Result<()>;
    async fn load(&self) -> Result<Option<Self::State>>;
}

/// A capability for the permission system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub scope: CapabilityScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityScope {
    Filesystem { paths: Vec<PathBuf>, write: bool },
    Network { urls: Vec<String> },
    Process { allow: Vec<String> },
    Ssh { hosts: Vec<String> },
    All,
    None,
}
```

---

## 7. Key Dependencies

| Crate | Version | Usage |
|-------|---------|-------|
| `uuid` | 1.x | UUID v7 generation |
| `serde` / `serde_derive` | 1.x | Serialization |
| `thiserror` | 2.x | Error derive macros |
| `tokio` | 1.x | Async runtime, broadcast channels |
| `tracing` | 0.1.x | Logging and diagnostics |
| `config` | 0.15.x | Configuration loading (TOML + env) |
| `chrono` | 0.4.x | Timestamps |
| `pathbuf` | (stdlib) | Path handling |

---

## Implementation Status (2026-07-01)

The core types in this document are fully **implemented in `crates/porpoise-core/src/`** (18/23 tasks done).

### Implemented
- ✅ `PorpoiseError` enum with 30+ typed variants across all domains
- ✅ 8 newtype IDs: `WorktreeId`, `TerminalId`, `AgentId`, `SessionId`, `PageId`, `CorrelationId`, `ProcessId`, `SkillId`
- ✅ UUID v7 generation via `uuid::Uuid::now_v7()`
- ✅ `AppConfig` hierarchy with all sub-configs + Default impls
- ✅ `SystemEvent` enum with 7 variant families + sub-events
- ✅ `EventBus` (tokio broadcast::channel) with tests
- ✅ `AppState` (Arc<RwLock<>>) with accessor methods
- ✅ `Capabilities` struct for permission model
- ✅ `EventHandler` and `Command` traits
- ✅ Config file discovery (XDG/AppData/env) + TOML loading

### Not Yet Implemented
- ⬜ `Platform` detection constants
- ⬜ `Version` struct
- ⬜ Serialization helpers
- ⬜ Full `#[doc]` on all public APIs
- ⬜ Property-based tests with `proptest`

---

*This document describes the type system that every Porpoise crate builds upon. Changes to core types ripple through the entire project — ensure backward compatibility or plan coordinated migrations across all crates.*
