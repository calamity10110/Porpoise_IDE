# Porpoise Installation Guide

> Complete installation instructions for all platforms and deployment modes.

---

## Quick Start (5 Minutes)

### Option A: Pre-built Binary (Fastest)

```bash
# Download CLI binary
curl -L https://github.com/porpoise-ide/porpoise/releases/latest/download/porpoise-cli-linux-x86_64 -o porpoise
chmod +x porpoise
sudo mv porpoise /usr/local/bin/

# Start daemon
porpoise-server &

# Create your first worktree with an agent
porpoise worktree create --name hello-porpoise --agent claude --prompt "Hello, world"
```

### Option B: Build from Source

```bash
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise
cargo build --release -p porpoise-cli
cargo build --release -p porpoise-server

# Binaries at:
#   target/release/porpoise       (CLI)
#   target/release/porpoise-server (daemon)
```

### Option C: Desktop App (Windows/macOS/Linux)

Download the installer from [GitHub Releases](https://github.com/porpoise-ide/porpoise/releases):
- **Windows**: `.msi` or `.exe` (NSIS installer)
- **macOS**: `.dmg` (coming soon)
- **Linux**: `.AppImage` (coming soon)

---

## Platform-Specific Guides

### Windows

**Prerequisites**:
- Windows 10 or 11 (64-bit)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Win11)
- Visual Studio C++ Build Tools (for building from source)

**Install**:
1. Download `.msi` from Releases
2. Run installer (SmartScreen may warn — click "More info" → "Run anyway" for self-signed)
3. Launch Porpoise from Start Menu

**Build from source**:
```powershell
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise
cargo build --release -p porpoise-app
# Installer at: crates\porpoise-app\target\release\bundle\nsis\Porpoise_0.1.0_x64-setup.exe
```

See [INSTALL_WINDOWS.md](./INSTALL_WINDOWS.md) for detailed guide.

### macOS

**Prerequisites**:
- macOS 12+ (Monterey)
- Xcode Command Line Tools
- Rust 1.85+ (`rustup install stable`)

```bash
brew install pkg-config webkit2gtk-4.1
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise
cargo build --release -p porpoise-cli
cargo build --release -p porpoise-server
```

### Linux

**Prerequisites**:
- Ubuntu 22.04+ / Fedora 38+ / Arch
- `libwebkit2gtk-4.1-dev`, `build-essential`, `pkg-config`, `libssl-dev`

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential pkg-config libssl-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel gcc-c++ pkg-config openssl-devel

# Build
git clone https://github.com/porpoise-ide/porpoise.git
cd porpoise
cargo build --release -p porpoise-cli
```

---

## Deployment Modes

### 1. Local Desktop (Default)

Run daemon + desktop app on your machine. Agents spawn locally.

```
┌─────────────────────────────────┐
│  Your Machine                   │
│  ┌───────────┐  ┌────────────┐  │
│  │ Porpoise  │  │  Daemon    │  │
│  │ Desktop   │←→│ (background)│  │
│  │ App       │  │            │  │
│  └───────────┘  └──────┬─────┘  │
│                        │        │
│                 ┌──────▼─────┐  │
│                 │ Agent Pool │  │
│                 │ Claude     │  │
│                 │ Codex      │  │
│                 │ Gemini     │  │
│                 └────────────┘  │
└─────────────────────────────────┘
```

**Best for**: Individual developers, local development.

### 2. Headless Server (Self-Hosted)

Run daemon on a remote server. Connect via CLI or mobile companion.

```bash
# On the server
ssh user@build-server
porpoise-server &

# On your machine (CLI connects via SSH tunnel)
ssh -L /tmp/porpoise.sock:/tmp/porpoise.sock user@build-server
porpoise worktree list
```

For mobile access:
```bash
# On the server
PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 porpoise-server
porpoise mobile qr  # Scan with phone
```

**Best for**: Teams, CI/CD, remote builds.

### 3. Docker Container

```dockerfile
FROM rust:1.85-slim as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y libwebkit2gtk-4.1-dev pkg-config libssl-dev
RUN cargo build --release -p porpoise-cli -p porpoise-server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/porpoise /usr/local/bin/
COPY --from=builder /app/target/release/porpoise-server /usr/local/bin/
EXPOSE 9876
CMD ["porpoise-server"]
```

```bash
docker build -t porpoise .
docker run -d -p 9876:9876 -v ~/.config/porpoise:/root/.config/porpoise porpoise
```

**Best for**: Reproducible deployments, cloud hosting.

### 4. Cloud (Future)

Porpoise Cloud will offer managed daemon hosting with:
- Pre-configured agent environments
- Shared worktree storage
- Team dashboards
- SSO + audit logs

> **Status**: Not yet available. Self-host today; cloud coming post-v1.

---

## Mobile Companion Install

### Android

1. Download APK from [GitHub Actions artifacts](https://github.com/porpoise-ide/porpoise/actions/workflows/android.yml)
2. Enable "Install from unknown sources" for your file manager
3. Install `app-release.apk`
4. Pair with daemon via QR code or manual entry

See [INSTALL_ANDROID.md](./INSTALL_ANDROID.md) for details.

### iOS

> **Status**: Not yet available. Flutter app is cross-platform but iOS build requires Apple Developer account.

---

## Chrome Extension Install

1. Open `chrome://extensions`
2. Enable **Developer mode** (top right toggle)
3. Click **Load unpacked**
4. Select the `extension/chrome/` directory from the repo
5. Pin the Porpoise icon to your toolbar
6. Click icon → Settings → Enter daemon host/port/token

---

## Post-Install Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORPOISE_CONFIG` | `~/.config/porpoise/config.toml` | Config file path |
| `PORPOISE_WS_PORT` | (disabled) | WebSocket port for mobile/extension |
| `PORPOISE_WS_TLS` | `0` | Enable TLS for WebSocket (`1` = on) |
| `PORPOISE_WS_HOSTS` | `localhost,127.0.0.1` | Certificate SANs (comma-separated) |
| `RUST_LOG` | `info` | Log level (`debug`, `trace`, `warn`, `error`) |

### Config File

```toml
# ~/.config/porpoise/config.toml
[core]
data_dir = "~/.local/share/porpoise"
event_bus_capacity = 1024

[agent]
max_concurrent_agents = 8
agent_timeout_secs = 86400

[db]
path = "~/.local/share/porpoise/porpoise.db"

[runtime]
max_concurrent_processes = 32
process_timeout_secs = 86400
```

### First Run

```bash
# Start daemon (creates data dir, generates session token)
porpoise-server

# Verify it's running
porpoise status

# List available agents (detected from PATH)
porpoise agent list

# Create your first worktree
porpoise worktree create --name my-first-task --agent claude --prompt "Explain this codebase"
```

---

## Verification Checklist

- [ ] `porpoise-server` starts without errors
- [ ] `porpoise status` returns `{"status": "running"}`
- [ ] `porpoise agent list` detects installed agents
- [ ] `porpoise worktree create` creates a git worktree
- [ ] Terminal commands work: `porpoise terminal create --worktree test`
- [ ] Desktop app launches and connects to daemon
- [ ] Mobile app can pair (if `PORPOISE_WS_PORT` set)
- [ ] Chrome extension connects (if loaded)

---

## Troubleshooting

### Daemon won't start

```bash
# Check if another instance is running
porpoise daemon status

# Check logs
cat ~/.local/share/porpoise/logs/porpoise.log

# Reset state (WARNING: deletes all data)
rm -rf ~/.local/share/porpoise
porpoise-server
```

### Agent not detected

```bash
# Verify agent is on PATH
which claude
which codex

# Check detector output
porpoise agent list --json
```

### IPC connection failed

```bash
# Check socket exists
ls -la /tmp/porpoise.sock  # Linux/macOS
# Or: \\.\pipe\porpoise     # Windows

# Verify permissions (Unix)
stat /tmp/porpoise.sock  # Should be 0o700
```

### Mobile can't connect

```bash
# Verify WSS is running
curl -k https://localhost:9876  # Should return WebSocket upgrade error

# Check firewall
sudo ufw allow 9876/tcp

# Verify cert fingerprint matches
porpoise mobile qr
```
