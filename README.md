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

Monitor and steer agents from your phone. Cross-platform Flutter app with WSS (TLS 1.3) pairing over QR code. Android builds via CI.

</td>
</tr>
<tr>
<td width="50%" valign="middle">

### Chrome Bridge

Chrome extension (Manifest V3) with DOM inspector, page-to-agent sending, and automation playback. Pair via WSS with certificate fingerprint verification.

</td>
<td width="50%" valign="middle">

### Auto-Update

Tauri updater with signed manifests, delta updates, and background installation. Windows installers via NSIS with optional self-signed code signing.

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

# Mobile companion pairing (requires running daemon with PORPOISE_WS_PORT)
porpoise mobile qr

# Output as JSON
porpoise worktree list --json
porpoise status --json
```

---

## Supported Agents

Works with **any CLI agent** — if it runs in a terminal, it runs in Porpoise.

Claude Code · Codex · Grok · Cursor · GitHub Copilot · OpenCode · MiMo Code · Amp · OpenClaude · Antigravity · Pi · oh-my-pi · Hermes Agent · Devin · Goose · Auggie · Autohand Code · Charm · Cline · Codebuff · Command Code · Continue · Droid · Kilocode · Kimi · Kiro · Mistral Vibe · Qwen Code · Rovo Dev · + any CLI agent

---

## Automation & Workflow Engine

Porpoise includes a complete enterprise automation system with workflow orchestration, web/desktop automation, and encrypted credential management.

### Workflow Editor (Desktop)
The Tauri app includes a node-based workflow editor for visually creating automation pipelines. Launch it from the desktop app's toolbar or open `frontend/workflow.html`.

**Features:**
- Drag-and-drop node-based workflow creation
- 7 step types: AgentCall, WebAction, ComputerAction, ApiCall, CredentialLookup, Delay, Condition
- Real-time execution viewer with status indicators
- YAML/JSON export and import
- Workflow templates saved to local storage

### Creating a Workflow (CLI)
```bash
# Run a workflow from a YAML definition file
porpoise workflow run ./deploy.yaml

# List workflow templates
porpoise workflow list

# Validate a workflow definition
porpoise workflow validate ./backup.yaml
```

### Workflow YAML Example
```yaml
id: daily-backup
name: Daily Database Backup
env:
  DB_HOST: ${DB_HOST}
steps:
  - type: CredentialLookup
    id: get-db-pass
    credential_name: db-password
    output_var: DB_PASS
    depends_on: []
  - type: AgentCall
    id: run-backup
    agent_kind: opencode
    prompt: "Run pg_dump on ${DB_HOST} with credentials from vault"
    depends_on: [get-db-pass]
  - type: WebAction
    id: notify-slack
    url: https://hooks.slack.com/services/xxx
    action:
      type: Navigate
    depends_on: [run-backup]
```

### Encrypted Credential Management
```bash
# Store a credential
porpoise credential store --name github-token --type api_key --value ghp_xxxx

# Retrieve a credential (decrypted at read time)
porpoise credential get github-token

# List stored credentials
porpoise credential list --type api_key
```

Credentials are encrypted at rest using AES-256-GCM and stored in `~/.config/porpoise/credentials.json`. The master key is derived from the user's system keychain.

### Desktop GUI Automation
```rust
use porpoise_computer_use::{ComputerUse, ClickButton, AutomationAction};

let mut cu = ComputerUse::new()?;
cu.mouse_move(500, 300)?;        // Move mouse to position
cu.mouse_click(ClickButton::Left)?; // Click
cu.type_text("Hello, world!")?;  // Type text
cu.key_press("enter")?;          // Press Enter key
```

### Agent Skill Integration
Workflows are available as skills that agents can invoke:
```bash
porpoise skill run workflow-creator    # Launch the workflow editor
porpoise skill run workflow-viewer     # View current execution state
porpoise skill run credential-manager  # Manage credentials
```

---

## Project Status

| Phase | Component | Status |
|-------|-----------|--------|
| 0 | Foundation (core, db, cli, CI) | ✅ Complete — 44 tasks |
| 1 | CLI & Runtime (relay, runtime, server) | ✅ Complete — 31 tasks |
| 2 | Git Integration | ✅ Complete — 13 tasks (745 loc) |
| 3 | Terminal Engine | ✅ Complete — 11 tasks (953 loc) |
| 4 | Agent Framework | ✅ Complete — 13 tasks (1,443 loc) |
| 5 | Desktop Application | ✅ Complete — 10 tasks (277 loc + Tauri frontend) |
| 6 | Advanced Features | ✅ Complete — 24/24 tasks (network, ssh, browser, notifications) |
| 7 | Plugin System | ✅ Complete — 14/14 tasks (WASM pipeline, hooks, hot-reload) |
| 8 | Polish & Release | ✅ Complete — 6 security audits, TLS, mobile, Windows MSI, update, Chrome ext |
| 9 | Automation & Enterprise | ✅ Complete — 23 tasks (automation, credentials, UI automation, workflow editor) |

**Total: 160+ Rust files, ~12,200 lines across 17 workspace crates. 117+ tests pass. Builds clean on Windows/macOS/Linux. `cargo clippy -- -D warnings` clean.**

### Architectural Hardening (6-Vector Audit)

All critical findings from the production-grade architectural review have been resolved:

- **Process Lifecycle**: `ProcessManager` and `WorktreeProcessManager` now store `tokio::process::Child` — no more orphaned processes or premature SIGKILL
- **PTY Safety**: `dup()` pattern replaces unsafe `from_raw_fd + mem::forget`. Close sends SIGHUP to process group with `waitpid` reaping
- **Credential Security**: PBKDF2-HMAC-SHA256 KDF (600K iterations) replaces single SHA-256. Cross-platform file locking (fs2) prevents TOCTOU data races
- **WASM Sandbox**: 128MB memory cap + 1MB stack limit + fuel metering on both `WasmRuntime` and `SandboxedRuntime`
- **SSH Non-Blocking**: `std::thread::sleep` replaced with `std::hint::spin_loop()` in exec loop
- **Event Routing**: `HookServer` uses real `AgentId` instead of generating random UUIDs per output event
- **WebSocket Auth**: Exact token matching (no more substring vulnerability)

### v1 Security Findings Closed

| Finding | Fix |
|---------|-----|
| **C1: No TLS** | `rustls-tls` on reqwest + tokio-tungstenite; all HTTPS/WSS encrypted by default |
| **C2: WASM integrity** | BLAKE3 sidecar signatures on `.cwasm` files; tampered modules rejected before `unsafe deserialize` |
| **C3: IPC panic** | Four `try_into().unwrap()` → `Result` propagation; truncated frames no longer crash daemon |
| **C4: SSH zeroize** | Passwords wrapped in `Zeroizing<String>`; wiped from memory on `Drop` |
| **C5: Windows ACL** | `icacls /inheritance:r /grant:r` restricts IPC token file to current user |
| **C6: PTY panic** | Windows PTY read/write/resize now returns `Result` instead of panicking on IO failure |

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
│   ├── porpoise-skills/     # WASM plugin runtime
│   ├── porpoise-automation/ # Workflow automation engine
│   ├── porpoise-credentials/ # Encrypted credential storage
│   └── porpoise-computer-use/ # Desktop GUI automation
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
| [docs/SDK.md](./docs/SDK.md) | Plugin SDK documentation |
| [docs/INSTALL_WINDOWS.md](./docs/INSTALL_WINDOWS.md) | Windows installer guide |
| [docs/INSTALL_ANDROID.md](./docs/INSTALL_ANDROID.md) | Android APK install guide |
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
