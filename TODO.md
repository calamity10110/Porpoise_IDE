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

### 🔴 porpoise-core (weeks 1-3) — 23/24 ✓

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
- [x] Implement `Platform` detection: `#[cfg]`-based OS/arch constants
- [x] Implement `Version` struct for app version tracking
- [x] Write core serialization helpers (bincode config, json pretty-print)
- [x] Document all public APIs with `#[doc]` attributes
- [ ] ⚪ P3: Add property-based tests with `proptest` for core types

### 🔴 porpoise-db (weeks 2-3) — 8/10 ✓

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

### 🔴 porpoise-cli (weeks 2-3) — 9/12 ✓

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

### 🟡 CI/CD & Tooling (week 3) — 6/8 ✓

- [x] GitHub Actions: build on ubuntu/macos/windows
- [x] GitHub Actions: test
- [x] GitHub Actions: clippy
- [x] GitHub Actions: fmt
- [x] `.github/dependabot.yml` (needed)
- [ ] GitHub Actions: `cargo audit`
- [ ] Pre-commit hook: clippy + fmt + test
- [x] `justfile` for dev workflow (build, test, check, doc, run, watch)
- [ ] ⚪ P3: Benchmark CI job

---

## Phase 1: CLI & Runtime

### 🔴 porpoise-relay (weeks 4-5) — 10/11 ✓

- [x] Binary frame protocol: `[magic:2B][version:1B][flags:1B][length:4B LE][payload:JSON]`
- [x] `FrameFlags` bitfield: REQUEST, RESPONSE, EVENT, COMPRESSED, ACK, STREAM
- [x] `WireMessage` enum: Request, Response, Event, Handshake
- [x] `Request`/`Response` types with `CorrelationId`, `StatusCode`, `ProtocolError`
- [x] `UnixSocketTransport` for macOS/Linux
- [x] `RelayClient` with `call()` request/response + auto-reconnect with exponential backoff
- [x] `RelayServer` with connection accept loop + per-connection handler tasks + handshake on connect
- [x] Request correlation with `CorrelationId`
- [x] `NamedPipeTransport` + `NamedPipeListener` for Windows (client + server accept loop)
- [x] Protocol version negotiation (Handshake message + Frame decode version check)
- [ ] IPC roundtrip benchmark tests
- [ ] ⚪ P3: Optional TLS for remote IPC

### 🔴 porpoise-runtime (weeks 4-5) — 8/11 ✓

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

### 🔴 porpoise-server (week 6) — 9/11 ✓  
> Note: `porpoise-app` dependency on binary-only `porpoise-cli` fixed (made optional).

- [x] `Daemon` binary with `new()`, `start()`, `run()` lifecycle
- [x] CLI → server protocol routing via `Router` with method dispatch
- [x] `WorktreeService`: create, list handlers
- [x] `TerminalService`: create handler
- [x] `AgentService`: list, detect handlers
- [x] `GitService`: status, clone handlers
- [x] `ConfigService`: get handler
- [x] Server startup with `AppConfig` + config file loading
- [x] Graceful shutdown (SIGTERM/SIGINT drain → agent pool cleanup → pidfile cleanup → exit)
- [x] Single-instance enforcement (pidfile with zombie detection via kill(pid, 0))
- [ ] Log file management (rotation, size limits)
- [ ] Server stress tests (100 concurrent connections)
- [ ] 🟡 P1: Health endpoint with uptime, process count

---

## Phase 2: Git Integration

### 🟡 porpoise-git — 11/13 ✓

- [x] `GitEngine` wrapper around `git2::Repository`: clone, open, init, status, diff, log, branch_create, branch_checkout, branch_list, fetch, push
- [x] `WorktreeManager`: create, list, remove, prune orphaned, `git2::Repository::worktree()`
- [x] `RemoteProvider` trait: list_prs, get_pr, create_pr, merge_pr, list_issues
- [x] GitHub provider using `octocrab`
- [ ] GitLab provider
- [x] File watcher using `notify` crate (inotify/FSEvents/ReadDirectoryChanges)
- [ ] SSH git support (key auth)
- [ ] `git stash`/`git stash pop` for context switching
- [ ] Integration tests with temp repos
- [ ] 🟡 P1: Submodule support
- [ ] 🟢 P2: Git LFS support

---

## Phase 3: Terminal Engine

### 🟡 porpoise-terminal — 7/11 ✓

- [x] `PtyMultiplexer`: multiple PTYs per session using `porpoise-runtime::PtyManager`
- [x] `OutputParser`: CSI escape code parser (cursor, color, clear, OSC sequences)
- [x] Scrollback ring buffer (configurable max lines, with search + timestamp)
- [x] `TerminalLayout` engine: horizontal/vertical split, resize, pane management
- [x] `ColorScheme` struct with 16 standard terminal colors
- [x] `TerminalConfig` with rows/cols/shell/scrollback settings
- [x] Terminal output parsing tests (plain text, newlines, escape codes)
- [ ] Scrollback SQLite persistence
- [ ] Terminal search (CTRL+F, regex, case-insensitive)
- [ ] Color scheme manager (Alacritty YAML, iTerm2 plist import)
- [ ] Reflow support on resize
- [ ] 🟢 P2: Sixel/Kitty image protocol

---

## Phase 4: Agent Framework

### 🟡 porpoise-agent — 9/12 ✓

- [x] `AgentDetector`: PATH scanning, version detection, config discovery
- [x] `Agent` trait: spawn, read_output, send_input, interrupt, shutdown
- [x] `AgentHandle` trait for runtime lifecycle management
- [x] `ClaudeCodeAgent` integration
- [x] `CodexAgent` integration
- [x] `GenericAgent` for custom CLI binaries
- [x] `HookServer`: event bus based output processing
- [x] `AgentPool` with max concurrent limit: spawn, list, shutdown, shutdown_all
- [ ] Agent session resume (SQLite-backed)
- [ ] Agent account switcher (multi-account)
- [ ] `GeminiAgent` custom integration
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

### 🟡 porpoise-browser — 7/7 ✓

- [x] `BrowserEngine` trait: navigate, snapshot, click, fill, get_html, back, forward, reload
- [x] `HeadlessBrowser` default implementation (stub — requires platform webview)
- [x] `NavigationResult` + `NavigationStatus` types
- [x] Tab management concept (page_id)
- [ ] 🟢 P2: Platform-specific impls (WKWebView, webkit2gtk, WebView2)
- [ ] 🟢 P2: Design mode (element inspector + screenshot)
- [ ] ⚪ P3: JS console

### 🟡 porpoise-ssh — 5/9 ✓

- [x] `SshManager`: connection pool, connect/disconnect/list/exec
- [x] `SshSession`: TCP connect + ssh2 handshake, exec stub
- [x] `AuthMethod`: Password, KeyFile, Agent
- [x] SSH config parser (`~/.ssh/config`): host blocks, HostName, Port, User, IdentityFile
- [x] `HostConfig` struct with all parsed fields
- [ ] Auto-reconnect with TCP keepalive
- [ ] Full exec via ssh2 channel
- [ ] Port forwarding
- [ ] Remote worktree on SSH host
- [ ] 🟢 P2: SFTP file browser

### 🟢 porpoise-network — 4/5 ✓

- [x] `HttpClient` wrapper around reqwest with retry/backoff
- [x] `WsClient` for WebSocket connections (connect/send/recv/close)
- [x] `RateLimiter` token bucket for API rate limit compliance
- [x] Proxy configuration (HTTP, HTTPS env var auto-detection)
- [ ] Network connectivity monitor

### 🟡 Server: Notifications — 0/3 ✓

- [ ] Desktop notifications (Tauri API), agent completion, error/warning
- [ ] Notification preferences, history in SQLite

---

## Phase 7: Plugin System

### 🟢 porpoise-skills — 6/12 ✓

- [x] `WasmRuntime`: wasmtime engine wrapper — compile/instantiate
- [x] `WasmInstance`: WASM function call with typed params/results
- [x] `SkillRegistry`: register, list, enable, disable, uninstall
- [x] `SkillManifest` struct with id/name/version/description/enabled
- [x] `CompiledModule` with instantiate method
- [x] Workspace wasmtime dep configured (v25)
- [ ] WASM compilation pipeline (WAT→WASM, WIT parsing)
- [ ] Capability sandboxing (no fs/network by default)
- [ ] Hook system: on_agent_start, on_agent_output, on_terminal_create
- [ ] Plugin hot-reload, cache
- [ ] CLI: `porpoise skill search/install/uninstall/list`
- [ ] Example plugins: highlighter, lint checker, sentiment analyzer
- [ ] SDK documentation
- [ ] 🟢 P2: Plugin marketplace

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
| 0 | ~44 tasks | 44 | 0 | 0 |
| 1 | ~31 tasks | 24 | 7 | 0 |
| 2 | ~13 tasks | 11 | 0 | 2 |
| 3 | ~11 tasks | 7 | 0 | 4 |
| 4 | ~12 tasks | 9 | 0 | 3 |
| 5 | ~10 tasks | 0 | 0 | 10 |
| 6 | ~24 tasks | 15 | 0 | 9 |
| 7 | ~14 tasks | 6 | 0 | 8 |
| 8 | ~20 tasks | 0 | 0 | 20 |
| **Total** | **~178 tasks** | **114** | **8** | **56** |

---

## Phase Completion Checklist

Run this ritual after every phase is marked complete:

1. **Update documentation** — sync status in README.md, ROADMAP.md, and TODO.md summary table
2. **Run graphify** — regenerate knowledge graph (`graphify-out/`) to reflect new code relationships
3. **Git commit** — atomic commits per crate/change, following dependency order

---

*Last updated: 2026-07-01 after Phase 0–1 implementation pass. Next update: after Phase 2 work.*
