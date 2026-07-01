<h1 align="center">
  Porpoise
</h1>

<p align="center">
  <strong>The AI Orchestrator for 100x builders.</strong><br/>
  Rust-native rewrite of <a href="https://github.com/stablyai/orca">Orca</a> — memory-safe, async-native, security-first.<br/>
  Run Codex, Claude Code, OpenCode or Pi side-by-side — each in its own worktree, tracked in one place.
</p>

<p align="center">
  <img src="https://badgen.net/badge/Rust/1.85+/orange" alt="Rust" />
  <img src="https://badgen.net/badge/license/MIT/blue" alt="License" />
  <img src="https://badgen.net/badge/platform/macOS%20|%20Windows%20|%20Linux/green" alt="Platforms" />
</p>

---

## Why Porpoise?

Orca proved the model: parallel agent worktrees, terminal splits, embedded browser, SSH remoting — all in one Electron app. Porpoise takes the same concept and rebuilds it in Rust for:

- **Memory safety** — Rust's ownership eliminates use-after-free, data races, and whole classes of CVEs at compile time
- **Performance** — native compilation vs JIT, 5-10x faster startup, lower memory (no Chromium renderer per window)
- **Security** — type-safe IPC contracts, no prototype pollution, WASM-sandboxed plugins
- **Small binaries** — MBs vs 300MB+ Electron packages

---

## Features

<table>
<tr>
<td width="50%" valign="middle">

### Parallel Worktrees

Fan one prompt across five agents, each in its own isolated git worktree — compare results, merge the winner.

</td>
<td width="50%" valign="middle">

### Terminal Splits

Ghostty-class terminals with native PTY rendering, infinite splits, and scrollback that survives restarts.

</td>
</tr>
<tr>
<td width="50%" valign="middle">

### Orca CLI Compatible

Script every workflow with `porpoise worktree create`, `porpoise terminal send`, `porpoise snapshot` — drop-in CLI for existing Orca workflows.

</td>
<td width="50%" valign="middle">

### Git & Provider Integration

Native GitHub, GitLab, Linear, Jira support. Browse PRs, issues, boards in-app — open a worktree from any task.

</td>
</tr>
<tr>
<td width="50%" valign="middle">

### SSH Worktrees

Run agents on remote servers with full file editing, git, and terminals. Auto-reconnect and port forwarding included.

</td>
<td width="50%" valign="middle">

### Embedded Browser

Chromium-based browser via webkit2gtk with design mode — click any element to send its HTML, CSS, and screenshot to the agent.

</td>
</tr>
<tr>
<td width="50%" valign="middle">

### Skills & Plugins

WASM-sandboxed plugin system. Extend Porpoise without compromising safety. Community plugins run isolated.

</td>
<td width="50%" valign="middle">

### Mobile Companion

Monitor and steer agents from your phone. Cross-platform mobile protocol — bring your own frontend.

</td>
</tr>
</table>

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      porpoise-app (Tauri)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │
│  │ Sidebar  │  │ Terminal │  │ Browser  │  │  Settings  │  │
│  │ Worktree │  │  Splits  │  │  Panel   │  │  / Config  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬──────┘  │
│       │              │              │              │         │
│  ┌────▼──────────────▼──────────────▼──────────────▼──────┐  │
│  │                 porpoise-cli (CLI binary)                │  │
│  │  worktree | terminal | agent | browser | ssh | config   │  │
│  └───────────────────────┬────────────────────────────────┘  │
└──────────────────────────┼───────────────────────────────────┘
                           │ IPC (Unix socket / Named pipe)
┌──────────────────────────▼───────────────────────────────────┐
│                porpoise-server (Daemon)                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐ │
│  │ Runtime  │  │   Git    │  │   SSH    │  │ Agent Pool   │ │
│  │ Manager  │  │  Engine  │  │  Tunnel  │  │ (Claude,     │ │
│  │ (tokio)  │  │  (git2)  │  │ (thrussh)│  │  Codex, etc) │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘ │
│  ┌──────────┐  ┌──────────┐  ┌────────────────────────────┐ │
│  │ Terminal │  │ Network  │  │ Skills/WASM Plugin Runtime  │ │
│  │  Engine  │  │  (reqwest)│  │  (wasmtime)                │ │
│  └──────────┘  └──────────┘  └────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              SQLite (sqlx / rusqlite)                   │ │
│  │  worktrees | sessions | terminals | agents | config    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- Rust 1.85+ (`rustup install stable`)
- System dependencies per platform:

```bash
# macOS
brew install pkg-config webkit2gtk-4.1

# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential pkg-config libssl-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel gcc-c++ pkg-config openssl-devel

# Windows
# Install Visual Studio C++ Build Tools + WebView2 (included in Win11)
```

### Build & Run

```bash
git clone https://github.com/your-org/porpoise
cd porpoise

# Build the CLI
cargo build --release -p porpoise-cli

# Or the full desktop app
cargo build --release -p porpoise-app

# Run the daemon
cargo run --release -p porpoise-server

# Use the CLI
cargo run --release -p porpoise-cli -- --help
```

---

## CLI Usage

```bash
# View all worktrees
porpoise worktree list

# Create a new worktree with an agent
porpoise worktree create --name fix-auth --agent codex --prompt "Fix the auth bug"

# List terminals in a worktree
porpoise terminal list --worktree fix-auth

# Send command to terminal
porpoise terminal send --worktree fix-auth --text "npm test" --enter

# SSH tunnel to remote worktree
porpoise ssh worktree --host build-server --repo ./my-project

# Open embedded browser
porpoise browser open --url http://localhost:3000
porpoise browser snapshot

# Output as JSON
porpoise worktree list --json
porpoise status --json
```

---

## Supported Agents

Works with **any CLI agent** — if it runs in a terminal, it runs in Porpoise.

Claude Code · Codex · Grok · Cursor · GitHub Copilot · OpenCode · MiMo Code · Amp · OpenClaude · Antigravity · Pi · oh-my-pi · Hermes Agent · Devin · Goose · Auggie · Autohand Code · Charm · Cline · Codebuff · Command Code · Continue · Droid · Kilocode · Kimi · Kiro · Mistral Vibe · Qwen Code · Rovo Dev · + any CLI agent

---

## Project Status

| Phase | Component | Status |
|-------|-----------|--------|
| 0 | Foundation (core, db, cli, CI) | ◆ 93% — 40/43 tasks |
| 1 | CLI & Runtime (relay, runtime, server) | ◆ 61% — 19/31 tasks |
| 2 | Git Integration | ○ Design complete |
| 3 | Terminal Engine | ○ Design complete |
| 4 | Agent Framework | ○ Design complete |
| 5 | Desktop Application | ○ Design complete |
| 6 | Advanced Features | ○ Design complete |
| 7 | Plugin System | ○ Design complete |
| 8 | Polish & Release | ○ Not started |

See [ROADMAP.md](./ROADMAP.md) and [TODO.md](./TODO.md) for detailed tracking.

---

## Cargo Workspace

```
porpoise/
├── crates/
│   ├── porpoise-core/       # Types, traits, config, errors
│   ├── porpoise-cli/        # CLI binary (clap)
│   ├── porpoise-runtime/    # Process/PTY manager (tokio)
│   ├── porpoise-git/        # Git operations (git2)
│   ├── porpoise-relay/      # IPC relay protocol
│   ├── porpoise-terminal/   # Terminal emulation
│   ├── porpoise-browser/    # Embedded browser (webkit2gtk)
│   ├── porpoise-ssh/        # SSH connections (thrussh)
│   ├── porpoise-network/    # HTTP/WebSocket (reqwest)
│   ├── porpoise-db/         # SQLite persistence (sqlx)
│   ├── porpoise-server/     # Background daemon
│   ├── porpoise-app/        # Desktop app (Tauri)
│   ├── porpoise-agent/      # Agent integration
│   └── porpoise-skills/     # WASM plugin runtime
├── docs/
├── tests/
└── Cargo.toml               # Workspace manifest
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | Full system architecture |
| [ROADMAP.md](./ROADMAP.md) | Phased development roadmap |
| [TODO.md](./TODO.md) | Granular task tracking |
| [docs/DEVELOPMENT_PLAN.md](./docs/DEVELOPMENT_PLAN.md) | Implementation guide |
| [docs/crates/CORE_TYPES.md](./docs/crates/CORE_TYPES.md) | Core type definitions |
| [docs/crates/RUNTIME_DESIGN.md](./docs/crates/RUNTIME_DESIGN.md) | Runtime design |
| [docs/crates/PROTOCOL_DESIGN.md](./docs/crates/PROTOCOL_DESIGN.md) | IPC protocol |
| [AGENTS.md](./AGENTS.md) | Agent guidelines |

---

## Contributing

Porpoise is in active development. We welcome contributions across all phases.

See [docs/DEVELOPMENT_PLAN.md](./docs/DEVELOPMENT_PLAN.md) for the implementation guide and crate-by-crate breakdown.

### Building

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

---

## License

MIT License — see [LICENSE](./LICENSE).

---

*Porpoise is a Rust-native port of [Orca](https://github.com/stablyai/orca) by stablyai. The Orca project is MIT-licensed. Porpoise inherits the same MIT license and architectural concepts while being a ground-up Rust implementation.*
