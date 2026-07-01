# Porpoise — AGENTS.md

> Guidelines for AI coding agents working on Porpoise, the Rust-native port of Orca.

---

## Target Language

Rust stable (edition 2024). `cargo` for builds. `crates.io` for dependencies.

## Design Principles

1. **Memory safety** — No unsafe code outside audited PTY and signal-handling modules.
2. **Async native** — Tokio runtime for all I/O. No blocking calls in async paths.
3. **Type-driven** — Leverage Rust's type system to encode invariants at compile time. Newtype IDs, sealed traits, enum state machines.
4. **Modular** — Each crate is a independently testable unit. Minimize cross-crate coupling.
5. **Security-first** — Capability-based permissions for plugins. Sandboxed WASM runtime. Authenticated IPC.

## Code Convention

- Format: `cargo fmt` with max_width=120, imports_granularity=Crate
- Lint: `cargo clippy --all-targets -- -D warnings`
- No `unwrap()` / `expect()` in production code — use `?` or proper error handling
- Document all public APIs with `#[doc]`
- File/module names are descriptive: `worktree-manager.rs` not `helpers.rs`
- Keep functions under 50 lines; break into smaller focused functions

## Project Structure

```
porpoise/                    # Cargo workspace root
├── crates/
│   ├── porpoise-core/       # Core types, traits, errors, config
│   ├── porpoise-cli/        # CLI binary (clap-based command parsing)
│   ├── porpoise-runtime/    # Process/PTY management (tokio)
│   ├── porpoise-git/        # Git operations (git2 crate)
│   ├── porpoise-relay/      # IPC protocol (bincode, Unix socket/Named pipe)
│   ├── porpoise-terminal/   # Terminal emulation (alacritty_terminal fork, or custom)
│   ├── porpoise-browser/    # Embedded browser (webkit2gtk/WebView2)
│   ├── porpoise-ssh/        # SSH connections (thrussh/ssh2)
│   ├── porpoise-network/    # HTTP/WebSocket networking (reqwest)
│   ├── porpoise-db/         # SQLite persistence (sqlx/rusqlite)
│   ├── porpoise-server/     # Background daemon
│   ├── porpoise-app/        # Desktop application (Tauri)
│   ├── porpoise-agent/      # Agent integration protocols
│   └── porpoise-skills/     # WASM plugin runtime (wasmtime)
├── docs/
│   ├── architecture/
│   ├── crates/              # Crate design documents
│   └── reference/
├── tests/                   # Integration tests
└── Cargo.toml               # Workspace manifest
```

## Development Workflow

1. `cargo build --workspace` — build everything
2. `cargo test --workspace` — run all tests
3. `cargo clippy --all-targets -- -D warnings` — lint
4. `cargo fmt --check` — formatting
5. `cargo doc --no-deps` — documentation

Always run these before committing. CI enforces them.

## Cross-Platform

Porpoise targets macOS (arm64 + x86_64), Linux (x86_64), and Windows (x86_64).

- PTY: `nix::pty::forkpty()` on Unix, `ConPTY API` on Windows
- IPC: Unix domain sockets (macOS/Linux), Named pipes (Windows)
- Browser: WKWebView (macOS), webkit2gtk (Linux), WebView2 (Windows)
- File watcher: FSEvents (macOS), inotify (Linux), ReadDirectoryChangesW (Windows)
- Paths: Use `std::path::PathBuf`, never hardcode `/` or `\`

Use conditional compilation (`#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os = "macos")]`) for platform-specific code.

## References

- [README.md](./README.md) — Project overview
- [ARCHITECTURE.md](./ARCHITECTURE.md) — System architecture
- [ROADMAP.md](./ROADMAP.md) — Development roadmap
- [TODO.md](./TODO.md) — Task tracking
- [docs/DEVELOPMENT_PLAN.md](./docs/DEVELOPMENT_PLAN.md) — Implementation guide
- [docs/crates/CORE_TYPES.md](./docs/crates/CORE_TYPES.md) — Core type design
- [docs/crates/RUNTIME_DESIGN.md](./docs/crates/RUNTIME_DESIGN.md) — Runtime design
- [docs/crates/PROTOCOL_DESIGN.md](./docs/crates/PROTOCOL_DESIGN.md) — IPC protocol design
