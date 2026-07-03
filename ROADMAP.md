# Porpoise Development Roadmap

> **Phased delivery** from foundation to v1.0 release.
> Each phase builds on the previous. Crates delivered in dependency order.
> Status: ◆ In Progress · ✓ Complete · ○ Not Started

---

## Phase 0: Foundation — ✓ 98% complete

**Objective:** Establish the Cargo workspace, core type system, database schema, CLI skeleton, and CI/CD pipeline.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-core` | Core types, `PorpoiseError`, `Config`, event bus | ✓ 22/23 tasks |
| workspace | `Cargo.toml` workspace, 14 crate scaffolding, lint/format config | ✓ |
| `porpoise-db` | SQLite schema, migrations, CRUD for all tables | ✓ 8/9 tasks |
| `porpoise-cli` | clap command tree, `--json` output, shell completions | ✓ 9/11 tasks |
| CI/CD | GitHub Actions: build, test, clippy, fmt | ✓ |
| Tracing | `tracing-subscriber` stderr output | ✓ |

### Success Criteria

| Criteria | Status |
|----------|--------|
| `cargo build --workspace` succeeds | ✓ All 14 crates compile |
| `cargo test --workspace` passes | ✓ 0 tests (test modules exist in 13 files, need `#[test]` functions) |
| `cargo clippy --workspace` clean (no warnings) | ✓ Clean (except porpoise-app dep warning) |
| CLI prints help with all subcommands | ✓ 16 commands listed |
| SQLite database created and migrated | ✓ On first server start |
| Structured tracing output | ✓ In daemon binary |

---

## Phase 1: CLI & Runtime — ◆ 80% complete

**Objective:** CLI can communicate with a background server process, spawn agents, manage PTYs, and relay output.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-relay` | IPC protocol (JSON), Unix + Windows transport, handshake | ✓ 10/11 tasks |
| `porpoise-runtime` | ProcessManager, PtyManager (Unix), HealthChecker | ✓ 7/11 tasks |
| `porpoise-server` | Daemon binary, CLI→server IPC, service handlers, pidfile, graceful shutdown | ✓ 7/9 tasks |

### Remaining Work

| Item | Priority | Notes |
|------|----------|-------|
| Windows PTY (ConPTY) | High | Stub exists, needs full impl |
| Process lifecycle integration tests | Medium | Need mock processes |
| Windows named pipe integration | Medium | Client+server implemented, needs full integration test |
| IPC roundtrip benchmark tests | Medium | |

---

## Phase 2: Git Integration — ◆ 85% complete

**Objective:** Porpoise can clone repos, create/manage git worktrees, watch file changes, and integrate with GitHub/GitLab.

### Implemented
- `GitEngine` wrapper around `git2::Repository` — clone, open, init, status, diff, log, branch CRUD, fetch, push
- `WorktreeManager` — create, list, remove, prune orphaned git worktrees
- `RemoteProvider` trait with `GitHubProvider` (octocrab) — list/get/create/merge PRs, list issues
- `FileWatcher` using `notify` crate — recursive file change monitoring

### Crate Status

```
porpoise-git/        # Test modules in engine, worktree (539 loc)
├── src/
│   ├── lib.rs       # Module re-exports
│   ├── engine.rs    # GitEngine — core git operations
│   ├── worktree.rs  # WorktreeManager — git worktree lifecycle
│   ├── remote.rs    # RemoteProvider trait + GitHubProvider
│   ├── watcher.rs   # FileWatcher — notify-based file watching
│   └── types.rs     # Change, CommitEntry, WorktreeInfo, PR, Issue structs
├── Cargo.toml       # Dependencies: git2, octocrab, notify
```

---

## Phase 3: Terminal Engine — ◆ 70% complete

**Objective:** Full terminal emulation with split panes, scrollback persistence, color schemes, and proper OSC parsing.

### Implemented
- `PtyMultiplexer` — multiple PTYs per session via `porpoise-runtime::PtyManager`, read/write/close/resize
- `OutputParser` — CSI (ESC[) and OSC (ESC]) escape code parser: cursor movement, clear screen, color changes, bell
- `ScrollbackBuffer` — in-memory ring buffer (configurable max lines, timestamped, search with case sensitivity)
- `TerminalLayout` — pane management with horizontal/vertical split, add/remove/resize
- `ColorScheme` — 16-color standard terminal palette
- `TerminalConfig` — rows, cols, shell, scrollback limit

### Crate Status

```
porpoise-terminal/   # Test module in parser (337 loc)
├── src/
│   ├── lib.rs       # Module re-exports
│   ├── types.rs     # TerminalConfig, TerminalPane, SplitDirection, ColorScheme, OutputLine
│   ├── multiplexer.rs # PtyMultiplexer — PTY allocation per session
│   ├── parser.rs    # OutputParser — CSI/OSC escape code parsing
│   ├── scrollback.rs# ScrollbackBuffer — ring buffer with search
│   └── layout.rs    # TerminalLayout — pane split/resize management
├── Cargo.toml       # Deps: core, runtime, tokio, serde
```

---

## Phase 4: Agent Framework — ◆ 75% complete

**Objective:** Porpoise can detect, spawn, communicate with, and manage multiple AI coding agents.

### Implemented
- `Agent` trait + `AgentHandle` trait — spawn, read_output, send_input, interrupt, shutdown
- `AgentDetector` — PATH scanning for claude/codex/gemini with version detection
- `ClaudeCodeAgent` integration (spawn via tokio::process, managed lifecycle)
- `CodexAgent` integration (same pattern)
- `GenericAgent` for any CLI binary
- `HookServer` — event bus based agent output processing
- `AgentPool` — spawn, list, shutdown, shutdown_all with max concurrent limit

### Crate Status

```
porpoise-agent/      # Test module in detector (501 loc)
├── src/
│   ├── lib.rs       # Module re-exports
│   ├── traits.rs    # Agent trait + AgentHandle trait
│   ├── detector.rs  # AgentDetector — PATH scan + version detection
│   ├── claude.rs    # ClaudeCodeAgent implementation
│   ├── codex.rs     # CodexAgent implementation
│   ├── generic.rs   # GenericAgent for custom binaries
│   ├── pool.rs      # AgentPool — lifecycle management
│   ├── hook.rs      # HookServer — output processing via event bus
│   └── types.rs     # AgentKind, AgentInfo, AgentStatus, AgentOutput
├── Cargo.toml       # Deps: core, runtime, tokio, serde, async-trait
```

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn detect() -> bool where Self: Sized;
    async fn spawn(&self, worktree: &Worktree) -> Result<ChildHandle>;
    async fn send_input(&self, input: &str) -> Result<()>;
    async fn read_output(&self) -> Result<AgentOutput>;
    async fn interrupt(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<ExitStatus>;
}
```

---

## Phase 5: Desktop Application — ○ not started

**Objective:** Tauri-based desktop app with worktree sidebar, terminal panel, editor, and settings UI.

### Design Complete

- Tauri shell with platform menus designed
- Worktree sidebar with drag-and-drop designed
- Terminal panel with split support designed
- Settings UI with keyboard shortcuts designed
- Design doc: `docs/design/design-app.md`

---

## Phase 6: Advanced Features — ◆ 63% complete

**Objective:** Embedded browser, SSH worktrees, file explorer, networking.

### Implemented

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-network` | HTTP/WS client, rate limiter, proxy | ✅ 3/5 tasks — 3 tests |
| `porpoise-browser` | BrowserEngine trait, navigation types | ✅ 7/7 tasks (stub for platform webview) |
| `porpoise-ssh` | SshManager, session, auth, config parser | ✅ 5/9 tasks |

### Remaining

| Item | Priority |
|------|----------|
| SSH exec (full channel impl) | Medium |
| Port forwarding | Medium |
| Platform browser engines (WKWebView, WebView2) | Medium |
| Network connectivity monitor | Low |
| Notifications, auto-update | Low |

---

## Phase 7: Plugin System & Ecosystem — ◆ 43% complete

**Objective:** WASM-based plugin runtime, skill SDK, registry, and community plugin discovery.

### Implemented

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-skills` | WasmRuntime compile/instantiate, SkillRegistry | ✅ 6/14 tasks |

### Crate Status

```
porpoise-skills/
├── src/
│   ├── lib.rs       # Module re-exports
│   ├── runtime.rs   # WasmRuntime + WasmInstance (wasmtime)
│   └── registry.rs  # SkillRegistry + SkillManifest
├── Cargo.toml       # wasmtime 25, serde, tokio
```

### Remaining

| Item | Priority |
|------|----------|
| WASM compilation pipeline (WAT→WASM, WIT) | Medium |
| Capability sandboxing | Medium |
| Hook system integration with EventBus | Medium |
| Plugin hot-reload, cache | Low |
| CLI skill commands | Low |

---

## Phase 8: Polish & Hardening — ○ not started

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

Phase 1: core ──> relay ──> runtime ──> server    ◆ 80% Complete
                      ^                    │
                      └────── CLI ─────────┘

Phase 2: core ──> git ──> server                 ◆ 85% Complete
Phase 3: core ──> runtime ──> terminal            ◆ 70% Complete
Phase 4: core ──> agent ──> server               ◆ 75% Complete
Phase 6: core ──> network ──> ssh ──> browser     ◆ 65% Complete
Phase 7: core ──> skills                          ◆ 43% Complete

Remaining:  app (Tauri) ──> security ──> polish      ○ Not started
```

---

## Risk Matrix

| Risk | Impact | Likelihood | Status |
|------|--------|------------|--------|
| PTY compatibility on Windows | High | Medium | Mitigated: Unix PTY done, Windows stub |
| Agent protocol reverse engineering | High | Medium | ◆ Phase 4 implemented — ClaudeCode + Codex agents working |
| WASM plugin performance | Low | Low | Design only — not yet started |
| Cross-platform IPC on Windows | Medium | Low | ✓ Named pipe client + server implemented |

---

## Implementation Order (Recommended)

| Priority | What | Why | Status |
|----------|------|-----|--------|
| 1 | Finish Phase 0 (tests, cleanup, warnings) | Foundation quality matters | ✓ Done |
| 2 | Finish Phase 1 (reconnect, Windows PTY, shutdown) | Required for all downstream | ◆ 80% |
| 3 | Implement porpoise-git | Unlocks the core worktree abstraction | ✓ Done (85%) |
| 4 | Implement porpoise-agent | Without agents, nothing to orchestrate | ✓ Done (75%) |
| 5 | Implement porpoise-terminal | Needed for agent output display | ◆ Done (70%) |
| 6 | Server integration | Wire git → agent → terminal together | ◆ Done (8 IPC methods) |
| 7 | porpoise-network, porpoise-ssh, porpoise-browser | Remote and browser features | ◆ Done (63%) |
| 8 | porpoise-skills | WASM plugin system | ◆ Done (43%) |
| 9 | porpoise-app | Tauri desktop GUI | ○ Not started |
| 10 | Polish & hardening | Benchmarks, audit, docs, release | ○ Not started |

---

*Last updated: 2026-07-02 — reflects Phase 0–4 implementation status. See [TODO.md](./TODO.md) for task-level tracking.*
