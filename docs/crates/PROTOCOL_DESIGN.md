# porpoise-relay: IPC Protocol Design

> Inter-process communication protocol for Porpoise.
> All communication between CLI, daemon, and UI goes through this relay.

---

## 1. Protocol Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        IPC Protocol Stack                            │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                 Application Layer                             │   │
│  │  Request/Response patterns, typed commands, events            │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
│                             │                                      │
│  ┌──────────────────────────▼──────────────────────────────────┐   │
│  │                 Serialization Layer                           │   │
│  │  serde_json for wire format (was bincode — changed for       │
│  │  debuggability and tooling compatibility)                     │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
│                             │                                      │
│  ┌──────────────────────────▼──────────────────────────────────┐   │
│  │                   Frame Layer                                │   │
│  │  [Magic: 2B][Version: 1B][Flags: 1B][Length: 4B][Payload]   │   │
│  └──────────────────────────┬──────────────────────────────────┘   │
│                             │                                      │
│  ┌──────────────────────────▼──────────────────────────────────┐   │
│  │                 Transport Layer                               │   │
│  │  Unix Domain Socket (macOS/Linux) | Named Pipe (Windows)     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Frame Format

Binary, length-prefixed framing:

```text
Offset  Size  Field           Description
──────  ────  ─────           ───────────
0       2     Magic           0x5050 ("PP" — Porpoise Protocol)
2       1     Version         Protocol version (currently 0x01)
3       1     Flags           Bitfield: 0x01=Request, 0x02=Response, 0x04=Event
                                0x08=Compressed, 0x10=Ack-Requested
4       4     Payload Length  Little-endian u32, bytes after header
8       N     Payload         JSON-serialized message (serde_json)
```

```rust
use serde::{Serialize, Deserialize};

pub const PROTOCOL_MAGIC: [u8; 2] = [0x50, 0x50];
pub const PROTOCOL_VERSION: u8 = 0x01;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct FrameFlags: u8 {
        const REQUEST   = 0b0000_0001;
        const RESPONSE  = 0b0000_0010;
        const EVENT     = 0b0000_0100;
        const COMPRESSED = 0b0000_1000;
        const ACK       = 0b0001_0000;
        const STREAM    = 0b0010_0000;
    }
}

/// Raw wire frame — parsed by the transport layer.
#[derive(Debug, Clone)]
pub struct Frame {
    pub version: u8,
    pub flags: FrameFlags,
    pub payload: Vec<u8>,
}
```

---

## 3. Message Types

```rust
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Every message on the wire is one of these three variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    Request(Request),
    Response(Response),
    Event(SystemEvent),
}

/// Request — sent by client, handled by server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: CorrelationId,
    pub method: String,
    pub params: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

/// Response — sent by server back to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: CorrelationId,
    pub status: StatusCode,
    pub body: serde_json::Value,
    pub error: Option<ProtocolError>,
}

/// Error detail in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusCode {
    Ok,
    Created,
    Accepted,
    NoContent,
    BadRequest,
    NotFound,
    Conflict,
    InternalError,
    Timeout,
    Unimplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidParams,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    RateLimited,
    Internal,
    Timeout,
    VersionMismatch,
    ConnectionRefused,
}
```

---

## 4. RPC Method Catalog

All RPC methods follow this pattern:

```rust
// ── Daemon ──
"daemon.status"        → params: {}                       → response: { uptime, version, process_count }
"daemon.shutdown"      → params: { graceful: bool }       → response: {}

// ── Worktree ──
"worktree.create"      → params: { name, repo?, agent?    → response: { id, path, branch }
                                    prompt? }
"worktree.list"        → params: { repo? }                → response: { worktrees: [...] }
"worktree.show"        → params: { id | name }            → response: { id, path, status, ... }
"worktree.delete"      → params: { id | name, force? }    → response: {}
"worktree.prune"       → params: { dry_run: bool }        → response: { orphans: [...]

// ── Terminal ──
"terminal.create"      → params: { worktree_id, shell?,  → response: { id }
                                    rows?, cols? }
"terminal.list"        → params: { worktree_id? }         → response: { terminals: [...] }
"terminal.send"        → params: { id, text, enter? }     → response: {}
"terminal.read"        → params: { id, cursor?, limit? }  → response: { data, cursor, limited }
"terminal.resize"      → params: { id, rows, cols }       → response: {}
"terminal.close"       → params: { id }                   → response: {}
"terminal.wait"        → params: { id, for, timeout_ms }  → response: { matched }

// ── Agent ──
"agent.list"           → params: {}                       → response: { agents: [...] }
"agent.run"            → params: { kind, worktree_id,     → response: { id, pid }
                                    prompt }
"agent.stop"           → params: { id }                   → response: {}
"agent.logs"           → params: { id, lines? }           → response: { logs: [...] }

// ── Git ──
"git.status"           → params: { repo_path }            → response: { branch, changes: [...] }
"git.diff"             → params: { repo_path, staged? }   → response: { files: [...] }
"git.log"              → params: { repo_path, count? }    → response: { commits: [...] }
"git.clone"            → params: { url, path, depth? }    → response: { path }
"git.branch"           → params: { repo_path, list? }     → response: { branches: [...] }

// ── SSH ──
"ssh.connect"          → params: { host, user?, auth }    → response: { session_id }
"ssh.disconnect"       → params: { session_id }           → response: {}
"ssh.exec"             → params: { session_id, command }  → response: { stdout, stderr, exit_code }
"ssh.port-forward"     → params: { session_id, local,    → response: { local_port }
                                    remote }
"ssh.worktree"         → params: { session_id, repo,     → response: { worktree_id }
                                    branch? }

// ── Browser ──
"browser.open"         → params: { url, width?, height? } → response: { page_id }
"browser.snapshot"     → params: { page_id }              → response: { path }
"browser.click"        → params: { page_id, selector }    → response: { snapshot? }
"browser.fill"         → params: { page_id, selector,     → response: {}
                                    value }

// ── Config ──
"config.get"           → params: { key }                  → response: { value }
"config.set"           → params: { key, value }           → response: {}
"config.list"          → params: {}                       → response: { entries: [...] }
```

---

## 5. Transport Layer

### Unix Domain Socket (macOS/Linux)

```rust
use tokio::net::UnixListener;
use tokio::net::UnixStream;

pub struct UnixSocketTransport {
    listener: Option<UnixListener>,
    socket_path: PathBuf,
}

impl UnixSocketTransport {
    /// Bind and listen on a Unix domain socket.
    pub async fn bind(path: &Path) -> Result<Self> {
        // Remove stale socket file
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
        
        let listener = UnixListener::bind(path)
            .map_err(|e| PorpoiseError::Ipc(format!("bind failed: {e}")))?;
        
        // Set permissions to owner-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).await?;
        }
        
        Ok(Self {
            listener: Some(listener),
            socket_path: path.to_path_buf(),
        })
    }
    
    /// Accept incoming connections.
    pub async fn accept(&self) -> Result<IpcConnection> {
        let (stream, _addr) = self.listener.as_ref()
            .ok_or_else(|| PorpoiseError::Ipc("not bound".into()))?
            .accept().await
            .map_err(|e| PorpoiseError::Ipc(format!("accept failed: {e}")))?;
        
        Ok(IpcConnection::new(Box::new(stream)))
    }
}

pub struct IpcConnection {
    stream: Box<dyn ReadWrite + Send + Unpin>,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

#[async_trait]
impl ReadWrite for UnixStream {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.readable().await?;
        self.try_read(buf)
    }
    
    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writable().await?;
        self.try_write(buf)?;
        Ok(())
    }
}

impl IpcConnection {
    /// Send a request and wait for a response.
    pub async fn request(&mut self, req: Request, timeout: Duration) -> Result<Response> {
        let frame = Frame {
            version: PROTOCOL_VERSION,
            flags: FrameFlags::REQUEST | FrameFlags::ACK,
            payload: bincode::serialize(&WireMessage::Request(req.clone()))?,
        };
        self.send_frame(frame).await?;
        
        tokio::time::timeout(timeout, self.receive_response(req.id)).await
            .map_err(|_| PorpoiseError::Timeout(timeout.as_millis() as u64))?
    }
    
    async fn send_frame(&mut self, frame: Frame) -> Result<()> {
        let mut header = Vec::with_capacity(8 + frame.payload.len());
        header.extend_from_slice(&PROTOCOL_MAGIC);
        header.push(frame.version);
        header.push(frame.flags.bits());
        header.extend_from_slice(&(frame.payload.len() as u32).to_le_bytes());
        header.extend_from_slice(&frame.payload);
        
        self.stream.write_all(&header).await
            .map_err(|e| PorpoiseError::Ipc(format!("write failed: {e}")))?;
        Ok(())
    }
    
    async fn read_frame(&mut self) -> Result<Frame> {
        // Read header (8 bytes)
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header).await
            .map_err(|e| PorpoiseError::Ipc(format!("read header failed: {e}")))?;
        
        // Validate magic
        if header[0..2] != PROTOCOL_MAGIC {
            return Err(PorpoiseError::Ipc("invalid magic bytes".into()));
        }
        
        let version = header[2];
        let flags = FrameFlags::from_bits(header[3])
            .ok_or_else(|| PorpoiseError::Ipc("invalid flags".into()))?;
        
        // Check version compatibility
        if version != PROTOCOL_VERSION {
            return Err(PorpoiseError::IpcVersionMismatch {
                server: PROTOCOL_VERSION,
                client: version,
            });
        }
        
        let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        
        // Read payload
        let mut payload = vec![0u8; length];
        if length > 0 {
            self.stream.read_exact(&mut payload).await
                .map_err(|e| PorpoiseError::Ipc(format!("read payload failed: {e}")))?;
        }
        
        Ok(Frame { version, flags, payload })
    }
}
```

### Named Pipe (Windows)

```rust
#[cfg(windows)]
pub struct NamedPipeTransport {
    pipe_name: String,
}

#[cfg(windows)]
impl NamedPipeTransport {
    pub async fn bind(name: &str) -> Result<Self> {
        // Named pipe path: \\.\pipe\porpoise
        let pipe_name = format!(r"\\.\pipe\{name}");
        
        // On Windows, create the named pipe server
        tokio::net::windows::named_pipe::ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)
            .map_err(|e| PorpoiseError::Ipc(format!("pipe bind failed: {e}")))?;
        
        Ok(Self { pipe_name })
    }
    
    pub async fn accept(&self) -> Result<IpcConnection> {
        // Accept new connection
        let pipe = tokio::net::windows::named_pipe::ServerOptions::new()
            .create(&self.pipe_name)
            .map_err(|e| PorpoiseError::Ipc(format!("pipe accept failed: {e}")))?;
        
        Ok(IpcConnection::new(Box::new(pipe)))
    }
}
```

---

## 6. Connection Lifecycle

```text
CLIENT                    SERVER
  │                         │
  │───── connect ──────────>│  Transport: connect to socket/pipe
  │<──── accept + version ──│  Server sends its protocol version
  │───── auth token ───────>│  Client authenticates
  │<──── session id ────────│  Server assigns session
  │                         │
  │───── Request ──────────>│  Normal operation
  │<──── Response ──────────│
  │                         │
  │<──── Event (push) ──────│  Server can push events anytime
  │                         │
  │          ... disconnect ...
  │                         │
  │───── connect ──────────>│  Reconnect
  │<──── accept ─────────── │
  │───── resume token ─────>│  Client presents session token
  │<──── session id ────────│  Session resumed (or new if token expired)
```

---

## 7. Client Implementation

```rust
/// High-level client for connecting to a Porpoise server.
pub struct RelayClient {
    connection: Mutex<Option<IpcConnection>>,
    socket_path: PathBuf,
    auth_token: String,
    session_id: Option<String>,
    event_tx: broadcast::Sender<SystemEvent>,
}

impl RelayClient {
    /// Connect to the server at the default socket path.
    pub async fn connect() -> Result<Self> {
        let path = default_socket_path()?;
        let token = load_or_create_auth_token()?;
        
        let mut client = Self {
            connection: Mutex::new(None),
            socket_path: path,
            auth_token: token,
            session_id: None,
            event_tx: EventBus::new(256).tx,
        };
        
        client.reconnect().await?;
        Ok(client)
    }
    
    async fn reconnect(&self) -> Result<()> {
        let stream = connect_to_server(&self.socket_path).await?;
        let mut conn = IpcConnection::new(stream);
        
        // Handshake
        self.handshake(&mut conn).await?;
        
        *self.connection.lock().await = Some(conn);
        
        // Spawn event listener
        let mut conn = self.connection.lock().await.take().unwrap();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            loop {
                match conn.read_frame().await {
                    Ok(frame) if frame.flags.contains(FrameFlags::EVENT) => {
                        if let Ok(WireMessage::Event(event)) = bincode::deserialize(&frame.payload) {
                            let _ = event_tx.send(event);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("IPC event listener error: {e}");
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    async fn handshake(&self, conn: &mut IpcConnection) -> Result<()> {
        // Send auth token
        let auth = Request {
            id: CorrelationId::new(),
            method: "auth.connect".into(),
            params: serde_json::json!({ "token": self.auth_token }),
            timeout_ms: Some(5000),
        };
        let resp = conn.request(auth, Duration::from_secs(5)).await?;
        if resp.status != StatusCode::Ok {
            return Err(PorpoiseError::Ipc("authentication failed".into()));
        }
        Ok(())
    }
    
    /// Send a request and wait for response with retries.
    pub async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req = Request {
            id: CorrelationId::new(),
            method: method.into(),
            params,
            timeout_ms: Some(30_000),
        };
        
        // Try with reconnect on failure
        for attempt in 0..3 {
            let mut guard = self.connection.lock().await;
            if let Some(ref mut conn) = *guard {
                match conn.request(req.clone(), Duration::from_secs(30)).await {
                    Ok(resp) => {
                        if resp.status == StatusCode::Ok {
                            return Ok(resp.body);
                        }
                        return Err(PorpoiseError::Ipc(resp.error
                            .map(|e| e.message)
                            .unwrap_or_else(|| "unknown error".into())));
                    }
                    Err(e) => {
                        tracing::warn!("IPC call failed (attempt {}): {e}", attempt + 1);
                        *guard = None; // drop broken connection
                    }
                }
            }
            
            // Reconnect
            drop(guard);
            self.reconnect().await?;
        }
        
        Err(PorpoiseError::IpcConnectionRefused)
    }
    
    /// Subscribe to server-pushed events.
    pub fn events(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_tx.subscribe()
    }
}
```

---

## 8. Server Implementation

```rust
/// IPC server — accepts client connections and dispatches requests.
pub struct RelayServer {
    transport: Box<dyn Transport>,
    router: Arc<Router>,
    state: AppState,
}

impl RelayServer {
    pub async fn bind(state: AppState) -> Result<Self> {
        let path = default_socket_path()?;
        
        #[cfg(unix)]
        let transport: Box<dyn Transport> = Box::new(
            UnixSocketTransport::bind(&path).await?
        );
        
        #[cfg(windows)]
        let transport: Box<dyn Transport> = Box::new(
            NamedPipeTransport::bind("porpoise").await?
        );
        
        let router = Arc::new(Router::new(&state));
        
        Ok(Self { transport, router, state })
    }
    
    /// Run the server — accepts connections and spawns handler tasks.
    pub async fn run(&self) -> Result<()> {
        loop {
            let connection = self.transport.accept().await?;
            let router = self.router.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_client(connection, router).await {
                    tracing::warn!("client handler error: {e}");
                }
            });
        }
    }
}

async fn handle_client(mut conn: IpcConnection, router: Arc<Router>) -> Result<()> {
    loop {
        let frame = conn.read_frame().await?;
        
        if !frame.flags.contains(FrameFlags::REQUEST) {
            continue; // ignore non-request frames on server
        }
        
        let wire: WireMessage = bincode::deserialize(&frame.payload)
            .map_err(|e| PorpoiseError::Ipc(format!("deserialize: {e}")))?;
        
        match wire {
            WireMessage::Request(req) => {
                let response = router.dispatch(req).await;
                let resp_frame = Frame {
                    version: PROTOCOL_VERSION,
                    flags: FrameFlags::RESPONSE,
                    payload: bincode::serialize(&WireMessage::Response(response))?,
                };
                conn.send_frame(resp_frame).await?;
            }
            _ => {}
        }
    }
}
```

---

## 9. Event Streaming

The server pushes events to all connected clients:

```rust
impl RelayServer {
    /// Push an event to all connected clients.
    pub async fn broadcast_event(&self, event: SystemEvent) -> Result<()> {
        let frame = Frame {
            version: PROTOCOL_VERSION,
            flags: FrameFlags::EVENT,
            payload: bincode::serialize(&WireMessage::Event(event))?,
        };
        
        let clients = self.client_registry.lock().await;
        for client in clients.values() {
            if let Err(e) = client.send_frame(frame.clone()).await {
                tracing::warn!("event broadcast failed: {e}");
            }
        }
        Ok(())
    }
}
```

---

## 10. Agent Hook Protocol

Agents communicate back to Porpoise through a file-based hook mechanism:

```text
Agent writes status updates as JSON to a hook endpoint file:
  /tmp/porpoise-hooks/<worktree_id>/
    ├── agent-hooks.json     ← Agent writes here
    └── agent-hooks.lock     ← File lock for atomic writes

Hook file format (newline-delimited JSON):
  {"type": "status", "status": "thinking", "ts": 1718000000}
  {"type": "output", "text": "Analyzing code...", "ts": 1718000001}
  {"type": "error", "text": "Build failed", "ts": 1718000002}
```

```rust
pub struct AgentHookServer {
    watch_dir: PathBuf,
    event_bus: EventBus,
}

impl AgentHookServer {
    /// Watch a worktree's hook directory for agent status changes.
    pub async fn watch_worktree(&self, worktree_id: &WorktreeId) -> Result<()> {
        let dir = self.watch_dir.join(worktree_id.to_string());
        tokio::fs::create_dir_all(&dir).await?;
        
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            // Parse hook file and emit events
        })?;
        
        watcher.watch(&dir, notify::RecursiveMode::NonRecursive)?;
        Ok(())
    }
}
```

---

## 11. Serialization Strategy

| Context | Format | Reason |
|---------|--------|--------|
| IPC wire protocol | serde_json | Debuggable, pipeable to jq, no schema versioning |
| CLI --json output | serde_json | Human-readable, pipeable to jq |
| Config files | TOML | Standard for Rust projects, easy to hand-edit |
| Event bus internal | Clone + Channel | Zero-copy for in-process communication |
| Log persistence | JSON lines (ndjson) | Parseable by log aggregators |
| Agent hook files | JSON lines | Append-only, atomic per-line writes |

---

## Implementation Status (2026-07-01)

| Feature | Status | Notes |
|---------|--------|-------|
| Binary frame protocol | ✅ Done | [magic][ver][flags][len][JSON payload] |
| WireMessage enum | ✅ Done | Request/Response/Event via serde_json |
| UnixSocketTransport | ✅ Done | Connect, send, receive |
| NamedPipeTransport | ⬜ Stub | Structure defined, tokio::net::pipe not yet implemented |
| RelayClient | ✅ Done | call() with serialization + transport |
| RelayServer | ✅ Done | Accept loop, per-client task, Router dispatch |
| Request correlation | ✅ Done | CorrelationId-based matching |
| Auto-reconnect | ⬜ Pending | Exponential backoff logic not yet written |
| Protocol version negotiation | ⬜ Pending | Version check in Frame::decode exists, handshake missing |
| IPC benchmarks | ⬜ Pending | criterion benchmarks not yet set up |

### Actual Implementation Changes

The implementation uses **serde_json** instead of bincode for the wire format. Rationale:
- Debuggable wire format — `echo` the payload to see what's being sent
- Pipeable with `jq` for testing and debugging
- No schema versioning issues with serde's field-optional-by-default model
- Performance is adequate for local IPC (<1ms target is achievable with JSON on Unix sockets)

---

*This protocol is designed for low-latency IPC between Porpoise components. Latency budget: <1ms for local IPC, <100ms for SSH relay.*
