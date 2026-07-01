# Module Design: porpoise-terminal

> Terminal emulation, PTY multiplexing, scrollback management.

---

## Purpose

`porpoise-terminal` provides terminal I/O capabilities: managing multiple PTY sessions within a single workspace, parsing terminal output for OSC sequences and color codes, maintaining scrollback buffers persisted to SQLite, and providing a layout engine for terminal splits.

**What problem it solves:** Without a dedicated terminal crate, Porpoise would have no structured way to handle multiple concurrent terminal sessions, no scrollback that survives restarts, and no way to parse agent OSC status codes from terminal output. This crate turns raw PTY byte streams into structured, queryable, persistent terminal data.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `tokio` | 1.x | Async I/O from PTY master fds |
| `tracing` | 0.1 | Logging |
| `serde` / `serde_json` | 1.x | Serialization for scrollback output |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Import — `TerminalId`, `SessionId`, `TerminalEvent`, `PorpoiseError::Terminal*` |
| `porpoise-runtime` | Import — `PtyManager` for PTY allocation and I/O |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| PTY master fd | `porpoise-runtime::PtyManager.alloc()` | Return value | `PtySession { master_fd, child_pid }` |
| Raw terminal output | PTY master fd | `tokio::io::AsyncRead` | `Vec<u8>` |
| Terminal input | CLI / App via IPC | `TerminalService.send()` | `&[u8]` |
| Resize requests | CLI / UI | `TerminalMultiplexer.resize()` | `(rows, cols)` |
| Split requests | CLI / UI | `TerminalMultiplexer.split()` | `SplitDirection` |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| Parsed output events | EventBus subscribers | `TerminalEvent::Output` | `{ id, data, timestamp }` |
| OSC status codes | Agent hook system | Parsed from output stream | Structured agent status |
| Scrollback data | SQLite (via DB layer) | Batch insert to terminal_history | Compressed row chunks |
| Layout state | Server / UI | `TerminalMultiplexer::layout()` | `Layout` tree |
| Search results | Server / CLI | `ScrollbackSearch::search()` | `Vec<Match>` with positions |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| PTY session registry | Process lifecycle |
| Output parser state machine | PTY master fds (owned by runtime) |
| Scrollback ring buffers | Database connections |
| Terminal layout tree | Authentication state |
| Color scheme cache | Agent protocol state |

---

## Program Flow

### Terminal I/O Pipeline

```
Agent/shell stdout → PTY master fd
                           │
                    ┌──────▼──────┐
                    │ OutputReader │  (tokio task per PTY)
                    │ async loop   │
                    └──────┬──────┘
                           │ raw bytes
                           ▼
                    ┌────────────────┐
                    │ OutputParser   │
                    │                │
                    │ ┌──────────┐   │
                    │ │ OSC      │──│──> Agent status events
                    │ │ Detector │   │
                    │ └──────────┘   │
                    │ ┌──────────┐   │
                    │ │ Color    │──│──> Color scheme updates
                    │ │ Parser   │   │
                    │ └──────────┘   │
                    │ ┌──────────┐   │
                    │ │ Bell     │──│──> Terminal bell event
                    │ │ Detector │   │
                    │ └──────────┘   │
                    └──────┬────────┘
                           │ structured output
                           ▼
              ┌────────────────────────┐
              │   ScrollbackBuffer     │
              │   (ring buffer)        │
              │                        │
              │  ┌─────┐ ┌─────┐      │
              │  │Row N│ │Row N│ ...   │
              │  │     │ │+1  │       │
              │  └─────┘ └─────┘      │
              │                        │
              │  flush every 100ms     │
              │  or 100KB              │
              └────────┬───────────────┘
                       │ batch insert
                       ▼
              ┌────────────────────────┐
              │   SQLite               │
              │   terminal_history     │
              └────────────────────────┘
```

### Terminal Split Layout

```
┌──────────────────────────────────────┐
│  Session "dev"                        │
│                                      │
│  ┌──────────────┬──────────────────┐  │
│  │  Terminal A  │  Terminal B      │  │
│  │  (60% width) │  (40% width)     │  │
│  │  pty://agent1│  pty://shell     │  │
│  │              │                  │  │
│  │  $ thinking  │  $ npm test      │  │
│  │  about fix   │  PASS: 42 tests  │  │
│  │              │                  │  │
│  └──────────────┴──────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │  Terminal C                      │  │
│  │  (full width, 30% height)        │  │
│  │  pty://agent2                    │  │
│  │  $ git status                    │  │
│  └──────────────────────────────────┘  │
└──────────────────────────────────────┘

Layout tree:
  Session(flex-col)
    ├── Split(flex-row, 70%)
    │   ├── Terminal(A, 60%)
    │   └── Terminal(B, 40%)
    └── Terminal(C, 30%)
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | `TerminalId`, `TerminalEvent`, `PorpoiseError` | core → terminal |
| porpoise-runtime | Direct call | `PtyManager` for PTY alloc/read/write/resize | terminal → runtime |
| porpoise-server | Direct call | Terminal service uses multiplexer | server → terminal |
| porpoise-db | Indirect | Scrollback persisted via server's DB pool | server → db |
| porpoise-relay | Via server | Terminal I/O events proxied to CLI/UI | server → relay |

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Ring buffer + SQLite flush | Low-latency writes (ring buffer) + durable persistence (batched SQLite writes at 100ms intervals) |
| OSC parser in terminal crate | OSC codes carry agent metadata; parsing at the terminal level keeps other crates clean |
| Batch insert for scrollback | Individual row inserts would kill SQLite performance; batch at 100ms/100KB window |
| Layout engine as tree | Recursive tree layout enables arbitrary split nesting; each node knows its children |
| Row-based scrollback storage | Enables efficient search by line number range; BLOB per row for exact replay |

---

*Terminal output is the highest-volume data path in Porpoise — a busy agent session can produce MBs per minute. The scrollback buffer must use bounded memory (configurable, default 100K rows per session) with oldest-row eviction. Monitor WAL file growth on long-running sessions.*
