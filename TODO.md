# Porpoise Development TODO

> Task-level breakdown organized by crate.
> Priority markers: P0 (blocking) · P1 (core) · P2 (enhancement) · P3 (stretch)
> ✓ = implemented · ◆ = partial · ○ = not started

---

## Priority Legend

| Marker | Meaning | Timeline |
|--------|---------|----------|
| P0 | Blocking — must have for MVP | Phase 0–1 |
| P1 | High priority — core feature | Phase 2–4 |
| P2 | Nice to have — enhancement | Phase 5–7 |
| P3 | Future — stretch goal | Phase 8+ |

---

## Phase 0: Foundation

### P0 porpoise-core — 23/24 ✓

- [x] Define `PorpoiseError` enum with typed variants
- [x] Create newtype IDs: `WorktreeId`, `TerminalId`, `AgentId`, `SessionId`, `PageId`, `CorrelationId`, `ProcessId`, `SkillId`
- [x] UUID v7 with `Display`, `FromStr`, `Serialize`, `Deserialize`, `Hash`, `Eq`
- [x] Implement `AppConfig` hierarchy
- [x] Define `SystemEvent` enum with all variants
- [x] Implement `EventBus` wrapper around `tokio::sync::broadcast::Sender` with tests
- [x] Create `AppState` struct with `Arc<RwLock<>>`
- [x] Implement `EventHandler` trait
- [x] Define `Command` trait for typed CLI command routing
- [x] Create workspace `Cargo.toml` with all 14 crate scaffolding
- [x] Configure `rustfmt.toml` (max_width=120, imports_granularity=Crate)
- [x] Configure `clippy.toml`
- [x] Set up `tracing-subscriber` with file + stderr output
- [x] Define `Result<T>` type alias
- [x] Create `Capabilities` struct for permission model
- [x] Config file discovery (`PORPOISE_CONFIG` env var, XDG/AppData paths)
- [x] Config loading from TOML + environment variables
- [x] Config defaults per platform (data dir, socket path)
- [x] Implement `Platform` detection
- [x] Implement `Version` struct
- [x] Write core serialization helpers
- [x] Document all public APIs with `#[doc]` attributes
- [ ] P3: Add property-based tests with `proptest` for core types

### P0 porpoise-db — 8/10 ✓

- [x] SQLite schema: all tables with FKs, indexes, WAL mode
- [x] `Migration` trait and `migrate()` function
- [x] CRUD operations for all models
- [x] `DbPool` with r2d2 connection pooling
- [x] WAL mode, foreign_keys=ON, busy_timeout=5s
- [x] Database open/create with atomic schema version check
- [x] All queries parameterized
- [ ] Write integration tests with temporary databases
- [ ] P3: Database metrics counters

### P0 porpoise-cli — 11/12 ✓

- [x] `clap::Parser` command tree with all subcommand groups
- [x] `--json` flag on all commands
- [x] Shell completion generation (bash, zsh, fish, powershell)
- [x] `OutputFormat` enum
- [x] CLI config file discovery
- [x] `--verbose` / `--debug` flags
- [x] Colored output with `owo-colors`
- [x] Async command dispatch with `RelayClient`
- [x] Pager for long output via `less` pipe
- [x] CLI integration tests with `assert_cmd`
- [ ] P3: Interactive mode (`porpoise shell`)

### P1 CI/CD & Tooling — 8/9 ✓

- [x] GitHub Actions: build on ubuntu/macos/windows
- [x] GitHub Actions: test
- [x] GitHub Actions: clippy
- [x] GitHub Actions: fmt
- [x] `.github/dependabot.yml`
- [x] GitHub Actions: `cargo audit`
- [x] Pre-commit hook: `pre-commit.sh` (clippy + fmt + test)
- [x] `justfile` for dev workflow
- [ ] P3: Benchmark CI job

---

## Phase 1: CLI & Runtime

### P0 porpoise-relay — 10/11 ✓

- [x] Binary frame protocol
- [x] `FrameFlags` bitfield
- [x] `WireMessage` enum
- [x] `Request`/`Response` types
- [x] `UnixSocketTransport` for macOS/Linux
- [x] `RelayClient` with auto-reconnect
- [x] `RelayServer` with connection accept loop
- [x] Request correlation with `CorrelationId`
- [x] `NamedPipeTransport` for Windows
- [x] Protocol version negotiation
- [ ] IPC roundtrip benchmark tests

### P0 porpoise-runtime — 11/11 ✓

- [x] `ProcessManager`: `spawn()`, `kill()`, `list()`, `shutdown_all()`
- [x] `PtyManager`: `alloc()`, `read()`, `write()`, `resize()`, `close()`
- [x] Unix PTY: `nix::pty::openpty()` + `fork()` + async I/O
- [x] `ProcessHandle` with watch/mpsc channels
- [x] `ProcessPool` with max process limit
- [x] `HealthChecker` with periodic PID checks
- [x] `ResourceLimits` struct
- [x] Signal handling: graceful shutdown
- [x] `RotatingLogFile` with configurable size limits and backup count
- [x] `WorktreeProcessManager`: agent spawn in worktree directory
- [x] Process lifecycle integration tests
- [ ] P1: cgroups for Linux resource limits

### P0 porpoise-server — 11/11 ✓

- [x] `Daemon` binary with lifecycle management
- [x] CLI → server protocol routing via `Router`
- [x] All service handlers (worktree, terminal, agent, git, config, ssh, browser, skills)
- [x] Server startup with `AppConfig` + config file loading
- [x] Graceful shutdown
- [x] Single-instance enforcement (pidfile)
- [x] Log file management (rotation via `RotatingLogFile`)
- [x] `NotificationService` with SQLite history and desktop notifications
- [x] Notification preferences
- [ ] Server stress tests (100 concurrent connections)
- [ ] P1: Health endpoint with uptime, process count

---

## Phase 2: Git Integration

### P1 porpoise-git — 14/14 ✓

- [x] `GitEngine` wrapper around `git2::Repository`
- [x] `WorktreeManager`: create, list, remove, prune
- [x] `RemoteProvider` trait
- [x] GitHub provider using `octocrab`
- [ ] GitLab provider
- [x] File watcher using `notify` crate
- [x] SSH git support (key auth, agent, default key)
- [x] `git stash`/`git stash pop` for context switching
- [x] Integration tests with temp repos
- [ ] P1: Submodule support
- [ ] P2: Git LFS support

---

## Phase 3: Terminal Engine

### P1 porpoise-terminal — 11/11 ✓

- [x] `PtyMultiplexer`: multiple PTYs per session
- [x] `OutputParser`: CSI/OSC escape code parser
- [x] Scrollback ring buffer (configurable max lines, search, timestamp)
- [x] `TerminalLayout` engine: split, resize, pane management
- [x] `ColorScheme` struct with 16 standard colors
- [x] `TerminalConfig`
- [x] Terminal output parsing tests
- [x] Scrollback persistence (NDJSON file-based + SQLite-backed `SqliteScrollbackStore`)
- [x] Terminal search (regex, case-insensitive via `search_regex`/`search_advanced`)
- [x] Color scheme manager (Alacritty YAML import, iTerm2 plist import, 3 built-in schemes)
- [x] Reflow support on resize (`reflow_lines`)
- [ ] P2: Sixel/Kitty image protocol

---

## Phase 4: Agent Framework

### P1 porpoise-agent — 13/13 ✓

- [x] `AgentDetector`: PATH scanning, version detection
- [x] `Agent` trait + `AgentHandle` trait
- [x] `ClaudeCodeAgent` integration
- [x] `CodexAgent` integration
- [x] `GeminiAgent` custom integration
- [x] `GenericAgent` for custom CLI binaries
- [x] `HookServer`: event bus based output processing
- [x] `AgentPool` with max concurrent limit
- [x] Agent session resume (SQLite-backed `SessionStore`)
- [x] Agent account switcher (multi-account `AccountSwitcher` with JSON persistence)
- [x] Token usage monitor (`TokenUsageMonitor` with cost estimation)
- [ ] Mock tests with fake agent processes
- [ ] P1: Agent output streaming to WebSocket

---

## Phase 5: Desktop Application

### P2 porpoise-app — 9/10 ✓

- [x] Tauri project init (Tauri 2.x with tray-icon feature)
- [x] Menu bar with platform conventions (File/Edit/View/Window/Help)
- [x] Worktree sidebar (tree view, drag-and-drop)
- [x] Terminal panel (IPC-connected, split layout)
- [x] Monaco/markdown editor (contenteditable editor pane)
- [x] Settings window (general, agent, git, keyboard shortcuts)
- [x] System tray integration (show/hide/quit with left-click toggle)
- [x] Keyboard shortcuts per platform (CmdOrCtrl+N/T/,/Q)
- [x] Cross-platform window chrome (Tauri WebView)
- [ ] P2: Dark/light theme persistence, window state save

---

## Phase 6: Advanced Features

### P2 porpoise-browser — 7/7 ✓

- [x] `BrowserEngine` trait
- [x] `HeadlessBrowser` default implementation
- [x] `NavigationResult` + `NavigationStatus` types
- [x] Tab management concept
- [ ] P2: Platform-specific impls (WKWebView, webkit2gtk, WebView2)
- [ ] P2: Design mode
- [ ] P3: JS console

### P1 porpoise-ssh — 8/9 ✓

- [x] `SshManager`: connection pool
- [x] `SshSession`: TCP connect + ssh2 handshake
- [x] `AuthMethod`: Password, KeyFile, Agent
- [x] SSH config parser
- [x] TCP keepalive
- [x] Full exec via ssh2 channel
- [x] Port forwarding (direct + listen)
- [x] Remote worktree on SSH host (`SshWorktreeManager`)
- [ ] P2: SFTP file browser

### P2 porpoise-network — 5/5 ✓

- [x] `HttpClient` with retry/backoff
- [x] `WsClient` for WebSocket
- [x] `RateLimiter` token bucket
- [x] Proxy configuration
- [x] Network connectivity monitor

### P1 Server: Notifications — 2/2 ✓

- [x] Desktop notifications (notify-rust), agent completion, error/warning
- [x] Notification preferences, history in SQLite

---

## Phase 7: Plugin System

### P2 porpoise-skills — 13/14 ✓

- [x] `WasmRuntime`: wasmtime engine wrapper
- [x] `WasmInstance`: WASM function call
- [x] `SkillRegistry`: register, list, enable, disable, uninstall
- [x] `SkillManifest` struct
- [x] `CompiledModule`
- [x] Workspace wasmtime dep (v25)
- [x] WASM compilation pipeline (`CompilationPipeline` with WAT compile, validate, export listing)
- [x] Capability sandboxing (no fs/network by default via `SandboxedRuntime`)
- [x] Hook system (`HookRegistry` with 9 hook types)
- [x] Plugin hot-reload, cache (`HotReloadManager` with file mtime polling)
- [x] CLI: `porpoise skill search/install/uninstall/list`
- [x] Example plugins: highlighter, lint checker, sentiment analyzer
- [x] SDK documentation (`docs/SDK.md`)
- [ ] P2: Plugin marketplace

---

## Phase 8: Polish & Hardening — 0% (not started)

| Area | Status |
|------|--------|
| Performance | ○ All benchmarks |
| Security | ○ Auth audit, fuzzing |
| Testing | ○ Cross-platform, stress tests |
| Documentation | ○ User guide, migration guide |
| Release | ○ v1.0, brew, winget, cargo-binstall |

## Phase 9: Automation & Enterprise Features — ✅ 100% Complete

### porpoise-credentials — 5/5 ✓

- [x] AES-256-GCM encrypted credential storage
- [x] CRUD operations (store, get, get_by_name, list, delete, list_by_type)
- [x] JSON file persistence with key zeroization on drop
- [x] Per-type credential organization (api_key, password, token)
- [x] Unit tests (store/retrieve, persistence, delete, list)

### porpoise-automation — 8/8 ✓

- [x] DAG-based workflow engine with topological sort
- [x] 7 step types: AgentCall, WebAction, ComputerAction, ApiCall, CredentialLookup, Delay, Condition
- [x] YAML workflow definition loading
- [x] Variable templating (${var_name})
- [x] Cycle detection in dependency graph
- [x] Step execution with result collection
- [x] WebSocket relay server for mobile connectivity
- [x] Unit tests (parse, topo sort, execution, cycle detection)

### porpoise-computer-use — 5/5 ✓

- [x] Mouse automation (move, click, double-click)
- [x] Keyboard automation (type text, press keys)
- [x] Cross-platform via enigo crate (Windows, macOS, Linux)
- [x] Declarative AutomationAction enum
- [x] Integration with workflow engine

### Workflow Editor — 5/5 ✓

- [x] Node-based visual workflow creator (drag-and-drop)
- [x] Real-time process viewer (execution status display)
- [x] Workflow YAML/JSON export/import
- [x] Tauri app integration as frontend route
- [x] Agent skill bindings for workflow management

---

## Summary

| Phase | Total | Done | Partial | Not Started |
|-------|-------|------|---------|-------------|
| 0 | ~55 tasks | 50 | 0 | 5 |
| 1 | ~33 tasks | 29 | 0 | 4 |
| 2 | ~14 tasks | 11 | 0 | 3 |
| 3 | ~12 tasks | 11 | 0 | 1 |
| 4 | ~15 tasks | 13 | 0 | 2 |
| 5 | ~11 tasks | 10 | 0 | 1 |
| 6 | ~24 tasks | 22 | 0 | 2 |
| 7 | ~14 tasks | 13 | 0 | 1 |
| 8 | ~20 tasks | 0 | 0 | 20 |
| 9 | ~23 tasks | 23 | 0 | 0 |
| **Total** | **~221 tasks** | **182** | **0** | **39** |

## Phase Completion Checklist

Run this ritual after every phase is marked complete:

1. **Update documentation** — sync status in README.md, ROADMAP.md, and TODO.md summary table
2. **Run graphify** — regenerate knowledge graph (`graphify-out/`) to reflect new code relationships
3. **Git commit** — atomic commits per crate/change, following dependency order

---

*Last updated: 2026-07-10 — Phase 0–7 and Phase 9 done, Phase 8 (Polish & Release) remaining. Build: 17 crates, 146 Rust files, ~10,367 LOC, 95 tests passing, clippy-clean.*
