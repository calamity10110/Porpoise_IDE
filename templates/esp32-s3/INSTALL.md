# Installation & Setup Guide

Step-by-step instructions to build and flash the Porpoise ESP32-S3 template.

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | nightly (auto-selected) | Compiler |
| espup | latest | Xtensa toolchain installer |
| espflash | ≥ 3.0 | Flash firmware to device |
| cargo-espflash | ≥ 3.0 | Cargo subcommand for flash |

---

## 1. Install Rust (if not installed)

```bash
# Linux / macOS / WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
winget install Rustlang.Rustup
# or download from https://rustup.rs
```

## 2. Install ESP Toolchain

```bash
# Install espup (manages Xtensa + RISC-V toolchains)
cargo install espup

# Install ESP32-S3 toolchain (Xtensa target)
espup install

# Source environment (add to your shell profile)
# Linux / macOS / WSL:
. $HOME/export-esp.sh

# Windows PowerShell:
# espup creates export-esp.ps1 — run it or add to $PROFILE
. $env:USERPROFILE\export-esp.ps1
```

### Verify

```bash
rustup target list --installed
# Should show: xtensa-esp32s3-none-elf
```

## 3. Install Flash Tools

```bash
cargo install espflash
cargo install cargo-espflash
```

## 4. Clone & Enter Template

```bash
cd templates/esp32-s3
```

## 5. Build

### Option A: Build Scripts (Recommended)

```bash
# Linux / WSL / macOS
./build.sh build                    # default board: waveshare-349
./build.sh build waveshare-349-touch

# Windows PowerShell
.\build.ps1 build
.\build.ps1 build waveshare-349-touch
```

### Option B: Make

```bash
make build                          # default board
make build BOARD=waveshare-349-touch
```

### Option C: Cargo Directly

```bash
cargo build --release --features waveshare-349
```

Output: `target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3`

## 6. Flash

Connect the ESP32-S3 board via USB.

```bash
# Build scripts (auto-detects port)
./build.sh flash
.\build.ps1 flash

# Make
make flash

# Manual (specify port)
espflash flash --monitor --port /dev/ttyUSB0 \
  target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3

# Windows (COM port)
espflash flash --monitor --port COM3 \
  target\xtensa-esp32s3-none-elf\release\porpoise-esp32s3.exe
```

## 7. Monitor Serial Output

```bash
# Via build script
./build.sh monitor
.\build.ps1 monitor

# Manual
espflash monitor --port /dev/ttyUSB0
# Ctrl+] to exit
```

Expected output:
```
=== Porpoise ESP32-S3 Template ===
Board: waveshare-349
CPU: 240 MHz
WiFi: Connected (192.168.1.100)
Auth: enabled (token: a1b2c3d4...)
HTTP :80 (auth), WebSocket :81 (auth)
```

---

## WiFi Configuration

Set WiFi credentials at build time via environment variables:

```bash
# Linux / macOS / WSL
export WIFI_SSID="MyNetwork"
export WIFI_PASS="MyPassword123"

# Windows PowerShell
$env:WIFI_SSID = "MyNetwork"
$env:WIFI_PASS = "MyPassword123"

# Then build
./build.sh build
```

Or edit `src/comms/wifi.rs` directly (not recommended for production).

---

## Available Boards

| Feature | Board |
|---------|-------|
| `waveshare-349` | Waveshare Touch LCD 3.49" (default) |
| `waveshare-349-touch` | Same + touch input enabled |
| `custom-board` | Your own board (see README) |

---

## Quick Test

After flashing, the device should:
1. Connect to WiFi
2. Show auth token on display (if screen attached)
3. Listen on HTTP :80 and WebSocket :81

```bash
# Test HTTP (replace <token> with device token, <ip> with device IP)
curl -H "Authorization: Bearer <token>" http://<ip>/api/info

# Test WebSocket
wscat -c ws://<ip>:81
# Send: {"type":"auth","token":"<token>"}
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `can't find crate for core` | `rustup target add xtensa-esp32s3-none-elf` |
| `linker not found` | Run `espup install` and source `export-esp.sh` |
| `espflash` not found | `cargo install espflash` |
| Device not detected | Check USB cable (data, not charge-only); install CP2102/CH340 drivers |
| WiFi won't connect | Verify SSID/password; check 2.4GHz (ESP32 doesn't support 5GHz) |
| Stack overflow | Increase stack size in `memory.x` |

---

## Next Steps

- Read [README.md](README.md) for full documentation
- Add a custom board: see "Creating a Custom Board" in README
- Add peripherals: see "Adding Components" in README
- Set up OTA: see "OTA Updates" in README
