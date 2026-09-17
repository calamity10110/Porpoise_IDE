# Porpoise IDE — Installation, Usage & Debugging Guide

Porpoise is a Rust-native AI Orchestration IDE (port of Orca) with three components:

| Component | Binary | Purpose |
|-----------|--------|---------|
| Server (daemon) | `porpoise-server.exe` | Background service; owns the database and IPC |
| Desktop app | `porpoise-app.exe` | Tauri GUI (window title "Porpoise") |
| CLI | `porpoise.exe` | Command-line control of the daemon |

> **Note**: The GUI binary is `target\debug\porpoise-app.exe`.
> `Porpoise.exe` (capital P) is the **same file** as the CLI `porpoise.exe` — Windows filenames are case-insensitive.

---

## 1. Installation

### Prerequisites
- Windows 10/11
- [Rust](https://rustup.rs) (stable, MSVC toolchain)
- No Node.js required for regular builds (the app frontend is static files)

### Build from source
```powershell
cd C:\Users\vuanh\Downloads\porpoises\Porpoise_ide

# Stop any running Porpoise processes first — a running exe locks the file
# and makes the build fail with "Access denied / file in use":
taskkill /F /IM porpoise-server.exe 2>$null
taskkill /F /IM porpoise-app.exe 2>$null

cargo build -p porpoise-server -p porpoise-cli -p porpoise-app
```

Artifacts land in `target\debug\`:
- `porpoise-server.exe` — daemon
- `porpoise.exe` — CLI
- `porpoise-app.exe` — desktop GUI

For an optimized build use `--release`; for a distributable installer use `cargo tauri build`.

### Verify the build
```powershell
cargo test                 # full workspace (110 tests)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## 2. Usage

### Step 1 — Start the server (must be first)
```powershell
.\target\debug\porpoise-server.exe
```
Data, database, and the IPC pipe live under:
```
%LOCALAPPDATA%\porpoise\
```
Keep this console window open, or launch it detached and redirect logs to a file.

### Step 2 — Start the desktop app
```powershell
.\target\debug\porpoise-app.exe
```
The GUI connects to the server over the named pipe. If it shows no data, check the server is running (Step 1) — see Troubleshooting.

### Step 3 — Or use the CLI
```powershell
.\target\debug\porpoise.exe daemon status     # real IPC health check (JSON)
.\target\debug\porpoise.exe agent list
.\target\debug\porpoise.exe worktree --help   # worktree / terminal / git / browser / ssh / skills
```
Append `--json` to any command for machine-readable output.

> **Important**: `porpoise status` (without `daemon`) is a hardcoded stub that
> always prints "running". Always use `porpoise daemon status` for the real state.

### ESP32 (optional)
The ESP32-S3 device service is a pure in-memory stub — no hardware is needed.
Nothing to configure; simply don't call the ESP32 endpoints if you have no device.
Available RPC endpoints (via the relay, when a device is present):
`esp32/list`, `esp32/get`, `esp32/command`, `esp32/ota_push`, `esp32/board_templates`.
Requesting an unknown device returns `invalid ID format for device: <device_id>`.

---

## 3. Troubleshooting / Debugging

### Symptom → Cause → Fix

| Symptom | Likely cause | Fix |
|---|---|---|
| CLI: `pipe connect: The system cannot find the file specified. (os error 2)` | Server not running | Start `porpoise-server.exe` (same user session) |
| CLI: `pipe write: The pipe is being closed. (os error 232)` | Server crashed mid-request | Restart server; check its console/log output |
| Build fails, exe "in use / access denied" | A running exe locks `target\debug\` | `taskkill /F /IM porpoise-server.exe` (and `-app`), rebuild |
| GUI window unresponsive / empty | Server was down or crashed when the app started | Restart server, then restart the app |
| Server exits immediately on start | Config/IPC error | Read the error printed to console; check `%LOCALAPPDATA%\porpoise\` is writable |

### Diagnostic commands
```powershell
# Is the daemon actually up? (real IPC round-trip)
.\target\debug\porpoise.exe daemon status
# Expected: {"status":"ok","uptime_seconds":N,...}

# Are the processes alive?
tasklist | findstr /I porpoise

# Is the IPC pipe present? (PowerShell)
[System.IO.Directory]::GetFiles("\\.\pipe\") | Select-String porpoise
# Expected: an entry like C:\Users\<you>\AppData\Local\porpoise\porpoise.sock
```

### Known-fixed bugs (for history / regression checks)
1. **IPC path mismatch** (`crates/porpoise-server/src/lib.rs`): server fell back to
   `%TEMP%` for its data dir while clients used `%LOCALAPPDATA%\porpoise` — they
   could never connect. Fixed by unifying the fallback to
   `AppConfig::default_data_dir()`. If IPC ever breaks again, first verify both
   sides resolve the same `porpoise.sock` path.
2. **Accept-loop crash** (`crates/porpoise-relay/src/transport/pipe.rs`):
   `first_pipe_instance(true)` was set on **every** named-pipe instance creation.
   That flag is only valid for the very first instance; the second one fails with
   os error 5 (Access Denied) and the server exits after the first client
   disconnects. Fixed by removing the flag from the accept loop.
3. **ESP32 compile errors**: missing `use crate::services::esp32_service;` import
   in `relay_handlers.rs` (E0433), and nonexistent `PorpoiseError::NotFound`
   replaced with `PorpoiseError::invalid_id("device", ...)` in `esp32_service.rs`.

4. **GUI totally dead (buttons/settings/terminal do nothing)** — two layers:
   - Tauri auto-appended CSP `sha256-...` hashes for the inline `<script>` in
     `index.html`; per CSP spec, presence of a hash **disables** `'unsafe-inline'`,
     so all 15 inline `onclick="..."` attributes never bind. Fixed in
     `crates/porpoise-app/tauri.conf.json` by setting
     `"dangerousDisableAssetCspModification": true` (the app ships its own
     `script-src 'self' 'unsafe-inline'` policy).
   - The PWA service worker (`sw.js`, cache `porpoise-v1`) served a **stale**
     `index.html` with the old CSP even after a rebuild. After any frontend
     change, wipe it once in the app: DevTools console →
     `navigator.serviceWorker.getRegistrations().then(rs=>rs.forEach(r=>r.unregister())); caches.keys().then(ks=>ks.forEach(k=>caches.delete(k)))`
     then Ctrl+R. (In a debug build you can open DevTools by launching with
     `$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222".)
5. **Default agent is opencode** (was claude). Defaults live in
   `crates/porpoise-app/src/commands.rs` (`default_agent`) and
   `crates/porpoise-server/src/services/config.rs`; the settings dropdown in
   `frontend/index.html` lists `opencode, claude, codex, gemini`.

### Log locations
- Server console / your redirected file (e.g. `server.log` in the repo root)
- `%LOCALAPPDATA%\porpoise\` — data dir
