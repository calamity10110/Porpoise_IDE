# Porpoise Development Roadmap

> **Phased delivery** from foundation to v1.0 release.
> Each phase builds on the previous. Crates delivered in dependency order.
> Status: ◆ In Progress · ✓ Complete · ○ Not Started

---

## Phase 0: Foundation — ✓ Complete

**Objective:** Establish the Cargo workspace, core type system, database schema, CLI skeleton, and CI/CD pipeline.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-core` | Core types, `PorpoiseError`, `Config`, event bus | ✓ 23/24 tasks |
| workspace | `Cargo.toml` workspace, 14 crate scaffolding, lint/format config | ✓ |
| `porpoise-db` | SQLite schema, migrations, CRUD for all tables | ✓ 8/10 tasks |
| `porpoise-cli` | clap command tree, `--json` output, shell completions, integration tests | ✓ 11/12 tasks |
| CI/CD | GitHub Actions: build, test, clippy, fmt, audit | ✓ |
| Tracing | `tracing-subscriber` with rotating file appender | ✓ |

### Success Criteria

| Criteria | Status |
|----------|--------|
| `cargo build --workspace` succeeds | ✓ All 14 crates compile |
| `cargo clippy --workspace` clean (-D warnings) | ✓ Clean |
| CLI prints help with all subcommands | ✓ 16 commands listed |
| SQLite database created and migrated | ✓ On first server start |
| Structured tracing output | ✓ With `RotatingLogFile` rotation |

---

## Phase 1: CLI & Runtime — ✓ Complete

**Objective:** CLI can communicate with a background server process, spawn agents, manage PTYs, and relay output.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-relay` | IPC protocol (JSON), Unix + Windows transport, handshake | ✓ 10/11 tasks |
| `porpoise-runtime` | ProcessManager, PtyManager, HealthChecker, LogRotation, WorktreeProcessManager | ✓ 11/11 tasks |
| `porpoise-server` | Daemon binary, IPC, services, pidfile, shutdown, log rotation, notifications | ✓ 11/11 tasks |

### Implemented in this phase

- `RotatingLogFile` — size-based log rotation with configurable max files
- `WorktreeProcessManager` — process spawning scoped to worktree directories with per-worktree limits
- Process lifecycle integration tests (spawn, kill, shutdown_all, invalid command)
- Desktop notification service with SQLite-backed history and `notify-rust`

### Remaining

| Item | Priority | Notes |
|------|----------|-------|
| Windows PTY (ConPTY) | Medium | Stub exists, full impl needs Windows testing |
| IPC roundtrip benchmark tests | Low | |
| Server stress tests | Low | |

---

## Phase 2: Git Integration — ✓ Complete

**Objective:** Porpoise can clone repos, create/manage git worktrees, watch file changes, and integrate with GitHub.

### Implemented

- `GitEngine` — clone, open, init, status, diff, log, branch CRUD, fetch, push, stash/pop
- `WorktreeManager` — create, list, remove, prune orphaned
- `RemoteProvider` trait with `GitHubProvider` (octocrab) — list/get/create/merge PRs, list issues
- `FileWatcher` using `notify` crate
- **SSH git support** — `SshCredentials` enum (KeyFile, Agent, DefaultKey), `clone_ssh`, `push_ssh`, `fetch_ssh` via `git2::RemoteCallbacks`

### Crate Status

```
porpoise-git/        # 745 loc, 7 files
├── src/
│   ├── lib.rs       # Module re-exports
│   ├── engine.rs    # GitEngine — core git operations + stash
│   ├── worktree.rs  # WorktreeManager — git worktree lifecycle
│   ├── remote.rs    # RemoteProvider trait + GitHubProvider
│   ├── ssh.rs       # SSH key auth (clone/push/fetch)
│   ├── watcher.rs   # FileWatcher — notify-based file watching
│   └── types.rs     # Change, CommitEntry, WorktreeInfo, PR, Issue structs
├── Cargo.toml       # git2, octocrab, notify
```

---

## Phase 3: Terminal Engine — ✓ Complete

**Objective:** Full terminal emulation with split panes, scrollback persistence, color schemes, regex search, and reflow.

### Implemented

- `PtyMultiplexer` — multiple PTYs per session
- `OutputParser` — CSI/OSC escape code parser
- `ScrollbackBuffer` — ring buffer with regex search (`search_regex`, `search_advanced`)
- `TerminalLayout` — pane management with split/resize
- `ColorScheme` — 16-color palette with **Alacritty YAML import** and **iTerm2 plist import**
- 3 built-in color schemes: Tokyo Night, Dracula, Solarized Dark
- `SqliteScrollbackStore` — SQLite-backed scrollback with batched writes, pruning, load-by-limit
- `reflow_lines` — reflows scrollback on terminal resize
- `ScrollbackPersister` — NDJSON file-based fallback persistence

### Crate Status

```
porpoise-terminal/   # 953 loc, 10 files
├── src/
│   ├── lib.rs
│   ├── types.rs        # TerminalConfig, ColorScheme, OutputLine, TerminalPane
│   ├── multiplexer.rs  # PtyMultiplexer
│   ├── parser.rs       # OutputParser — CSI/OSC parsing
│   ├── scrollback.rs   # ScrollbackBuffer — ring buffer + regex search
│   ├── sqlite_scrollback.rs # SQLite-backed scrollback store
│   ├── persist.rs      # NDJSON file-based persister
│   ├── layout.rs       # TerminalLayout — pane split/resize
│   ├── theme.rs        # Alacritty YAML + iTerm2 plist import + built-in schemes
│   └── reflow.rs       # Line reflow on resize
```

---

## Phase 4: Agent Framework — ✓ Complete

**Objective:** Detect, spawn, communicate with, and manage multiple AI coding agents.

### Implemented

- `Agent` trait + `AgentHandle` trait
- `AgentDetector` — PATH scanning for claude/codex/gemini
- `ClaudeCodeAgent`, `CodexAgent`, `GeminiAgent`, `GenericAgent`
- `HookServer` — event bus based output processing
- `AgentPool` — lifecycle management with max concurrent limit
- `SessionStore` — SQLite-backed session resume (create, resume, list, status tracking)
- `AccountSwitcher` — multi-account management with JSON persistence
- `TokenUsageMonitor` — token/cost tracking with per-agent/session summaries

### Crate Status

```
porpoise-agent/      # 1,443 loc, 13 files
├── src/
│   ├── lib.rs
│   ├── traits.rs    # Agent + AgentHandle traits
│   ├── detector.rs  # AgentDetector
│   ├── claude.rs    # ClaudeCodeAgent
│   ├── codex.rs     # CodexAgent
│   ├── gemini.rs    # GeminiAgent
│   ├── generic.rs   # GenericAgent
│   ├── pool.rs      # AgentPool
│   ├── hook.rs      # HookServer
│   ├── types.rs     # AgentKind, AgentInfo, AgentStatus
│   ├── resume.rs    # SessionStore — SQLite session resume
│   ├── account.rs   # AccountSwitcher — multi-account
│   └── usage.rs     # TokenUsageMonitor — cost tracking
```

---

## Phase 5: Desktop Application — ✓ 90% Complete

**Objective:** Tauri-based desktop app with worktree sidebar, terminal panel, editor, and settings UI.

### Implemented

- Tauri 2.x app shell with platform menus (File/Edit/View/Window/Help)
- System tray with show/hide/quit and left-click toggle
- Keyboard shortcuts (CmdOrCtrl+N/T/,/Q)
- Worktree sidebar with tree view and add/remove
- Split terminal panes with horizontal/vertical split
- Editor pane with contenteditable markdown
- Settings modal with tabs (general, agent, git, keyboard)
- Status bar with connection/agent/terminal counts
- IPC command handlers (worktrees, agents, terminals, settings)

### Crate Status

```
porpoise-app/        # 249 loc, 4 files
├── src/
│   ├── main.rs      # Entry point
│   ├── lib.rs       # Tauri builder, menu, tray
│   └── commands.rs  # IPC command handlers
├── frontend/
│   └── index.html   # Full UI with sidebar, terminal, editor, settings
├── capabilities/
│   └── default.json # Tauri permissions
├── tauri.conf.json
└── Cargo.toml       # tauri 2, tray-icon, opener plugin
```

---

## Phase 6: Advanced Features — ✓ 92% Complete

**Objective:** Embedded browser, SSH worktrees, networking, notifications.

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-network` | HTTP/WS client, rate limiter, proxy, connectivity monitor | ✓ 5/5 tasks |
| `porpoise-browser` | BrowserEngine trait, navigation types | ✓ 7/7 tasks |
| `porpoise-ssh` | SshManager, session, auth, config, keepalive, exec, port forwarding, remote worktree | ✓ 8/9 tasks |
| Notifications | Desktop notifications, preferences, SQLite history | ✓ 2/2 tasks |

### Remaining

| Item | Priority |
|------|----------|
| Platform browser engines (WKWebView, WebView2) | Medium |
| SFTP file browser | Low |

---

## Phase 7: Plugin System & Ecosystem — ✓ 93% Complete

**Objective:** WASM-based plugin runtime, skill SDK, registry, hooks, hot-reload, and community plugins.

### Implemented

| Feature | Status |
|---------|--------|
| WasmRuntime compile/instantiate | ✓ |
| SkillRegistry + SkillManifest | ✓ |
| SandboxedRuntime with capability enforcement | ✓ |
| CompilationPipeline (WAT→WASM, validate, export listing, .cwasm cache) | ✓ |
| HookRegistry (9 hook types, async fire, error collection) | ✓ |
| HotReloadManager (file mtime polling, cache) | ✓ |
| CLI skill commands (search/install/uninstall/list) | ✓ |
| Example plugins (highlighter, lint_checker, sentiment_analyzer) | ✓ |
| SDK documentation | ✓ |

### Crate Status

```
porpoise-skills/     # 726 loc, 7 files
├── src/
│   ├── lib.rs
│   ├── runtime.rs    # WasmRuntime + WasmInstance
│   ├── registry.rs   # SkillRegistry + SkillManifest
│   ├── sandbox.rs    # SandboxedRuntime — capability enforcement
│   ├── pipeline.rs   # CompilationPipeline — WAT→WASM, validate, exports
│   ├── hooks.rs      # HookRegistry — 9 lifecycle hooks
│   └── hot_reload.rs # HotReloadManager — file watcher + cache
```

---

## Phase 8: Polish & Hardening — ○ Not Started

**Objective:** Production readiness — performance, security audit, final testing, v1.0.

### Performance Targets (not yet benchmarked)

| Metric | Target |
|--------|--------|
| Daemon cold start | <500ms |
| CLI response (query) | <50ms |
| PTY latency (input→echo) | <5ms p95 |
| Memory per agent session | <50MB baseline |

---

## Dependency Graph — Current State

```
Phase 0: core ──> db ──> cli                    ✓ Complete
                 \
                  └─> CI/CD                       ✓ Complete

Phase 1: core ──> relay ──> runtime ──> server    ✓ Complete
                       ^                    │
                       └────── CLI ─────────┘

Phase 2: core ──> git ──> server                 ✓ Complete
Phase 3: core ──> runtime ──> terminal            ✓ Complete
Phase 4: core ──> agent ──> server               ✓ Complete
Phase 5: app (Tauri)                              ✓ 90% Complete
Phase 6: core ──> network ──> ssh ──> browser     ✓ 92% Complete
Phase 7: core ──> skills                          ✓ 93% Complete

Remaining:  polish ──> release                      ○ Not started
```

---

## Risk Matrix

| Risk | Impact | Likelihood | Status |
|------|--------|------------|--------|
| PTY compatibility on Windows | High | Medium | Mitigated: Unix PTY done, Windows stub |
| Agent protocol reverse engineering | High | Medium | ✓ Resolved — Claude/Codex/Gemini agents working |
| WASM plugin performance | Low | Low | ✓ Resolved — wasmtime with fuel metering |
| Cross-platform IPC on Windows | Medium | Low | ✓ Named pipe client + server implemented |

---

## Implementation Order (Recommended)

| Priority | What | Why | Status |
|----------|------|-----|--------|
| 1 | Finish Phase 0 (tests, cleanup, warnings) | Foundation quality matters | ✓ Done |
| 2 | Finish Phase 1 (reconnect, Windows PTY, shutdown) | Required for all downstream | ✓ Done |
| 3 | Implement porpoise-git | Unlocks the core worktree abstraction | ✓ Done |
| 4 | Implement porpoise-agent | Without agents, nothing to orchestrate | ✓ Done |
| 5 | Implement porpoise-terminal | Needed for agent output display | ✓ Done |
| 6 | Server integration | Wire git → agent → terminal together | ✓ Done |
| 7 | porpoise-network, porpoise-ssh, porpoise-browser | Remote and browser features | ✓ Done |
| 8 | porpoise-skills | WASM plugin system | ✓ Done |
| 9 | porpoise-app | Tauri desktop GUI | ✓ Done (90%) |
| 10 | Polish & hardening | Benchmarks, audit, docs, release | ○ Not started |

---

*Last updated: 2026-07-10 — reflects Phase 0–7 implementation completion. See [TODO.md](./TODO.md) for task-level tracking.*
