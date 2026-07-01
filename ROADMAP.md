# Porpoise Development Roadmap

> **Phased delivery** from foundation to v1.0 release.
> Each phase builds on the previous. Crates delivered in dependency order.
> Status: ◆ In Progress · ✓ Complete · ○ Not Started

---

## Phase 0: Foundation — ◆ 93% complete

**Objective:** Establish the Cargo workspace, core type system, database schema, CLI skeleton, and CI/CD pipeline.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-core` | Core types, `PorpoiseError`, `Config`, event bus | ✓ 18/23 tasks |
| workspace | `Cargo.toml` workspace, 14 crate scaffolding, lint/format config | ✓ |
| `porpoise-db` | SQLite schema, migrations, CRUD for all tables | ✓ 8/9 tasks |
| `porpoise-cli` | clap command tree, `--json` output, shell completions | ✓ 9/11 tasks |
| CI/CD | GitHub Actions: build, test, clippy, fmt | ✓ |
| Tracing | `tracing-subscriber` stderr output | ✓ |

### Success Criteria

| Criteria | Status |
|----------|--------|
| `cargo build --workspace` succeeds | ✓ All 14 crates compile |
| `cargo test --workspace` passes | ✓ 11 tests pass (core + relay) |
| `cargo clippy --workspace` clean (no warnings) | ◆ ~15 warnings remain (dead code, unused) |
| CLI prints help with all subcommands | ✓ 16 commands listed |
| SQLite database created and migrated | ✓ On first server start |
| Structured tracing output | ✓ In daemon binary |

---

## Phase 1: CLI & Runtime — ◆ 61% complete

**Objective:** CLI can communicate with a background server process, spawn agents, manage PTYs, and relay output.

### Deliverables

| Crate | Milestone | Status |
|-------|-----------|--------|
| `porpoise-relay` | IPC protocol (JSON), Unix socket transport | ✓ 7/11 tasks |
| `porpoise-runtime` | ProcessManager, PtyManager (Unix), HealthChecker | ✓ 7/11 tasks |
| `porpoise-server` | Daemon binary, CLI→server IPC, service handlers | ✓ 5/9 tasks |

### Remaining Work

| Item | Priority | Notes |
|------|----------|-------|
| Windows named pipe transport | High | Stub exists, needs full impl |
| Auto-reconnect with backoff | High | Exponential backoff + jitter |
| Graceful shutdown (drain then exit) | Medium | SIGTERM handler exists but drain logic missing |
| Process lifecycle integration tests | Medium | Need mock processes |
| PID file / single-instance enforcement | Medium | Prevents duplicate daemon |
| Protocol version negotiation | Low | Currently hardcoded to v0x01 |

---

## Phase 2: Git Integration — ○ not started

**Objective:** Porpoise can clone repos, create/manage git worktrees, watch file changes, and integrate with GitHub/GitLab.

### Design Complete

- `GitEngine` wrapper around `git2::Repository` designed
- `WorktreeManager` with create/list/prune designed
- `RemoteProvider` trait with GitHub/GitLab/Gitea/Azure DevOps designed
- File watcher with notify crate (inotify/FSEvents) designed
- Design doc: `docs/design/design-git.md`

### Crate Status

```
porpoise-git/        # Placeholder — compiles, no implementation
├── src/lib.rs       # Minimal placeholder
├── Cargo.toml       # Dependencies configured (git2, octocrab, notify)
```

---

## Phase 3: Terminal Engine — ○ not started

**Objective:** Full terminal emulation with split panes, scrollback persistence, color schemes, and proper OSC parsing.

### Design Complete

- `PtyMultiplexer` for multiple PTYs per session designed
- `OutputParser` for OSC sequences designed
- Scrollback ring buffer → SQLite persistence designed
- Terminal split layout engine designed
- Design doc: `docs/design/design-terminal.md`

---

## Phase 4: Agent Framework — ○ not started

**Objective:** Porpoise can detect, spawn, communicate with, and manage multiple AI coding agents.

### Design Complete

- `Agent` trait with abstract protocol designed
- `ClaudeCodeAgent`, `CodexAgent`, `GeminiAgent` integrations designed
- `HookServer` for agent status detection designed
- Design doc: `docs/design/design-agent.md`

### Agent Protocol Abstraction

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

## Phase 6: Advanced Features — ○ not started

**Objective:** Embedded browser, SSH worktrees, file explorer, notifications, auto-update.

### Design Complete

- `BrowserEngine` trait (WKWebView/webkit2gtk/WebView2) designed — `docs/design/design-browser.md`
- `SshManager` and `SshSession` with auth/port-forwarding designed — `docs/design/design-ssh.md`
- `HttpClient` and `WsClient` with rate limiting designed — `docs/design/design-network.md`
- Notification service and auto-update designed
- Design docs: `design-browser.md`, `design-ssh.md`, `design-network.md`, `design-server.md`

---

## Phase 7: Plugin System & Ecosystem — ○ not started

**Objective:** WASM-based plugin runtime, skill SDK, registry, and community plugin discovery.

### Design Complete

- `wasmtime` runtime with WASI support designed
- WIT-based plugin API designed
- Capability-based sandboxing designed
- Hook system designed
- Design doc: `docs/design/design-skills.md`

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

Phase 1: core ──> relay ──> runtime ──> server    ◆ 61% Complete
                     ^                    │
                     └────── CLI ─────────┘

Phase 2+: core ──> git ──> terminal ──> agent     ○ Design only
          core ──> browser ──> ssh ───> app
          core ──> network ──> skills
```

---

## Risk Matrix

| Risk | Impact | Likelihood | Status |
|------|--------|------------|--------|
| PTY compatibility on Windows | High | Medium | Mitigated: Unix PTY done, Windows stub |
| Agent protocol reverse engineering | High | Medium | Design only — not yet started |
| WASM plugin performance | Low | Low | Design only — not yet started |
| Cross-platform IPC on Windows | Medium | Medium | Unix sockets done, named pipes pending |

---

## Implementation Order (Recommended)

| Priority | What | Why |
|----------|------|-----|
| 1 | Finish Phase 0 (tests, cleanup, warnings) | Foundation quality matters |
| 2 | Finish Phase 1 (reconnect, Windows PTY, shutdown) | Required for all downstream |
| 3 | Implement porpoise-git | Unlocks the core worktree abstraction |
| 4 | Implement porpoise-agent | Without agents, nothing to orchestrate |
| 5 | Implement porpoise-terminal | Needed for agent output display |
| 6 | Server integration | Wire git → agent → terminal together |
| 7 | porpoise-ssh, porpoise-browser, porpoise-network | Remote and browser features |
| 8 | porpoise-skills, porpoise-app | Plugin system and desktop GUI |

---

*Last updated: 2026-07-01 — reflects Phase 0 and Phase 1 implementation status. See [TODO.md](./TODO.md) for task-level tracking.*
