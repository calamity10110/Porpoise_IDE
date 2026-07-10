# Porpoise Architecture Guide

> Rust-native AI orchestration IDE — port of [Orca](https://github.com/stablyai/orca)

---

## 1. System Overview

Porpoise is a modular, async-native system for orchestrating AI coding agents across parallel git worktrees. It replaces Orca's Electron + TypeScript architecture with a Rust Cargo workspace using Tokio for async I/O, Tauri for desktop UI, and SQLite for persistence.

```
┌──────────────────────────────────────────────────────────────────────┐
│                        USER INTERFACE LAYER                          │
│                                                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐ │
│  │   porpoise-cli   │  │   porpoise-app   │  │   Mobile Client      │ │
│  │   (clap CLI)     │  │   (Tauri/gtk)    │  │   (WebSocket)        │ │
│  └────────┬─────────┘  └────────┬─────────┘  └──────────┬───────────┘ │
│           │                     │                        │            │
└───────────┼─────────────────────┼────────────────────────┼────────────┘
            │                     │                        │
             │     IPC Layer (Unix domain socket / Named pipe)           │
             │     Protocol: JSON over binary frame                      │
            │                     │                        │            │
┌───────────▼─────────────────────▼────────────────────────▼────────────┐
│                      DAEMON/SERVICE LAYER                              │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     porpoise-server                               │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐  │  │
│  │  │  Runtime   │ │    Git     │ │    SSH     │ │ Agent Pool   │  │  │
│  │  │  Manager   │ │   Engine   │ │   Tunnel   │ │ Manager      │  │  │
│  │  │  (tokio)   │ │  (git2)    │ │ (thrussh)  │ │              │  │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └──────────────┘  │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐  │  │
│  │  │  Terminal  │ │  Network   │ │   Relay    │ │ Skills/WASM  │  │  │
│  │  │  Engine    │ │  (reqwest) │ │   (IPC)    │ │ (wasmtime)   │  │  │
│  │  └────────────┘ └────────────┘ └────────────┘ └──────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      SQLite Database                              │  │
│  │  worktrees | sessions | terminals | agents | config | history    │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Async everywhere** — All I/O is async via Tokio. No blocking calls on the main path.
2. **Layered modularity** — Core types at the bottom, services in the middle, interfaces at the top.
3. **Type-safe IPC** — JSON serialization over binary length-prefixed frames. Debuggable wire format, no schema conflicts.
4. **Immutable core** — Core domain types are `Clone + Send + Sync`. State mutation is explicit via `Arc<RwLock<>>`.
5. **Safety by construction** — Unsafe code is isolated in minimal, audited modules (PTY, signal handling).
6. **Platform abstraction** — Platform-specific code behind traits with cfg-based dispatch.

---

## 2. Crate Dependency Graph

```
porpoise-core (foundation types, traits, errors)
├── porpoise-db (SQLite persistence)
├── porpoise-relay (IPC protocol)
├── porpoise-network (HTTP/WS networking)
│
├── porpoise-cli ────────────── depends on: core, relay, network
├── porpoise-runtime ────────── depends on: core, relay
├── porpoise-git ────────────── depends on: core, network
├── porpoise-terminal ───────── depends on: core, runtime
├── porpoise-agent ──────────── depends on: core, runtime, git
├── porpoise-ssh ────────────── depends on: core, network
├── porpoise-browser ────────── depends on: core, network
├── porpoise-skills ─────────── depends on: core
│
├── porpoise-server ─────────── depends on: runtime, git, terminal, agent, ssh,
│                               browser, skills, db, relay, network
└── porpoise-app ────────────── depends on: cli, server (embeds as library)
```

Layer diagram:

```
     ┌─────────────┐  ┌────────────┐
     │ porpoise-app │  │porpoise-cli│     APPLICATION LAYER
     └──────┬───────┘  └─────┬──────┘
            │                │
     ┌──────▼────────────────▼──────┐
     │      porpoise-server          │  ORCHESTRATION LAYER
     │  ┌───┬───┬────┬────┬───┬───┐ │
     │  │ R │ G │ Ag │ T  │ B │ S │ │
     │  │ u │ i │ e │ e  │ r │ k │ │
     │  │ n │ t │ n │ r  │ w │ i │ │
     │  │ t │   │ t │ m  │ z │ l │ │
     │  └───┴───┴────┴────┴───┴───┘ │
     │  ┌──────────┬───────────────┐  │
     │  │   DB     │    Relay      │  │
     │  └──────────┴───────────────┘  │
     └───────────────┬────────────────┘
                     │
     ┌───────────────▼────────────────┐
     │        porpoise-core            │  FOUNDATION LAYER
     │  Types | Traits | Errors | Config│
     └─────────────────────────────────┘
```

---

## 3. Data Flow

### CLI Command Flow

```
User types: porpoise worktree create --name "fix-auth"

┌──────────┐     ┌────────────┐     ┌──────────────┐     ┌──────────────┐
│ porpoise │────>│ porpoise-  │────>│ porpoise-    │────>│ porpoise-    │
│ -cli     │ IPC │ relay      │ IPC │ server       │     │ runtime      │
│ clap     │     │ send req   │     │ route cmd    │     │ spawn        │
│ parse    │     │ wait resp  │     │ to handler   │     │ process      │
└──────────┘     └────────────┘     └──────┬───────┘     └──────────────┘
                                           │
                                    ┌──────▼───────┐     ┌──────────────┐
                                    │ porpoise-git │────>│ git worktree │
                                    │ create       │     │ checkout     │
                                    └──────────────┘     └──────────────┘
```

### Request/Response Flow

```rust
// 1. CLI sends a typed request via IPC
let request = RelayRequest {
    id: CorrelationId::new(),
    method: "worktree_create",
    params: serde_json::json!({ "name": "fix-auth", "agent": "codex" }),
    timestamp: now(),
};

// 2. Relay serializes with serde_json and sends over Unix socket
relay.send(&request).await?;

// 3. Server deserializes, routes to handler
let response = server.handle(request).await?;

// 4. CLI blocks (async) on response, then formats output
match response {
    Ok(data) => print_json(data),
    Err(e) => eprintln!("Error: {e}"),
}
```

### Agent Lifecycle

```
1. CLI:  porpoise worktree create --agent codex --prompt "Fix bugs"
2. Server: WorktreeManager.create() → porpoise-git.clone() → git checkout -b
3. Server: AgentManager.spawn("codex", worktree_path)
4. Runtime: PtyManager.alloc() → nix::pty::forkpty()
5. Runtime: tokio::process::Command("codex", args) in PTY
6. Server: AgentHookServer starts → listens for agent status OSC codes
7. Agent: sends OSC status codes → HookServer → EventBus → UI updates
8. User: sends input via TerminalManager → PTY master fd
9. Agent: exits → Runtime detects → AgentManager.cleanup() → EventBus
```

---

## 4. IPC Model

### Transport

| Platform | Transport | Path |
|----------|-----------|------|
| macOS    | Unix domain socket | `$TMPDIR/porpoise.sock` |
| Linux    | Unix domain socket | `/run/user/$UID/porpoise.sock` |
| Windows  | Named pipe | `\\.\pipe\porpoise` |

### Protocol

Binary protocol with length-prefixed messages:

```
┌──────────────────────────────────────────────┐
│ Byte Layout:                                  │
│ ┌────────┬────────┬──────────┬──────────────┐│
│ │ Magic  │ Version│ Payload  │ Payload      ││
│ │ 0x5050 │ 0x01   │ Length   │ (JSON)       ││
│ │ (2B)   │ (1B)   │ (4B LE)  │ (variable)   ││
│ └────────┴────────┴──────────┴──────────────┘│
└──────────────────────────────────────────────┘
```

### Message Types

```rust
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub id: CorrelationId,
    pub method: String,
    pub params: Value,       // serde_json::Value
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub id: CorrelationId,
    pub status: StatusCode,
    pub body: Value,
    pub error: Option<IpcError>,
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    WorktreeCreated { id: WorktreeId, path: PathBuf },
    WorktreeDeleted { id: WorktreeId },
    TerminalOutput { id: TerminalId, data: Vec<u8> },
    AgentStatus { id: AgentId, status: AgentStatusKind },
    AgentOutput { id: AgentId, text: String },
    SshConnected { host: String },
    SshDisconnected { host: String },
    BrowserSnapshot { page_id: PageId },
    Error { message: String },
}
```

### Reconnection

Clients reconnect with exponential backoff (100ms base, 30s max, jitter). The server reassigns sessions on reconnect if the client presents a valid session token.

---

## 5. Process Model

```
┌──────────────────────────────────────────────────────┐
│                  porpoise-server                       │
│                                                        │
│  ┌──────────────────────────────────────────────────┐ │
│  │ ProcessManager (tokio task tree)                  │ │
│  │                                                    │ │
│  │  ┌──────────────┐   ┌──────────────┐              │ │
│  │  │ Main loop    │   │ Health       │              │ │
│  │  │ (accept IPC) │   │ Checker      │              │ │
│  │  └──────┬───────┘   └──────┬───────┘              │ │
│  │         │                  │                       │ │
│  │  ┌──────▼──────────────────▼───────────────────┐  │ │
│  │  │  Child Task Pool (tokio tasks, one per agent) │  │ │
│  │  │                                               │  │ │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐         │  │ │
│  │  │  │ Agent 1 │ │ Agent 2 │ │ Agent 3 │  ...    │  │ │
│  │  │  │ (PID X) │ │ (PID Y) │ │ (PID Z) │         │  │ │
│  │  │  └─────────┘ └─────────┘ └─────────┘         │  │ │
│  │  └──────────────────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

### Unix Signal Handling

```rust
#[cfg(unix)]
pub async fn signal_handler() {
    let mut term = signal(SignalKind::terminate())?;
    let mut int = signal(SignalKind::interrupt())?;
    let mut hup = signal(SignalKind::hangup())?;
    
    tokio::select! {
        _ = term.recv() => graceful_shutdown("SIGTERM").await,
        _ = int.recv()  => graceful_shutdown("SIGINT").await,
        _ = hup.recv()  => reload_config().await,
    }
}
```

### Process Supervision

Each agent subprocess is wrapped in a tokio task with:
- stdin/stdout/stderr piped through PTY master
- Child exit status monitored via `SIGCHLD` / `WaitStatus`
- Output forwarded to EventBus for UI
- stdin written from Queue<Input> for input framing
- Health pings via agent OSC codes (every 30s)

---

## 6. Terminal Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Terminal System                            │
│                                                              │
│  ┌─────────────────┐      ┌──────────────────────────────┐  │
│  │  Client Side    │      │  Server Side (porpoise-      │  │
│  │  (UI/Terminal   │      │  runtime)                    │  │
│  │   Emulator)     │      │                              │  │
│  │                 │      │  ┌────────────────────────┐  │  │
│  │  ┌───────────┐  │  IPC │  │  PtyPool              │  │  │
│  │  │ Terminal  │  │<─────│> │  ┌────┐ ┌────┐ ┌────┐ │  │  │
│  │  │ Renderer  │  │      │  │  │PTY1│ │PTY2│ │PTY3│ │  │  │
│  │  └───────────┘  │      │  │  └────┘ └────┘ └────┘ │  │  │
│  │                 │      │  └────────────────────────┘  │  │
│  └─────────────────┘      │                              │  │
│                            │  ┌────────────────────────┐  │  │
│                            │  │ Scrollback Buffer       │  │  │
│                            │  │ (ring buffer → SQLite)  │  │  │
│                            │  └────────────────────────┘  │  │
│                            └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### PTY Allocation

```rust
use nix::pty::{self, PtyMaster};
use nix::unistd::{fork, ForkResult};
use std::os::fd::{FromRawFd, OwnedFd};

pub struct PtySession {
    pub master: PtyMaster,
    pub child_pid: Pid,
    pub stdin_tx: mpsc::Sender<Vec<u8>>,
    pub stdout_rx: mpsc::Receiver<Vec<u8>>,
}

impl PtySession {
    pub fn spawn(shell: &str, rows: u16, cols: u16) -> Result<Self> {
        let (master, slave) = pty::openpty(None, None)?;
        
        match unsafe { fork()? } {
            ForkResult::Child => {
                // Child: set up slave PTY as stdio
                nix::unistd::setsid()?;
                // ... dup2 slave to stdin/stdout/stderr
                exec::execvp(shell, &[shell])?;
            }
            ForkResult::Parent { child } => {
                // Parent: manage master PTY
                Ok(Self::new(master, child))
            }
        }
    }
}
```

### Output Multiplexing

Terminal output is framed via OSC sequences for agent detection, error extraction, and status updates. A `TerminalMultiplexer` trait abstracts the layout engine:

```rust
#[async_trait]
pub trait TerminalMultiplexer: Send + Sync {
    async fn layout(&self) -> Layout;
    async fn write_terminal(&self, id: TerminalId, data: &[u8]);
    async fn resize(&self, id: TerminalId, rows: u16, cols: u16);
    async fn split(&mut self, direction: SplitDirection) -> Result<TerminalId>;
    async fn close(&mut self, id: TerminalId) -> Result<()>;
}
```

---

## 7. Git Integration

### porpoise-git Architecture

```
┌────────────────────────────────────────────────────┐
│ GitEngine                                             │
│  ┌────────────────────────────────────────────────┐  │
│  │ git2 Repository wrapper                        │  │
│  │ - open / clone / init                          │  │
│  │ - status / diff / log                          │  │
│  │ - branch / checkout / merge                    │  │
│  └────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────┐  │
│  │ WorktreeManager                                │  │
│  │ - create / list / prune git worktrees          │  │
│  │ - auto-cleanup orphaned worktree dirs          │  │
│  │ - manage .git/worktrees/ metadata              │  │
│  └────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────┐  │
│  │ RemoteProvider trait                           │  │
│  │  ├── GitHubProvider (octocrab)                 │  │
│  │  ├── GitLabProvider (uberactiv)                │  │
│  │  ├── GiteaProvider (custom HTTP)               │  │
│  │  └── AzureDevOpsProvider (custom HTTP)         │  │
│  └────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────┐  │
│  │ Watcher (notify crate)                         │  │
│  │ - inotify (Linux) / FSEvents (macOS)           │  │
│  │ - watches .git/ for status changes             │  │
│  │ - coalesces events into debounced refresh      │  │
│  └────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

### Worktree Model

```rust
pub struct Worktree {
    pub id: WorktreeId,
    pub repo_path: PathBuf,
    pub worktree_path: PathBuf,
    pub git_dir: PathBuf,
    pub branch: String,
    pub base_ref: String,
    pub agent: Option<AgentId>,
    pub status: WorktreeStatus,
    pub created_at: DateTime<Utc>,
}

pub enum WorktreeStatus {
    Idle,
    Setup,
    Running { agent: AgentId },
    Completed { exit_code: i32 },
    Failed { error: String },
}
```

---

## 8. SSH Layer

```
┌────────────────────────────────────────────────────┐
│ porpoise-ssh                                        │
│  ┌──────────────────────────────────────────────┐  │
│  │ SshManager                                    │  │
│  │  ├── connect(host, auth) -> SshSession       │  │
│  │  ├── session.list()                           │  │
│  │  └── close(id)                               │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │ SshSession                                    │  │
│  │  ├── exec(command) -> Output                  │  │
│  │  ├── shell() -> PtyChannel                    │  │
│  │  ├── port_forward(local, remote)              │  │
│  │  ├── file_read(path) -> Vec<u8>               │  │
│  │  ├── file_write(path, data)                   │  │
│  │  └── reconnect()                              │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │ AuthProvider trait                            │  │
│  │  ├── KeyAuth { private_key_path }            │  │
│  │  ├── PasswordAuth { password }                │  │
│  │  └── AgentAuth (ssh-agent socket)            │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

Uses `thrussh` for pure-Rust SSH (no libssh2 dependency) or `ssh2` crate for battle-tested libssh2 bindings on platforms where it's available.

---

## 9. Security Model

### Principle: Capability-Based Security

Porpoise uses a capability-based permission model where each component explicitly declares what it needs:

```rust
pub struct Capabilities {
    pub fs_read: Vec<PathBuf>,
    pub fs_write: Vec<PathBuf>,
    pub network: Vec<UrlPattern>,
    pub process: Vec<ProcessPattern>,
    pub ssh: Vec<HostPattern>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            fs_read: vec![],
            fs_write: vec![],
            network: vec![],
            process: vec![],
            ssh: vec![],
        }
    }
}
```

### Plugin Sandbox (WASM)

Skills/plugins compile to WASM and run in a `wasmtime` instance with:

- Linear memory only (no raw pointers)
- No filesystem access without explicit capability grants
- No network access without explicit capability grants
- CPU/memory limits enforced by wasmtime
- Host functions exposed through a whitelisted API bridge

### Secure IPC

- Unix socket permissions: `0o700` (owner only)
- Windows named pipe ACL: restricted to the creating user
- Session tokens: 256-bit random, stored encrypted at rest
- All IPC messages validated against a schema before dispatch

---

## 10. Plugin System

```
┌────────────────────────────────────────────────────┐
│ porpoise-skills                                    │
│  ┌──────────────────────────────────────────────┐  │
│  │ WasmRuntime (wasmtime::Engine)                │  │
│  │  ├── compile_plugin(path) -> CompiledPlugin   │  │
│  │  ├── instantiate(plugin, caps) -> Instance    │  │
│  │  └── call(instance, "hook", args) -> Result   │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │ SkillRegistry                                 │  │
│  │  ├── register(id, wasm_bytes)                │  │
│  │  ├── list() -> Vec<SkillManifest>             │  │
│  │  ├── enable(id) / disable(id)                │  │
│  │  └── uninstall(id)                           │  │
│  └──────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────┐  │
│  │ HookSystem                                    │  │
│  │  ├── on_agent_start(hook)                    │  │
│  │  ├── on_agent_output(hook)                   │  │
│  │  ├── on_terminal_create(hook)                │  │
│  │  └── on_event(event, hook)                   │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

### Plugin API (WIT-based)

Skills use [WIT (WebAssembly Interface Types)](https://github.com/WebAssembly/component-model) for their API boundary:

```wit
interface porpoise-skills {
    /// Called when a hook triggers
    handle-hook: func(hook-type: string, payload: string) -> result<string, string>;
    
    /// Get the plugin's manifest
    get-manifest: func() -> manifest;
    
    /// Called during startup for initialization
    init: func(config: string) -> result<_, string>;
}
```

---

## 11. Database Schema

```sql
-- Core
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

CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id),
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    exit_code   INTEGER
);

CREATE TABLE terminals (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    rows        INTEGER NOT NULL DEFAULT 24,
    cols        INTEGER NOT NULL DEFAULT 80,
    shell       TEXT NOT NULL DEFAULT 'bash',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE terminal_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    terminal_id TEXT NOT NULL REFERENCES terminals(id),
    timestamp   TEXT NOT NULL DEFAULT (datetime('now')),
    data        BLOB NOT NULL,
    row_start   INTEGER,
    row_end     INTEGER
);

CREATE TABLE agents (
    id          TEXT PRIMARY KEY,
    worktree_id TEXT NOT NULL REFERENCES worktrees(id),
    kind        TEXT NOT NULL,  -- 'claude', 'codex', 'gemini', etc
    pid         INTEGER,
    status      TEXT NOT NULL DEFAULT 'spawning',
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    stopped_at  TEXT
);

CREATE TABLE config (
    key         TEXT PRIMARY KEY,
    value       BLOB NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for performance
CREATE INDEX idx_sessions_worktree ON sessions(worktree_id);
CREATE INDEX idx_terminals_session ON terminals(session_id);
CREATE INDEX idx_terminal_history_time ON terminal_history(timestamp);
CREATE INDEX idx_agents_worktree ON agents(worktree_id);
```

---

## 12. Cross-Platform Strategy

### Conditional Compilation

```rust
// Platform-specific imports
#[cfg(unix)]
use std::os::unix::net::UnixListener;

#[cfg(windows)]
use tokio::net::named_pipe::NamedPipeListener;

// Platform-specific implementations
#[cfg(unix)]
impl Transport for UnixSocketTransport { ... }

#[cfg(windows)]
impl Transport for NamedPipeTransport { ... }
```

### Platform Support Matrix

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| CLI | ✅ Built | ✅ Built | ✅ Built |
| PTY | ✅ forkpty | ✅ forkpty | ⬜ ConPTY (stub) |
| IPC | ✅ Unix socket | ✅ Unix socket | ✅ Named pipe (client + server) |
| Git | ✅ (git2) | ✅ (git2) | ✅ (git2) |
| SSH | ◆ (ssh2 engine) | ◆ (ssh2 engine) | ◆ (ssh2 engine) |
| Browser | ⬜ WKWebView | ⬜ webkit2gtk | ⬜ WebView2 |
| File watch | ✅ (notify) | ✅ (notify) | ✅ (notify) |
| Agent | ✅ Claude/Codex/Generic | ✅ Claude/Codex/Generic | ✅ Claude/Codex/Generic |
| Desktop | ⬜ (Tauri) | ⬜ (Tauri) | ⬜ (Tauri) |
| Daemon | ✅ pidfile | ✅ pidfile | ⬜ service |
| Terminal | ✅ PtyMux, parser | ✅ PtyMux, parser | ⬜ ConPTY |
| Network | ✅ reqwest/WS client | ✅ reqwest/WS client | ✅ reqwest/WS client |
| Skills/WASM | ◆ wasmtime engine | ◆ wasmtime engine | ◆ wasmtime engine |
| Platform detection | ✅ Runtime cfg | ✅ Runtime cfg | ✅ Runtime cfg |

✅ = Implemented · ◆ = Partial · ⬜ = Design done, not implemented

---

## 13. Crate Reference

| Crate | Description | Key Dependencies |
|-------|-------------|-----------------|
| `porpoise-core` | Foundation types, traits, errors, config | `serde`, `config`, `thiserror` |
| `porpoise-cli` | CLI binary with clap | `clap`, `serde_json`, `porpoise-core` |
| `porpoise-runtime` | Process/PTY management | `tokio`, `nix`, `porpoise-core` |
| `porpoise-git` | Git operations | `git2`, `octocrab`, `porpoise-core` |
| `porpoise-relay` | IPC relay protocol | `tokio`, `bincode`, `porpoise-core` |
| `porpoise-terminal` | Terminal emulation | `tokio`, `nix`, `porpoise-runtime` |
| `porpoise-browser` | Embedded browser | `webkit2gtk`/`webview2`, `porpoise-core` |
| `porpoise-ssh` | SSH connections | `thrussh`/`ssh2`, `porpoise-core` |
| `porpoise-network` | HTTP/WebSocket | `reqwest`, `tokio-tungstenite` |
| `porpoise-db` | SQLite persistence | `sqlx`/`rusqlite`, `porpoise-core` |
| `porpoise-server` | Background daemon | All service crates above |
| `porpoise-app` | Desktop application | `tauri`, `porpoise-cli` |
| `porpoise-agent` | Agent integration protocols | `porpoise-core`, `porpoise-runtime` |
| `porpoise-skills` | WASM plugin runtime | `wasmtime`, `porpoise-core` |

---

## 14. Event Bus

```rust
use tokio::sync::broadcast;

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

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }
    
    pub fn publish(&self, event: SystemEvent) {
        let _ = self.tx.send(event); // ignore ReceiverDropped errors
    }
}
```

---

---

## 15. Implementation Status

### Crate Implementation Matrix (2026-07-10)

| Crate | Lines | Files | Level | Status |
|-------|-------|-------|-------|--------|
| `porpoise-core` | 1,277 | 17 | Foundation | ✅ 23/24 tasks — types, errors, config, event bus, state, platform, version |
| `porpoise-db` | 671 | 10 | Foundation | ✅ 8/10 tasks — all CRUD models, migrations, pool |
| `porpoise-cli` | 700 | 16 | Application | ✅ 11/12 tasks — 16 commands, JSON output, completions, integration tests |
| `porpoise-relay` | 537 | 9 | Service | ✅ 10/11 tasks — Unix + Windows IPC, auto-reconnect, version neg. |
| `porpoise-runtime` | 950 | 11 | Service | ✅ 11/11 tasks — Unix PTY, ProcessManager, HealthChecker, signal, log rotation, WorktreeProcessManager |
| `porpoise-server` | 703 | 14 | Orchestration | ✅ 11/11 tasks — daemon, pidfile, graceful shutdown, services, log rotation, notifications |
| `porpoise-git` | 745 | 7 | Service | ✅ 14/14 tasks — GitEngine, WorktreeManager, GitHub provider, SSH key auth, stash |
| `porpoise-agent` | 1,443 | 13 | Service | ✅ 13/13 tasks — Agent trait, detectors (Claude/Codex/Gemini), pool, hook, session resume, account switcher, token usage |
| `porpoise-terminal` | 953 | 10 | Service | ✅ 11/11 tasks — PtyMultiplexer, parser, scrollback (SQLite + file), color schemes, reflow, regex search |
| `porpoise-network` | 340 | 6 | Service | ✅ 5/5 tasks — HTTP/WS client, rate limiter, proxy, connectivity monitor |
| `porpoise-ssh` | 471 | 7 | Service | ✅ 8/9 tasks — SshManager, session, auth, config, keepalive, exec, port forwarding, remote worktree |
| `porpoise-browser` | 80 | 3 | Service | ✅ 7/7 tasks — BrowserEngine trait stub, navigation types |
| `porpoise-skills` | 726 | 7 | Service | ✅ 13/14 tasks — WasmRuntime, SkillRegistry, sandbox, pipeline, hooks, hot-reload |
| `porpoise-app` | 249 | 4 | Application | ✅ 9/10 tasks — Tauri 2.x, menu bar, tray, keyboard shortcuts, frontend UI |

**Total: 134 Rust source files, ~9,945 lines across 14 crates.**

### Key Architectural Decisions Made During Implementation

| Decision | Rationale |
|----------|-----------|
| serde_json instead of bincode for IPC | Debuggable wire format, simpler tooling, no schema versioning issues |
| Cross-platform IPC via cfg gating | Unix sockets on Unix, named pipes on Windows; `#[cfg]`-separated modules |
| Platform-specific PTY via `src/pty/mod.rs` | Unified `PtySession` struct, platform impls behind `#[cfg]` |
| Direct tokio I/O instead of Transport trait | Avoids `async_trait` dyn-compatibility issues on Windows |
| UUID v7 for all entity IDs | Time-ordered sorting, no coordinator needed, collision-free |
| `thiserror` over `anyhow` | Precise error types for library crates; consumers decide presentation |
| `RotatingLogFile` for server logs | Size-based rotation with configurable max files, no external deps |
| `CompilationPipeline` for WASM | WAT→WASM compilation with wasmtime, .cwasm cache, export validation |
| `HookRegistry` with 9 hook types | Async callback system with error collection, no panic propagation |
| `HotReloadManager` with mtime polling | Simple, dependency-free approach; no filesystem watch overhead |

---

*This architecture is living documentation. As Porpoise evolves, update this document to reflect changes. Last updated: 2026-07-10 after Phase 0–7 implementation completion.*
