# Module Design: porpoise-relay

> Inter-process communication (IPC) — the nervous system connecting all Porpoise processes.

---

## Purpose

`porpoise-relay` implements the binary IPC protocol that enables communication between the CLI, server daemon, and desktop application. It provides a typed request/response RPC mechanism plus server-pushed events, all over platform-native transport (Unix domain sockets on macOS/Linux, named pipes on Windows).

**What problem it solves:** Without a standardized IPC layer, every Porpoise component would need to implement its own socket handling, serialization, and protocol negotiation. The relay provides a single, consistent, versioned, and authenticated communication channel that all components share.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `tokio` | 1.x | Async I/O for socket/pipe read/write |
| `bincode` | 2.x | Binary serialization for wire format |
| `serde` / `serde_json` | 1.x | JSON for CLI output, bincode for wire |
| `tracing` | 0.1 | Logging |
| `bitflags` | 2.x | `FrameFlags` bitfield |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Import — `CorrelationId`, `SystemEvent`, `PorpoiseError::Ipc*`, `StatusCode` |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| Raw bytes from socket | OS transport | `tokio::net::read()` | `Vec<u8>` (framed) |
| `Request` structs | CLI / App | `RelayClient::call()` | Typed `Request { id, method, params }` |
| `Response` structs | Server handlers | Handler function return | Typed `Response { id, status, body }` |
| `SystemEvent` enum | Server services | `EventBus::publish()` | Typed enum |
| Connection listeners | OS transport | `accept()` loop | `TcpStream` / `PipeStream` |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| Framed bytes on socket | OS transport | `tokio::net::write_all()` | Binary: `[magic][version][flags][length][payload]` |
| Deserialized `Request` | Server router | `Router::dispatch()` | Typed `Request` |
| Deserialized `Response` | CLI / App | `RelayClient` future resolves | Typed `Response` |
| Pushed `SystemEvent` | CLI / App | `broadcast::Receiver` | Typed event |
| `Connection` handle | Server connection loop | `accept()` return | `IpcConnection` (boxed transport) |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| Socket/pipe file descriptors | Business logic |
| Wire protocol framing | Agent lifecycle |
| Connection accept loop | Worktree management |
| Frame serialization/deserialization | Event handling |
| Reconnection backoff state | Database |
| Protocol version negotiation | Config |

---

## Program Flow

### Connection Establishment

```
CLIENT                          SERVER
  │                               │
  │  ┌───────────────────────┐    │
  │  │ Connect to socket/pipe│───>│
  │  └───────────────────────┘    │
  │                               │  ┌──────────────────────┐
  │  <────────────────────────────│──│ Accept + create       │
  │                               │  │ IpcConnection handle  │
  │  ┌───────────────────────┐    │  └──────────────────────┘
  │  │ Send auth token       │───>│
  │  └───────────────────────┘    │  ┌──────────────────────┐
  │  <────────────────────────────│──│ Validate token,       │
  │                               │  │ assign session_id    │
  │  ┌───────────────────────┐    │  └──────────────────────┘
  │  │ Connected. Ready.     │    │
  │  └───────────────────────┘    │
```

### Request/Response Cycle

```
CLIENT                          SERVER
  │                               │
  │  ┌───────────────────────┐    │
  │  │ Build Request {id,    │    │
  │  │  method, params}      │    │
  │  └──────────┬────────────┘    │
  │             ▼                 │
  │  ┌───────────────────────┐    │
  │  │ Serialize with bincode│    │
  │  └──────────┬────────────┘    │
  │             ▼                 │
  │  ┌───────────────────────┐    │
  │  │ Wrap in Frame:        │───>│  ┌──────────────────────┐
  │  │ [magic][ver][flags]   │    │  │ Read frame header    │
  │  │ [length][payload]     │    │  │ (8 bytes)            │
  │  └───────────────────────┘    │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Validate magic+version│
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Read payload (N bytes)│
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Deserialize Request  │
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Route to handler     │
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Build Response       │
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │  <────────────────────────────│──│ Serialize + send back│
  │                               │  └──────────────────────┘
  │  ┌───────────────────────┐    │
  │  │ Deserialize Response  │    │
  │  └──────────┬────────────┘    │
  │             ▼                 │
  │  ┌───────────────────────┐    │
  │  │ Return to caller      │    │
  │  └───────────────────────┘    │
```

### Event Push (Server → Client, async)

```
SERVER                          CLIENT
  │                               │
  │  ┌───────────────────────┐    │
  │  │ Service publishes to  │    │
  │  │ EventBus              │    │
  │  └──────────┬────────────┘    │
  │             ▼                 │
  │  ┌───────────────────────┐    │
  │  │ RelayServer receives  │    │
  │  │ event from subscription│   │
  │  └──────────┬────────────┘    │
  │             ▼                 │
  │  ┌───────────────────────┐    │
  │  │ Serialize as Event,   │───>│  ┌──────────────────────┐
  │  │ send to all clients   │    │  │ Receive event frame  │
  │  └───────────────────────┘    │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Deserialize as       │
  │                               │  │ SystemEvent          │
  │                               │  └──────────┬───────────┘
  │                               │             ▼
  │                               │  ┌──────────────────────┐
  │                               │  │ Forward to           │
  │                               │  │ subscriber channel   │
  │                               │  └──────────────────────┘
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | `CorrelationId`, `StatusCode`, `PorpoiseError::Ipc*` | core → relay |
| porpoise-cli | Direct import | `RelayClient` for IPC to daemon | cli → relay |
| porpoise-server | Direct import | `RelayServer` to accept client connections | server → relay |
| porpoise-app | Direct import | `RelayClient` for IPC (same as CLI) | app → relay |
| porpoise-network | Optional | TLS-wrapped relay for remote connections | relay ↔ network |

**Transport auto-detection:** The relay uses `#[cfg(unix)]` / `#[cfg(windows)]` at compile time to select the right transport. Callers don't need to care about the platform.

---

## Wire Format Detail

```
Byte 0-1:    Magic bytes      0x50 0x50  ("PP")
Byte 2:      Version          0x01
Byte 3:      Flags            Bitfield:
                                bit 0: Request
                                bit 1: Response
                                bit 2: Event
                                bit 3: Compressed
                                bit 4: Ack-Requested
                                bit 5: Stream-Continuation
Byte 4-7:    Payload Length   u32 little-endian
Byte 8..N:   Payload          bincode-serialized WireMessage
```

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Binary protocol (bincode) | 5-10x faster than JSON serialization, deterministic size for framing |
| Length-prefixed framing | Simple, robust framing; no sentinel bytes needed (vs. delimiter-based) |
| Magic bytes | Prevents accidental connection to wrong port; version aids graceful migration |
| Async per-connection tasks | Each client gets an independent tokio task; one slow client doesn't block others |
| Exponential backoff reconnect | Prevents reconnection storms when server is restarting; jitter prevents thundering herd |
| Frame-level + payload-level validation | Frame header validated first (fast reject), then payload deserialized (expensive, but only if frame is valid) |

---

*The IPC protocol is the backbone of all Porpoise communication. Changes to the wire format require version negotiation and backward-compatible frame parsing. Always bump PROTOCOL_VERSION when adding new frame types or changing the message schema.*
