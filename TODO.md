# Porpoise Development TODO

> Task-level breakdown organized by crate.
> Priority markers: 🔴 P0 (blocking) · 🟡 P1 (core) · 🟢 P2 (enhancement) · ⚪ P3 (stretch)
> ✓ = implemented · ◆ = partial · ○ = not started

---

## Priority Legend

| Marker | Meaning | Timeline |
|--------|---------|----------|
| 🔴 P0 | Blocking — must have for MVP | Phase 0–1 |
| 🟡 P1 | High priority — core feature | Phase 2–4 |
| 🟢 P2 | Nice to have — enhancement | Phase 5–7 |
| ⚪ P3 | Future — stretch goal | Phase 8+ |

---

## Phase 0: Foundation

### 🔴 porpoise-core (weeks 1-3) — 18/23 ✓

- [x] Define `PorpoiseError` enum with typed variants: `Config`, `Db`, `Runtime`, `Git`, `Ssh`, `Agent`, `Network`, `Ipc`, `Terminal`, `Browser`, `Plugin`, `Validation`, `Internal`
- [x] Create newtype IDs: `WorktreeId`, `TerminalId`, `AgentId`, `SessionId`, `PageId`, `CorrelationId`, `ProcessId`, `SkillId`
- [x] UUID v7 with `Display`, `FromStr`, `Serialize`, `Deserialize`, `Hash`, `Eq`
- [x] Implement `AppConfig` hierarchy: `CoreConfig`, `CliConfig`, `DbConfig`, `RuntimeConfig`, `AgentConfig`, `GitConfig`, `SshConfig`, `BrowserConfig`
- [x] Define `SystemEvent` enum with `Worktree`, `Terminal`, `Agent`, `Git`, `Ssh`, `Browser`, `System` variants
- [x] Implement `EventBus` wrapper around `tokio::sync::broadcast::Sender` with tests
- [x] Create `AppState` struct with `Arc<RwLock<>>` for interior mutability
- [x] Implement `EventHandler` trait with async `handle()`
- [x] Define `Command` trait for typed CLI command routing
- [x] Create workspace `Cargo.toml` with all 14 crate scaffolding
- [x] Configure `rustfmt.toml` (max_width=120, imports_granularity=Crate)
- [x] Configure `clippy.toml`
- [x] Set up `tracing-subscriber` with file + stderr output (in server binary)
- [x] Define `Result<T>` type alias
- [x] Create `Capabilities` struct for permission model (Phase 7 prep)
- [x] Config file discovery (`PORPOISE_CONFIG` env var, XDG/AppData paths)
- [x] Config loading from TOML + environment variables
- [x] Config defaults per platform (data dir, socket path)
- [ ] Implement `Platform` detection: `#[cfg]`-based OS/arch constants
- [ ] Implement `Version` struct for app version tracking
- [ ] Write core serialization helpers (bincode config, json pretty-print)
- [ ] Document all public APIs with `#[doc]` attributes
- [ ] ⚪ P3: Add property-based tests with `proptest` for core types

### 🔴 porpoise-db (weeks 2-3) — 8/9 ✓

- [x] SQLite schema: `worktrees`, `sessions`, `terminals`, `terminal_history`, `agents`, `config`, `event_log`, `schema_version`
- [x] All tables with proper foreign keys (`ON DELETE CASCADE`), indexes, WAL mode
- [x] `Migration` trait and `migrate()` function with version tracking
- [x] CRUD operations: `WorktreeRow`, `SessionRow`, `TerminalRow`, `HistoryRow`, `AgentRow`, `ConfigEntry`
- [x] `DbPool` with r2d2 connection pooling (4 connections)
- [x] WAL mode, foreign_keys=ON, busy_timeout=5s
- [x] Database open/create with atomic schema version check
- [x] All queries parameterized (no string concatenation)
- [ ] Write integration tests with temporary databases
- [ ] ⚪ P3: Database metrics counters (queries, latency, cache hits)

### 🔴 porpoise-cli (weeks 2-3) — 9/11 ✓

- [x] `clap::Parser` command tree with all subcommand groups:
  - `daemon { start, stop, status }`
  - `worktree { create, list, show, rm, prune }`
  - `terminal { create, list, send, read, resize, close }`
  - `agent { list, run, stop, logs }`
  - `git { status, diff, log, clone, branch }`
  - `browser { open, snapshot, click, fill }`
  - `ssh { connect, worktree, port-forward }`
  - `config { get, set, list, edit }`
  - `skill { search, install, uninstall, list }`
  - `status`, `version`
- [x] `--json` flag on all commands
- [x] Shell completion generation (bash, zsh, fish, powershell)
- [x] `OutputFormat` enum: `{ Plain, Json, JsonPretty }`
- [x] CLI config file discovery (XDG/AppData)
- [x] `--verbose` / `--debug` flags
- [x] Colored output with `owo-colors`
- [x] Async command dispatch with `RelayClient`
- [ ] Pager for long output (`worktree list`, `agent logs`)
- [ ] CLI integration tests with `assert_cmd`/`assert_fs`
- [ ] ⚪ P3: Interactive mode (`porpoise shell`)

### 🟡 CI/CD & Tooling (week 3) — 5/8 ✓

- [x] GitHub Actions: build on ubuntu/macos/windows
- [x] GitHub Actions: test
- [x] GitHub Actions: clippy
- [x] GitHub Actions: fmt
- [x] `.github/dependabot.yml` (needed)
- [ ] GitHub Actions: `cargo audit`
- [ ] Pre-commit hook: clippy + fmt + test
- [ ] `justfile` or `Makefile.toml`
- [ ] ⚪ P3: Benchmark CI job

---

## Phase 1: CLI & Runtime

### 🔴 porpoise-relay (weeks 4-5) — 7/11 ✓

- [x] Binary frame protocol: `[magic:2B][version:1B][flags:1B][length:4B LE][payload:JSON]`
- [x] `FrameFlags` bitfield: REQUEST, RESPONSE, EVENT, COMPRESSED, ACK, STREAM
- [x] `WireMessage` enum: Request, Response, Event
- [x] `Request`/`Response` types with `CorrelationId`, `StatusCode`, `ProtocolError`
- [x] `UnixSocketTransport` for macOS/Linux
- [x] `RelayClient` with `call()` request/response
- [x] `RelayServer` with connection accept loop + per-connection handler tasks
- [x] Request correlation with `CorrelationId`
- [ ] `NamedPipeTransport` for Windows (stub exists, needs full impl)
- [ ] Auto-reconnect with exponential backoff
- [ ] Protocol version negotiation on connect
- [ ] IPC roundtrip benchmark tests
- [ ] ⚪ P3: Optional TLS for remote IPC

### 🔴 porpoise-runtime (weeks 4-5) — 7/11 ✓

- [x] `ProcessManager`: `spawn()`, `kill()`, `list()`, `shutdown_all()`
- [x] `PtyManager`: `alloc()`, `read()`, `write()`, `resize()`, `close()`
- [x] Unix PTY: `nix::pty::openpty()` + `fork()` + async I/O via tokio
- [x] `ProcessHandle` with watch channel for status, mpsc channel for commands
- [x] `ProcessPool` with max process limit
- [x] `HealthChecker` with periodic PID liveness checks
- [x] `ResourceLimits` struct (fd limits via `setrlimit`)
- [x] Signal handling: `SIGTERM`/`SIGINT` graceful shutdown
- [ ] Windows PTY: `ConPTY` via `CreatePseudoConsole` (stub exists)
- [ ] `WorktreeProcessManager`: agent spawn in worktree directory
- [ ] Process lifecycle integration tests
- [ ] 🟡 P1: cgroups for Linux resource limits

### 🔴 porpoise-server (week 6) — 5/9 ✓

- [x] `Daemon` binary with `new()`, `start()`, `run()` lifecycle
- [x] CLI → server protocol routing via `Router` with method dispatch
- [x] `WorktreeService`: create, list handlers
- [x] `TerminalService`: create handler
- [x] `AgentService`: list handler
- [x] `ConfigService`: get handler
- [x] Server startup with `AppConfig` + config file loading
- [ ] Graceful shutdown (SIGTERM → drain → cleanup → exit)
- [ ] Single-instance enforcement (pidfile / mutex)
- [ ] Log file management (rotation, size limits)
- [ ] Server stress tests (100 concurrent connections)
- [ ] 🟡 P1: Health endpoint with uptime, process count

---

## Phase 2: Git Integration

### 🟡 porpoise-git — 0/13 ✓ (placeholder crate only)

- [ ] `GitEngine` wrapper around `git2::Repository`: clone, open, status, diff, log, branch, checkout, merge
- [ ] `WorktreeManager`: create, list, remove, prune, `git2::Repository::worktree()`
- [ ] `RemoteProvider` trait: list_prs, get_pr, create_pr, merge_pr, list_issues
- [ ] GitHub provider using `octocrab`
- [ ] GitLab provider
- [ ] File watcher using `notify` crate (inotify/FSEvents/ReadDirectoryChanges)
- [ ] SSH git support (key auth)
- [ ] `git stash`/`git stash pop` for context switching
- [ ] Integration tests with temp repos
- [ ] 🟡 P1: Submodule support
- [ ] 🟢 P2: Git LFS support

---

## Phase 3: Terminal Engine

### 🟡 porpoise-terminal — 0/11 ✓ (placeholder crate only)

- [ ] `PtyMultiplexer`: multiple PTYs per session
- [ ] `OutputParser`: OSC sequences, color codes, hyperlinks
- [ ] Scrollback ring buffer (10k-100k lines, SQLite persistence)
- [ ] `TerminalLayout` engine: horizontal/vertical split, resize, tab groups
- [ ] Terminal search (CTRL+F, regex, case-insensitive)
- [ ] Color scheme manager (Alacritty YAML, iTerm2 plist import)
- [ ] Terminal output → EventBus for UI
- [ ] Reflow support on resize
- [ ] 🟢 P2: TrueColor detection, palette querying
- [ ] 🟢 P2: Sixel/Kitty image protocol

---

## Phase 4: Agent Framework

### 🟡 porpoise-agent — 0/12 ✓ (placeholder crate only)

- [ ] `AgentDetector`: PATH scanning, version detection, config discovery
- [ ] `Agent` trait: spawn, send_input, read_output, interrupt, shutdown
- [ ] `ClaudeCodeAgent` integration
- [ ] `CodexAgent` integration
- [ ] `GeminiAgent`/`GrokAgent` integration
- [ ] `HookServer`: file-based hook endpoint for agent status
- [ ] Agent session resume (SQLite-backed)
- [ ] Agent account switcher (multi-account)
- [ ] `AgentPool` with max concurrent limit
- [ ] Mock tests with fake agent processes
- [ ] 🟡 P1: Agent output streaming to WebSocket
- [ ] 🟢 P2: Custom agent configuration DSL

---

## Phase 5: Desktop Application

### 🟡 porpoise-app — 0/10 ✓ (placeholder crate only)

- [ ] Tauri project init
- [ ] Menu bar with platform conventions
- [ ] Worktree sidebar (tree view, drag-and-drop)
- [ ] Terminal panel (IPC-connected, split layout)
- [ ] Monaco/markdown editor
- [ ] Settings window (general, agent, git, keyboard shortcuts)
- [ ] System tray integration
- [ ] Keyboard shortcuts per platform
- [ ] Cross-platform window chrome
- [ ] 🟢 P2: Dark/light theme, window state persistence

---

## Phase 6: Advanced Features

### 🟡 porpoise-browser — 0/7 ✓ (placeholder crate only)

- [ ] `BrowserEngine` trait: WKWebView, webkit2gtk, WebView2
- [ ] Navigation: goto, back, forward, reload
- [ ] Snapshot (full page screenshot)
- [ ] Element interaction: click, fill, select
- [ ] Design Mode: click → inspect → screenshot → agent
- [ ] Tab management, cookie sharing
- [ ] ⚪ P3: JS console

### 🟡 porpoise-ssh — 0/9 ✓ (placeholder crate only)

- [ ] `SshManager`: connect, disconnect, list
- [ ] `SshSession`: exec, shell, port_forward, file_read, file_write
- [ ] `AuthMethod`: KeyAuth, PasswordAuth, AgentAuth
- [ ] Auto-reconnect with TCP keepalive
- [ ] SSH config parser (`~/.ssh/config`)
- [ ] Remote worktree on SSH host
- [ ] 🟢 P2: SFTP file browser

### 🟢 porpoise-network — 0/5 ✓ (placeholder crate only)

- [ ] HTTP client (reqwest), WebSocket (tokio-tungstenite)
- [ ] Rate limiter, proxy support, retry with backoff
- [ ] Network connectivity monitor

### 🟡 Server: Notifications — 0/3 ✓

- [ ] Desktop notifications (Tauri API), agent completion, error/warning
- [ ] Notification preferences, history in SQLite

---

## Phase 7: Plugin System

### 🟢 porpoise-skills — 0/12 ✓ (placeholder crate only)

- [ ] `wasmtime` engine with WASI
- [ ] WIT-based plugin API: handle-hook, get-manifest, init
- [ ] WASM compilation pipeline
- [ ] `SkillRegistry`: register, list, enable, disable, uninstall
- [ ] Capability sandboxing (no fs/network by default)
- [ ] Hook system: on_agent_start, on_agent_output, on_terminal_create
- [ ] Plugin hot-reload, cache
- [ ] CLI: `porpoise skill search/install/uninstall/list`
- [ ] Example plugins: highlighter, lint checker, sentiment analyzer
- [ ] SDK documentation
- [ ] 🟢 P2: Plugin marketplace
- [ ] ⚪ P3: Performance profiling

---

## Phase 8: Polish & Hardening — 0% (not started)

| Area | Status |
|------|--------|
| Performance | ○ All benchmarks |
| Security | ○ cargo audit, auth audit, fuzzing |
| Testing | ○ Cross-platform, stress tests |
| Documentation | ○ API docs, user guide, migration guide |
| Release | ○ v1.0, brew, winget, cargo-binstall |

---

## Summary

| Phase | Total | ✓ Done | ◆ Partial | ○ Not Started |
|-------|-------|--------|-----------|---------------|
| 0 | ~43 tasks | 40 | 3 | 0 |
| 1 | ~31 tasks | 19 | 12 | 0 |
| 2 | ~13 tasks | 0 | 0 | 13 |
| 3 | ~11 tasks | 0 | 0 | 11 |
| 4 | ~12 tasks | 0 | 0 | 12 |
| 5 | ~10 tasks | 0 | 0 | 10 |
| 6 | ~24 tasks | 0 | 0 | 24 |
| 7 | ~12 tasks | 0 | 0 | 12 |
| 8 | ~20 tasks | 0 | 0 | 20 |
| **Total** | **~176 tasks** | **59** | **15** | **102** |

---

## Phase Completion Checklist

Run this ritual after every phase is marked complete:

1. **Update documentation** — sync status in README.md, ROADMAP.md, and TODO.md summary table
2. **Run graphify** — regenerate knowledge graph (`graphify-out/`) to reflect new code relationships
3. **Git commit** — atomic commits per crate/change, following dependency order

---

*Last updated: 2026-07-01 after Phase 0–1 implementation pass. Next update: after Phase 2 work.*
