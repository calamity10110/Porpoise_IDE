# Installing Porpoise on Windows

## System Requirements

- Windows 10 or 11 (64-bit)
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)
- Visual Studio C++ Build Tools (for building from source)

## Download the Installer

Installers are produced automatically by CI on tagged releases:

1. Go to the repository's **Actions** → **Release** workflow
2. Find the latest successful run
3. Download one of:
   - **porpoise-windows-msi**: Windows Installer (.msi)
   - **porpoise-windows-nsis**: NSIS installer (.exe)

Alternatively, download the CLI binary **porpoise-cli-windows-x86_64.exe**.

## Build Locally

### Prerequisites

1. Install [Rust](https://rustup.rs/) (1.85+)
2. Install Tauri CLI:
   ```powershell
   cargo install tauri-cli --version "^2.0"
   ```
3. WebView2 is pre-installed on Windows 11.
   For Windows 10, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Build

```powershell
# Build the desktop app installer
cd crates/porpoise-app
cargo tauri build

# Build the CLI binary
cd ../..
cargo build --release -p porpoise-cli
```

The installer will be at:
- `crates/porpoise-app/target/release/bundle/nsis/Porpoise_0.1.0_x64-setup.exe`
- `crates/porpoise-app/target/release/bundle/msi/Porpoise_0.1.0_x64.msi`

## Code Signing

The installer is unsigned by default (self-signed). Windows SmartScreen will show a warning.

### Generate a self-signed cert

Run in an **Administrator** PowerShell:

```powershell
.\scripts\generate-dev-cert.ps1
```

Then set environment variables before building:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "C:\path\to\porpoise-dev.pfx"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "your-password"
cargo tauri build
```

To trust the certificate on your machine:
1. Open the `.pfx` file
2. Choose "Local Machine" → "Place all certificates in the following store"
3. Browse → "Trusted Root Certification Authorities"

## Running

After installation:
- **Porpoise Desktop**: Start from Start Menu or desktop shortcut
- **Porpoise CLI**: Add `C:\Program Files\Porpoise\` to your PATH, or use `porpoise --help`

The daemon starts automatically when you launch the desktop app.
You can also start it manually:

```powershell
porpoise-server
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| SmartScreen warning | Click "More info" → "Run anyway" (self-signed) |
| WebView2 not found | Install from Microsoft's website |
| MSI install fails | Run as Administrator |

## Updating

When a new version is released:
1. Download the new installer
2. Run it — it will upgrade over the existing installation
3. Auto-update support coming in a future release
