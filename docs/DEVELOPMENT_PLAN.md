# Porpoise Development Plan

> Implementation guide covering prerequisites, crate-by-crate breakdown, testing strategy, and deployment.

---

## 1. Prerequisites

### Rust Toolchain

```bash
rustup install stable          # Rust 1.85+
rustup default stable
rustup component add clippy    # Linting
rustup component add rustfmt   # Formatting
rustup target add wasm32-wasi  # For plugin compilation (Phase 7)
cargo install cargo-audit       # Security auditing
cargo install cargo-tarpaulin   # Code coverage
cargo install cargo-watch       # Dev iteration
cargo install just              # Command runner (optional)
```

### System Dependencies

```bash
# macOS
brew install pkg-config webkit2gtk-4.1
brew install pkg-config openssl dbus

# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev libwebkit2gtk-4.1-dev \
     libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
     libxdo-dev libdbus-1-dev

# Fedora
sudo dnf install gcc-c++ pkg-config openssl-devel webkit2gtk4.1-devel \
     gtk3-devel libappindicator-gtk3-devel librsvg2-devel

# Windows (via vcpkg or MSYS2)
# Install Visual Studio 2022 with C++ toolchain
# Install WebView2 runtime (included in Windows 11)
```

---

## 2. Crate-by-Crate Implementation Guide

### 2.1 porpoise-core

**Location:** `crates/porpoise-core/`

**Status:** ✅ **Complete** — 18/23 tasks done. 11 unit tests passing.

**Module Map (implemented):**

```
crates/porpoise-core/src/
├── lib.rs                 # Public API re-exports
├── error.rs               # PorpoiseError enum (30+ typed variants)
├── bus.rs                 # EventBus (tokio broadcast) + tests
├── state.rs               # AppState (Arc<RwLock<>>) + tests
├── config/
│   ├── mod.rs             # AppConfig + 8 sub-configs (all Default)
│   ├── discovery.rs       # Config path discovery (XDG/AppData/env)
│   └── defaults.rs        # Platform-specific defaults (data dir, socket)
├── types/
│   ├── mod.rs
│   ├── id.rs              # 8 newtype IDs with UUID v7 (macro-generated)
│   ├── event.rs           # SystemEvent enum + all sub-events
│   └── capabilities.rs    # Capability-based permission model
└── traits/
    ├── mod.rs
    ├── event_handler.rs   # EventHandler trait (async)
    └── command.rs         # Command trait (typed, async)
```

**Tests:** 11 unit tests passing (IDs roundtrip, event bus pub/sub, app state). Remaining: property-based tests with `proptest`, public API `#[doc]` attributes.

### 2.2 porpoise-db

**Status:** ✅ **Complete** — 8/9 tasks done.

**Location:** `crates/porpoise-db/`

**Module Map (implemented):**

```
crates/porpoise-db/src/
├── lib.rs                 # Public API re-exports
├── schema.rs             # Table definitions + indexes (8 tables)
├── migration.rs          # Versioned migration system (idempotent)
├── pool.rs               # r2d2 connection pool (4 connections, WAL mode)
└── models/
    ├── mod.rs             # Create/Read/Update/Delete traits
    ├── worktree.rs       # WorktreeRow CRUD
    ├── session.rs        # SessionRow CRUD
    ├── terminal.rs       # TerminalRow + HistoryRow CRUD
    ├── agent.rs          # AgentRow CRUD
    └── config.rs         # ConfigEntry key-value store
```

**Key decisions (actual):**
- **rusqlite** with `bundled` feature (no system SQLite needed)
- **r2d2** connection pooling (4 connections, configurable)
- WAL mode, `foreign_keys=ON`, `busy_timeout=5000`
- Migrations embedded in binary, run on server start
- Database at `$PORPOISE_DATA_DIR/porpoise.db`
- All queries parameterized (no string concatenation)
- ISO-8601 text dates (not INTEGER timestamps)

### 2.3 porpoise-cli

**Location:** `crates/porpoise-cli/`

**Module Map:**

```
crates/porpoise-cli/src/
├── main.rs               # Entry point
├── app.rs                # Clap command tree definition
├── output.rs             # Output formatting (plain/JSON/YAML)
├── commands/
│   ├── mod.rs
│   ├── daemon.rs         # daemon start/stop/status
│   ├── worktree.rs       # worktree create/list/show/rm/prune
│   ├── terminal.rs       # terminal create/list/send/read/resize/close
│   ├── agent.rs          # agent list/run/stop/logs
│   ├── git.rs            # git status/diff/log/clone/branch
│   ├── browser.rs        # browser open/snapshot/click/fill
│   ├── ssh.rs            # ssh connect/worktree/port-forward
│   ├── config.rs         # config get/set/list
│   └── skill.rs          # skill search/install/uninstall/list
├── completion.rs         # Shell completion generation
└── config.rs             # CLI-specific config (output format, pager, color)
```

### 2.4 porpoise-runtime

See [RUNTIME_DESIGN.md](./crates/RUNTIME_DESIGN.md) for full details.

**Module Map:**

```
crates/porpoise-runtime/src/
├── lib.rs
├── process/
│   ├── mod.rs
│   ├── manager.rs        # ProcessManager (spawn/monitor/kill)
│   ├── handle.rs         # ProcessHandle with status watch
│   └── pool.rs           # ProcessPool with resource limits
├── pty/
│   ├── mod.rs
│   ├── unix.rs           # Unix PTY (nix::pty::forkpty)
│   ├── windows.rs        # Windows PTY (ConPTY)
│   └── session.rs        # PtySession
├── health.rs             # HealthChecker
├── signal.rs             # Signal handling
└── limits.rs             # Resource limits
```

### 2.5 porpoise-relay

See [PROTOCOL_DESIGN.md](./crates/PROTOCOL_DESIGN.md) for full details.

**Status:** ◆ **Partial** — 7/11 tasks done.

**Module Map (implemented):**

```
crates/porpoise-relay/src/
├── lib.rs                 # Platform-conditional exports (cfg(unix))
├── frame.rs              # Frame encoding/decoding + tests
├── message.rs            # WireMessage enum, Request/Response/Event types
├── transport/
│   ├── mod.rs
│   ├── unix.rs           # UnixSocketTransport (connect/send/receive)
│   └── pipe.rs           # NamedPipeTransport stub (cfg(windows))
├── client.rs             # RelayClient (connect/call via Mutex<UnixSocketTransport>)
├── server.rs             # RelayServer (bind/accept/event loop via read_frame/write_frame)
└── router.rs             # Router (HashMap<method, HandlerFn>, dispatch)
```

**Key changes from initial design:**
- Uses **serde_json** instead of bincode for wire format (de Debbugability)
- No `Transport` trait — uses concrete UnixSocketTransport directly (avoids `async_trait` dyn-compatibility issues)
- `#[cfg(unix)]` on entire IPC layer (named pipes deferred)

### 2.6 porpoise-git

**Module Map:**

```
crates/porpoise-git/src/
├── lib.rs
├── engine.rs             # GitEngine (git2 wrapper)
├── worktree.rs           # WorktreeManager
├── remote/
│   ├── mod.rs            # RemoteProvider trait
│   ├── github.rs         # GitHub via octocrab
│   ├── gitlab.rs         # GitLab via uberactiv
│   ├── gitea.rs          # Gitea provider
│   └── azure.rs          # Azure DevOps provider
├── status.rs             # Status parsing and diff
├── watcher.rs            # File watcher (notify crate)
├── auth.rs               # SSH key / token auth
└── error.rs              # Git-specific errors
```

### 2.7 porpoise-agent

**Module Map:**

```
crates/porpoise-agent/src/
├── lib.rs
├── agent.rs              # Agent trait definition
├── detector.rs           # PATH scanning for installed agents
├── claude.rs             # Claude Code integration
├── codex.rs              # Codex integration
├── gemini.rs             # Gemini/Grok integration
├── generic.rs            # Generic CLI agent fallback
├── hooks.rs              # Agent hook server
├── session.rs            # Session management + resume
└── pool.rs               # AgentPool with concurrency limits
```

### 2.8 porpoise-terminal

**Module Map:**

```
crates/porpoise-terminal/src/
├── lib.rs
├── multiplexer.rs        # PtyMultiplexer (multiple PTYs)
├── parser.rs             # Output parser (OSC, color codes)
├── scrollback.rs         # Ring buffer + SQLite persistence
├── layout.rs             # Terminal split layout engine
├── search.rs             # Scrollback text search
├── color.rs              # Color scheme management
└── error.rs              # Terminal-specific errors
```

### 2.9 porpoise-ssh

**Module Map:**

```
crates/porpoise-ssh/src/
├── lib.rs
├── manager.rs            # SshManager (connect/disconnect/list)
├── session.rs            # SshSession (exec/shell/port-forward)
├── auth.rs               # AuthMethod (key/password/agent)
├── config.rs             # SSH config parser (~/.ssh/config)
├── port_forward.rs       # Port forwarding
├── reconnect.rs          # Auto-reconnect
└── error.rs              # SSH-specific errors
```

### 2.10 porpoise-browser

**Module Map:**

```
crates/porpoise-browser/src/
├── lib.rs
├── engine.rs             # BrowserEngine trait
├── macos.rs              # WKWebView (macOS)
├── linux.rs              # webkit2gtk (Linux)
├── windows.rs            # WebView2 (Windows)
├── navigation.rs         # goto/back/forward/reload
├── design_mode.rs        # Element inspector + screenshot
├── screenshot.rs         # Screenshot pipeline
└── error.rs              # Browser-specific errors
```

### 2.11 porpoise-skills

**Module Map:**

```
crates/porpoise-skills/src/
├── lib.rs
├── runtime.rs            # wasmtime engine wrapper
├── compiler.rs           # WASM compilation pipeline
├── registry.rs           # SkillRegistry (install/list/enable/disable)
├── hooks.rs              # Hook system (agent/terminal events)
├── sandbox.rs            # Capability-based sandboxing
├── manifest.rs           # SkillManifest parsing
└── error.rs              # Plugin-specific errors
```

---

## 3. Testing Strategy

### Unit Tests

- Every public function in every module
- Mock external dependencies (git2, filesystem, network)
- Property-based tests with `proptest` for serialization roundtrips
- Target: 85%+ line coverage

### Integration Tests

Located in `tests/` at workspace root:

```
tests/
├── common/
│   ├── mod.rs            # Test fixtures, helpers
│   └── temp_repo.rs      # Temp git repository fixture
├── cli_tests.rs          # CLI end-to-end tests
├── ipc_tests.rs          # IPC roundtrip tests
├── runtime_tests.rs      # Process lifecycle tests
├── git_tests.rs          # Git operation tests
├── agent_tests.rs        # Agent protocol mock tests
└── db_tests.rs           # Database integration tests
```

### End-to-End Tests

- Scenario-based tests that exercise full workflows
- Spin up server, connect CLI, run commands, verify state
- Use `tempfile` for temporary directories
- Run with `--test-threads=1` to avoid port conflicts

### Testing Tools

| Tool | Purpose |
|------|---------|
| `cargo test` | Standard test runner |
| `cargo-tarpaulin` | Code coverage |
| `proptest` | Property-based testing |
| `assert_cmd` | CLI integration tests |
| `tempfile` | Temporary files/dirs |
| `mockall` | Trait mocking |
| `test-log` | Log output in tests |

---

## 4. Performance Benchmarks

Create benchmarks in each crate:

| Benchmark | Target | Crate |
|-----------|--------|-------|
| IPC roundtrip latency | <500µs p99 | porpoise-relay |
| PTY echo latency | <5ms p95 | porpoise-runtime |
| Git status (10k files) | <500ms | porpoise-git |
| Agent spawn-to-ready | <2s | porpoise-agent |
| Scrollback insert (100k rows) | <100ms | porpoise-terminal |
| Plugin call overhead | <100µs | porpoise-skills |
| Daemon cold start | <500ms | porpoise-server |
| CLI --help output | <50ms | porpoise-cli |

Use `criterion` for microbenchmarks. Track results in `.benchmarks/`.

---

## 5. Security Review Checklist

- [ ] `cargo audit` — zero vulnerabilities
- [ ] All IPC endpoints require authentication
- [ ] No secrets in logs (API keys, tokens, passwords)
- [ ] PTY master fds not accessible to child processes
- [ ] Plugin WASM: no filesystem access, no network access by default
- [ ] SQL injection: all queries parameterized (sqlx/rusqlite)
- [ ] Path traversal: all user-provided paths canonicalized
- [ ] Race conditions: worktree operations serialized via WorktreeManager
- [ ] Signal safety: no async operations in signal handlers
- [ ] Supply chain: `Cargo.lock` committed, Dependabot configured

---

## 6. Code Review Guidelines

### What Reviewers Check

1. **Correctness** — Does the code do what it claims?
2. **Safety** — Any unsafe code? Unwrap/expect in production paths?
3. **Error handling** — Are all error paths handled with `?`?
4. **Async correctness** — Any blocking calls in async functions?
5. **Resource leaks** — File handles, sockets, database connections?
6. **Test coverage** — Are there tests for the new code?
7. **Documentation** — Public items documented with `#[doc]`?
8. **Style** — `cargo fmt` + `cargo clippy` clean?

### Review Process

1. Author opens PR with `[Phase N]` prefix
2. CI passes (build + test + clippy + fmt)
3. At least one maintainer reviews
4. All review comments resolved
5. Squash-merge to main

---

## 7. Build & Deployment

### Local Build

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --workspace

# Specific crate
cargo build --release -p porpoise-cli
cargo build --release -p porpoise-server
cargo build --release -p porpoise-app
```

### CI Pipeline (GitHub Actions)

```yaml
# .github/workflows/ci.yml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo fmt --check
      - run: cargo audit
```

### Release Pipeline

```bash
# Tag the release
git tag v1.0.0
git push origin v1.0.0

# CI builds:
# - porpoise-cli-x86_64-unknown-linux-gnu
# - porpoise-cli-x86_64-apple-darwin
# - porpoise-cli-aarch64-apple-darwin
# - porpoise-cli-x86_64-pc-windows-msvc
# - porpoise-app*.dmg (macOS)
# - porpoise-app*.AppImage (Linux)
# - porpoise-app*.msi (Windows)
# - Cargo publish for library crates
```

### Distribution

| Platform | Method | Format |
|----------|--------|--------|
| macOS | Homebrew | `.tar.gz` + formula |
| Linux | APT/AppImage | `.deb` / `.AppImage` |
| Windows | Scoop/Winget | `.msi` / `.exe` |
| All | GitHub Releases | Platform tarballs |
| All | `cargo install` | Cargo crate |

---

*This development plan evolves as Porpoise progresses. Update sections as implementation reveals new patterns or challenges. See [ROADMAP.md](../ROADMAP.md) for timeline and [TODO.md](../TODO.md) for current task tracking.*
