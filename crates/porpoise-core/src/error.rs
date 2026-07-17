use std::path::PathBuf;

use thiserror::Error;

/// Convenience alias for `Result<T, PorpoiseError>` throughout the Porpoise system.
pub type Result<T> = std::result::Result<T, PorpoiseError>;

/// Top-level error type for all Porpoise operations.
///
/// Each variant wraps a specific error domain. Use `match` or the `#[error]` Display
/// impl for user-facing messages. Domain crates implement `From<T>` for their
/// respective error types.
#[derive(Error, Debug)]
pub enum PorpoiseError {
    // Configuration
    #[error("configuration error: {0}")]
    Config(String),
    #[error("config file not found at {0}")]
    ConfigNotFound(PathBuf),
    #[error("config parse error: {0}")]
    ConfigParse(String),

    // Database
    #[error("database error: {0}")]
    Db(String),
    #[error("migration error: {0}")]
    DbMigration(String),

    // Runtime / Process
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("process {pid} exited with code {code}")]
    ProcessExit { pid: u32, code: i32 },
    #[error("PTY allocation failed: {0}")]
    PtyError(String),
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),

    // Git
    #[error("git error: {0}")]
    Git(String),
    #[error("no such branch: {0}")]
    GitNoSuchBranch(String),
    #[error("merge conflict in {0}")]
    GitMergeConflict(PathBuf),

    // SSH
    #[error("SSH connection failed to {host}: {reason}")]
    SshConnect { host: String, reason: String },
    #[error("SSH authentication failed: {0}")]
    SshAuth(String),

    // Agent
    #[error("agent error: {0}")]
    Agent(String),
    #[error("agent {id} not found")]
    AgentNotFound { id: String },
    #[error("no agent of kind {kind} detected on PATH")]
    AgentNotDetected { kind: String },

    // Network
    #[error("network error: {0}")]
    Network(String),
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    #[error("request timeout after {0}ms")]
    Timeout(u64),

    // IPC / Relay
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("IPC connection refused")]
    IpcConnectionRefused,
    #[error("protocol version mismatch: server supports ({server_min}-{server_max}) client={client}")]
    IpcVersionMismatch { server_min: u8, server_max: u8, client: u8 },
    #[error("IPC authentication failed: {0}")]
    IpcAuthFailed(String),
    #[error("IPC session expired")]
    IpcSessionExpired,

    // Terminal
    #[error("terminal error: {0}")]
    Terminal(String),

    // Browser
    #[error("browser error: {0}")]
    Browser(String),

    // Plugin / Skills
    #[error("plugin error: {0}")]
    Plugin(String),
    #[error("WASM runtime error: {0}")]
    Wasm(String),
    #[error("plugin {id} denied capability: {capability}")]
    PluginCapabilityDenied { id: String, capability: String },
    #[error("WASM module {module} denied import: {import}")]
    WasmImportDenied { module: String, import: String },

    // Validation
    #[error("invalid ID format for {type_name}: {value}")]
    InvalidId { type_name: &'static str, value: String },
    #[error("validation error: {0}")]
    Validation(String),

    // Internal
    #[error("internal error: {0}")]
    Internal(String),
    #[error("not yet implemented: {0}")]
    Unimplemented(&'static str),
}

impl PorpoiseError {
    /// Constructs an `InvalidId` error for the given type name and raw value.
    pub fn invalid_id(type_name: &'static str, value: impl std::fmt::Display) -> Self {
        Self::InvalidId {
            type_name,
            value: value.to_string(),
        }
    }
}
