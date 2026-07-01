# Module Design: porpoise-cli

> Command-line interface binary — the primary user-facing entry point.

---

## Purpose

`porpoise-cli` is the command-line interface that users interact with directly. It parses commands via `clap`, connects to the Porpoise daemon via IPC (through `porpoise-relay`), and formats responses for terminal display or JSON output.

**What problem it solves:** Provides a human-friendly CLI for all Porpoise operations (worktree management, terminal sessions, agent control, git operations, SSH tunnels, browser automation, configuration) without requiring users to interact with the daemon directly.

---

## Dependencies

### External

| Crate | Version | Why |
|-------|---------|-----|
| `clap` | 4.x | Argument parsing with derive macros |
| `serde_json` | 1.x | JSON output formatting |
| `owo-colors` | 4.x | Terminal color output |
| `tokio` | 1.x | Async main function |
| `tracing` | 0.1 | CLI-level logging |
| `clap_complete` | 4.x | Shell completion generation |

### Internal

| Crate | Dependency Type |
|-------|----------------|
| `porpoise-core` | Direct import — types, `PorpoiseError`, `OutputFormat` |
| `porpoise-relay` | Direct import — `RelayClient` for IPC to daemon |

---

## Required Input

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| CLI arguments | User via terminal | `std::env::args()` | Raw strings → `clap` parsed |
| `PORPOISE_CONFIG` env var | User environment | `std::env::var()` | Path string |
| `--json` flag | CLI args | `clap` derive | Boolean |
| `--verbose` / `--debug` | CLI args | `clap` derive | Count/flag |
| Server response | porpoise-server daemon | IPC via `RelayClient` | `Response` (JSON body) |
| Config file | `~/.config/porpoise/config.toml` | `config` crate | TOML |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| Formatted terminal output | User | `println!()` / `eprintln!()` | Plain text or colored |
| JSON output | User / scripts (`jq`) | `serde_json::to_string_pretty()` | JSON to stdout |
| IPC requests | porpoise-server daemon | `RelayClient::call()` | `Request` (bincode via socket) |
| Exit code | Shell | `std::process::exit(n)` | `i32` (0=success) |
| Shell completions | Shell (bash/zsh/fish) | `clap_complete` generator | Shell script |
| Error messages | User | `eprintln!()` to stderr | Colored text |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| CLI argument parsing | Server state |
| Output formatting | Process lifecycle |
| Config file discovery | Database |
| Shell completion scripts | Agent management |
| Error display logic | IPC protocol state |

---

## Program Flow

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  User    │────>│ porpoise-cli │────>│  RelayClient  │────>│  Server IPC  │
│  types   │     │  (main.rs)   │     │  (connect)    │     │  (dispatch)  │
│ command  │     │              │     │              │     │              │
└──────────┘     └──────────────┘     └──────────────┘     └──────────────┘
                       │                      │                    │
                  ┌────▼─────┐          ┌──────▼──────┐     ┌──────▼──────┐
                  │  clap    │          │  Serialize  │     │  Deserialize│
                  │  parse   │          │  Request    │     │  Response   │
                  └──────────┘          └─────────────┘     └─────────────┘
                       │                                           │
                  ┌────▼─────┐                               ┌─────▼──────┐
                  │  Command  │                               │  Format    │
                  │  Routing  │                               │  Output    │
                  └──────────┘                               └────────────┘
```

Detailed flow for `porpoise worktree create --name fix-auth`:

```
1. User runs: porpoise worktree create --name fix-auth --agent codex
2. main() → app.rs → clap parses → WorktreeCreate command struct
3. Command handler creates RelayClient connection to daemon socket
4. Client sends typed Request: { method: "worktree.create", params: { name, agent } }
5. IPC transport: serialize with bincode → write to Unix socket / named pipe
6. Server receives, routes to WorktreeService, executes
7. Server sends Response: { status: Ok, body: { id, path, branch } }
8. Client receives Response, formats output
9. OutputFormatter: if --json → JSON to stdout, else plain text table
10. Exit with code 0
```

### Subcommand Lifecycle States

```
[Start] → Parse args → Resolve config → Connect to daemon
    │
    ├── Daemon not running → "porpoise daemon start" hint → exit 1
    ├── Connection refused → retry 3x with backoff → exit 1
    ├── Auth failed → "invalid session token" → exit 1
    │
    └── Connected → Dispatch command
        │
        ├── Success → Format output → exit 0
        ├── Error → Format error → exit 1
        └── Timeout → "request timed out" → exit 1
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-core | Import | Types, `OutputFormat`, `PorpoiseError` | core → cli |
| porpoise-relay | Import | `RelayClient::call()` | relay → cli |
| porpoise-server | IPC (socket) | `Request`/`Response` messages | cli ↔ server |
| porpoise-app | Same (Tauri cmd) | CLI commands invoked from GUI | app → cli |
| porpoise-db | Indirect | Through server responses | server → db |

**Key rule:** CLI never talks to the database directly. All state queries go through the server. This keeps the server as the single source of truth.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Server-required model | CLI is a thin client; daemon owns all state. Prevents stale data, race conditions |
| `--json` for all commands | Enables scripting, piping, and IDE integration |
| `clap` derive over builder | Less boilerplate, compile-time validation of argument definitions |
| Auto-generated completions | Reduces user friction; why make users write completions by hand? |
| Color on by default | Better UX for humans; `--color never` for CI/scripts |

---

*The CLI is the primary user interface. Every new server feature must be accessible from the CLI before any GUI work begins. This ensures the system is always usable from a terminal-first workflow.*
