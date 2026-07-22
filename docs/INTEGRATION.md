# Porpoise Integration Guide

> How to integrate Porpoise into your development workflow, CI/CD pipeline, and existing tools.

---

## 1. CLI Integration

### Scripting

Porpoise CLI supports JSON output for all commands via `--json` flag:

```bash
# List worktrees as JSON (for scripting)
porpoise --json worktree list

# Create worktree and capture ID
WORKTREE_ID=$(porpoise --json worktree create --name fix-bug | jq -r '.id')

# Spawn agent and wait for completion
porpoise agent run claude --worktree "$WORKTREE_ID" --prompt "Fix the bug"

# Check status
porpoise --json status
```

### CI/CD Pipeline (GitHub Actions)

```yaml
name: AI Code Review
on: [pull_request]

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Porpoise
        run: |
          curl -L https://github.com/porpoise-ide/porpoise/releases/latest/download/porpoise-cli-linux-x86_64 -o /usr/local/bin/porpoise
          chmod +x /usr/local/bin/porpoise
      - name: Start daemon
        run: porpoise-server &
      - name: Create review worktree
        run: |
          porpoise worktree create --name pr-review --repo .
          porpoise agent run claude --worktree pr-review --prompt "Review this PR for bugs"
      - name: Post review
        run: porpoise --json agent logs pr-review >> $GITHUB_STEP_SUMMARY
```

### Shell Completions

```bash
# Bash
porpoise completion bash > /etc/bash_completion.d/porpoise

# Zsh
porpoise completion zsh > "${fpath[1]}/_porpoise"

# Fish
porpoise completion fish > ~/.config/fish/completions/porpoise.fish
```

---

## 2. Desktop App Integration

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `CmdOrCtrl+N` | New worktree |
| `CmdOrCtrl+T` | New terminal |
| `CmdOrCtrl+,` | Settings |
| `CmdOrCtrl+Q` | Quit |
| `CmdOrCtrl+Shift+W` | Close window |

### IPC Commands (Tauri)

The desktop app exposes IPC commands callable from the frontend JavaScript:

```javascript
// List worktrees
const worktrees = await window.__TAURI__.core.invoke('list_worktrees');

// Create worktree
await window.__TAURI__.core.invoke('create_worktree', {
  name: 'fix-auth',
  repo: '/path/to/repo'
});

// Get daemon status
const status = await window.__TAURI__.core.invoke('get_status');
```

### Workflow Editor

Launch the visual workflow editor from the desktop app toolbar, or open `frontend/workflow.html` directly.

Supported step types:
- `AgentCall` — spawn an agent with a prompt
- `WebAction` — navigate/click/fill in embedded browser
- `ComputerAction` — mouse/keyboard automation
- `ApiCall` — HTTP request
- `CredentialLookup` — fetch from encrypted vault
- `Delay` — wait
- `Condition` — branch on variable

---

## 3. Mobile Companion Setup

### Prerequisites

- Porpoise daemon running on desktop with WSS enabled
- Android 7.0+ device on same network (or tunnel)

### Pairing

1. **Desktop**: Start daemon with mobile support:
   ```bash
   PORPOISE_WS_PORT=9876 PORPOISE_WS_TLS=1 porpoise-server
   ```

2. **Desktop**: Get pairing info:
   ```bash
   porpoise mobile qr
   ```
   Output:
   ```json
   {
     "url": "wss://192.168.1.100:9876",
     "token": "a1b2c3d4...",
     "tls_fingerprint": "sha256:e5f6..."
   }
   ```

3. **Mobile**: Open Porpoise Mobile app → enter host, port, and token → Connect

4. **Verify**: The TLS fingerprint on your phone must match the daemon output. This is TOFU (Trust On First Use).

### Mobile Features

| Feature | Description |
|---------|-------------|
| Worktree list | View all active worktrees |
| Agent monitor | See agent status, output, completion |
| Terminal view | Read terminal output (read-only) |
| Push notifications | Agent completion/failure alerts |
| Settings | Configure connection, TLS, notifications |

---

## 4. Chrome Extension Setup

### Install

1. Open `chrome://extensions`
2. Enable **Developer mode** (top right)
3. Click **Load unpacked**
4. Select `extension/chrome/` directory

### Pair

1. Click the Porpoise icon in Chrome toolbar
2. Enter daemon host, port, and auth token
3. Toggle "Use TLS" if daemon uses WSS
4. Click **Connect**

### Features

| Feature | Trigger | Description |
|---------|---------|-------------|
| **Send page to agent** | Right-click → "Send to Porpoise" | Sends URL + title to selected agent |
| **Inspect element** | Popup → "Inspect Mode" | Click any element → captures HTML, CSS, selector |
| **Automation playback** | Via RPC `browser/play_sequence` | Replays click/type/wait sequences |
| **Status badge** | Always visible | Green = connected, red = disconnected |

### Content Script API

The extension's content script responds to these messages:

```javascript
// Toggle DOM inspector
chrome.tabs.sendMessage(tabId, { type: "inspect_mode", enable: true });

// Capture full page
chrome.tabs.sendMessage(tabId, { type: "capture_page" });

// Play automation sequence
chrome.tabs.sendMessage(tabId, {
  type: "play_sequence",
  steps: [
    { type: "click", selector: "#submit" },
    { type: "type", selector: "#input", value: "hello" },
    { type: "wait", ms: 1000 },
    { type: "extract", selector: ".result" }
  ]
});
```

---

## 5. WASM Plugin SDK

### Create a Plugin

Plugins are WASM modules compiled from WAT (WebAssembly Text) or WASM binaries.

```wat
;; example.wat — a simple highlighter plugin
(module
  (func (export "on_agent_output") (param $output i32) (result i32)
    local.get $output
    ;; process output...
  )
)
```

### Install a Plugin

```bash
# Compile and cache
porpoise skill install ./my-plugin.wat

# List installed
porpoise skill list

# Uninstall
porpoise skill uninstall my-plugin
```

### Security Model

| Capability | Default | How to Grant |
|------------|---------|--------------|
| Filesystem read | ❌ Denied | Capability in manifest |
| Filesystem write | ❌ Denied | Capability in manifest |
| Network | ❌ Denied | Capability in manifest |
| Process spawn | ❌ Denied | Capability in manifest |
| Memory | 128MB max | Configured at compile time |
| Stack | 1MB max | Configured at compile time |
| CPU | Fuel-metered | 10,000 units default |

All `.cwasm` files are integrity-checked via BLAKE3 hash sidecar before deserialization.

---

## 6. RPC API Reference

The daemon exposes an RPC interface over IPC (Unix socket / Named pipe) and WebSocket.

### Authentication

All connections require a session token. The token is generated on daemon start and stored at `data_dir/ipc-token`.

### Methods

| Method | Parameters | Returns |
|--------|------------|---------|
| `worktree_create` | `name`, `repo?`, `agent?`, `prompt?` | `{ id, path, branch }` |
| `worktree_list` | — | `[{ id, name, status, agent }]` |
| `worktree_rm` | `name` | `{ status: "removed" }` |
| `terminal_create` | `worktree`, `shell?` | `{ id }` |
| `terminal_send` | `id`, `text`, `enter` | `{ status: "ok" }` |
| `terminal_read` | `id` | `{ data, bytes }` |
| `terminal_resize` | `id`, `rows`, `cols` | `{ status: "ok" }` |
| `terminal_close` | `id` | `{ status: "closed" }` |
| `agent_list` | — | `[{ id, kind, status }]` |
| `agent_run` | `kind`, `worktree`, `prompt` | `{ id }` |
| `agent_stop` | `id` | `{ status: "stopped" }` |
| `git_status` | `repo_path` | `{ branch, staged, modified, untracked }` |
| `git_diff` | `repo_path`, `staged?` | `{ diff }` |
| `git_log` | `repo_path`, `count?` | `[{ hash, author, message }]` |
| `browser_open` | `url` | `{ page_id }` |
| `browser_snapshot` | `page_id` | `{ html, title, url }` |
| `ssh_connect` | `host`, `port`, `user`, `password?`, `key_path?` | `{ session_id }` |
| `mobile/pairing_info` | — | `{ host, port, token, tls_fingerprint }` |
| `health` | — | `{ uptime, agents, status }` |
| `daemon_stop` | — | `{ status: "stopping" }` |

### Events (Server-Pushed)

| Event | Payload |
|-------|---------|
| `TerminalOutput` | `{ id, data, timestamp }` |
| `TerminalBell` | `{ id }` |
| `AgentStatus` | `{ id, status }` |
| `AgentOutput` | `{ id, text }` |
| `WorktreeCreated` | `{ id, path }` |
| `WorktreeDeleted` | `{ id }` |

### Protocol

Binary frames with 8-byte header:

```
┌────────┬────────┬──────────┬──────────────┐
│ Magic  │ Version│ Length   │ Payload      │
│ 0x5050 │ 0x01   │ (4B LE)  │ (JSON)       │
│ (2B)   │ (1B)   │          │ (variable)   │
└────────┴────────┴──────────┴──────────────┘
```

Version negotiation: server enforces `MIN_PROTOCOL_VERSION` range on every frame.

---

## 7. Credential Vault Integration

### Store a Credential

```bash
porpoise credential store --name github-token --type api_key --value ghp_xxxx
```

### Retrieve in a Workflow

```yaml
steps:
  - type: CredentialLookup
    credential_name: github-token
    output_var: GH_TOKEN
  - type: AgentCall
    prompt: "Use token ${GH_TOKEN} to create a PR"
```

### Security

- AES-256-GCM encryption at rest
- PBKDF2-HMAC-SHA256 key derivation (600,000 iterations in release)
- File locking (fs2) prevents TOCTOU races
- Zeroized on Drop
- Master key from user-provided passphrase
