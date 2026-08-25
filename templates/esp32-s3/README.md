# Porpoise ESP32-S3 Template — Complete Guide

A `no_std` Rust template for ESP32-S3 boards with modular peripherals, secure remote control via WiFi, and integration with the Porpoise IDE.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Project Structure](#project-structure)
4. [Creating a Custom Board](#creating-a-custom-board)
5. [Adding Components](#adding-components)
6. [Creating Custom Sensors](#creating-custom-sensors)
7. [Security Model](#security-model)
8. [WiFi Configuration](#wifi-configuration)
9. [Remote Control Protocol](#remote-control-protocol)
10. [OTA Updates](#ota-updates)
11. [Troubleshooting](#troubleshooting)

---

## Quick Start

```bash
# 1. Install Rust + ESP32 toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add xtensa-esp32s3-none-elf
cargo install espflash

# 2. Clone and build
cd templates/esp32-s3
cargo build --release --features waveshare-349

# 3. Flash (hold BOOT button, then)
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3

# 4. Connect
# Device starts in AP mode: SSID "Porpoise-Setup", password shown on screen
# Open Porpoise IDE → Add Device → Enter token displayed on device
```

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ | `rustup` |
| ESP32 target | latest | `rustup target add xtensa-esp32s3-none-elf` |
| espflash | 2.0+ | `cargo install espflash` |
| ESP-IDF (optional) | 5.1+ | For advanced WiFi/BLE features |

### Supported Boards (built-in)

| Board | Feature Flag | Display | Touch | IMU |
|-------|-------------|---------|-------|-----|
| Waveshare LCD 3.49" (SPI) | `waveshare-349` | GC9A01 240×240 | CST816S | — |
| Waveshare LCD 3.49" (RGB) | `waveshare-349-touch` | ST7701S 480×480 | GT911 | QMI8658 |
| Custom | `custom-board` | User-defined | User-defined | User-defined |

---

## Project Structure

```
esp32-s3/
├── Cargo.toml                  # Dependencies + feature flags
├── boards/
│   ├── waveshare_lcd_349/      # SPI variant board config
│   │   ├── config.toml         # Pin mapping
│   │   └── mod.rs              # BoardConfig impl
│   ├── waveshare_lcd_349_touch/ # RGB variant board config
│   │   ├── config.toml
│   │   └── mod.rs
│   └── custom_example/         # Template for custom boards
│       ├── config.toml
│       └── mod.rs
├── src/
│   ├── main.rs                 # Boot sequence + main loop
│   ├── board/                  # Board config loader
│   ├── comms/
│   │   ├── mod.rs              # Comms module root
│   │   ├── wifi.rs             # WiFi STA/AP manager
│   │   ├── http_server.rs      # REST API (auth-enabled)
│   │   ├── websocket.rs        # WebSocket (auth-enabled)
│   │   ├── protocol.rs         # JSON wire format
│   │   └── security.rs         # Token auth + rate limiting
│   ├── orchestration/
│   │   ├── mod.rs              # Orchestrator
│   │   └── command_dispatcher.rs # Command routing
│   ├── peripherals/
│   │   ├── mod.rs              # Peripheral trait + registry
│   │   ├── display.rs          # Display trait
│   │   ├── touch.rs            # Touch trait
│   │   ├── audio.rs            # Audio I/O traits
│   │   ├── camera.rs           # Camera trait
│   │   ├── imu.rs              # IMU trait
│   │   ├── mouse.rs            # Mouse trait
│   │   ├── keyboard.rs         # Keyboard trait
│   │   └── custom.rs           # Custom peripheral trait
│   └── sensors/
│       └── mod.rs              # Sensor trait + registry
└── README.md                   # This file
```

---

## Creating a Custom Board

### Step 1: Create Board Directory

```bash
mkdir -p boards/my_board
```

### Step 2: Write `config.toml`

```toml
# boards/my_board/config.toml
[board]
name = "my_board"
manufacturer = "MyCompany"
display_width = 320
display_height = 240
psram_size = 8388608    # 8MB
flash_size = 16777216   # 16MB

[display]
type = "SPI"            # or "I2C" or "RGB"
interface = "SPI"
width = 320
height = 240
color_format = "RGB565"

[display.spi]
sclk = 12
mosi = 11
cs = 10
dc = 9
reset = 8
backlight = 7
spi_freq_hz = 40000000

[touch]
type = "CST816S"
interface = "I2C"
i2c_sda = 4
i2c_scl = 5
i2c_addr = "0x15"
irq_pin = 6

[imu]
type = "QMI8658"
interface = "I2C"
i2c_sda = 4
i2c_scl = 5
i2c_addr = "0x6B"

[audio.mic]
type = "I2S"
i2s_sck = 16
i2s_ws = 17
i2s_sd = 15

[audio.speaker]
type = "I2S"
i2s_bclk = 46
i2s_ws = 3
i2s_dout = 9
```

### Step 3: Write `mod.rs`

```rust
// boards/my_board/mod.rs
use crate::board::BoardConfig;
use crate::peripherals::display::DisplayInterface;

pub struct MyBoard;

impl BoardConfig for MyBoard {
    fn name(&self) -> &str { "my_board" }
    fn display_width(&self) -> u16 { 320 }
    fn display_height(&self) -> u16 { 240 }
    fn has_touch(&self) -> bool { true }
    fn has_imu(&self) -> bool { true }
    fn has_mic(&self) -> bool { true }
    fn has_speaker(&self) -> bool { true }

    fn display_interface(&self) -> DisplayInterface {
        DisplayInterface::Spi {
            sclk: 12, mosi: 11, cs: 10, dc: 9,
            reset: Some(8), backlight: Some(7),
            freq_hz: 40_000_000,
        }
    }
}
```

### Step 4: Add Cargo Feature

```toml
# Cargo.toml
[features]
default = ["waveshare-349"]
waveshare-349 = []
waveshare-349-touch = []
my-board = []           # <-- Add this
custom-board = []
```

### Step 5: Register in `main.rs`

```rust
// src/board/mod.rs — add your board module
#[cfg(feature = "my-board")]
pub mod my_board;

// src/main.rs — use your board
#[cfg(feature = "my-board")]
use board::my_board::MyBoard;

#[entry]
fn main() -> ! {
    #[cfg(feature = "my-board")]
    let board = MyBoard;
    // ...
}
```

### Step 6: Build and Flash

```bash
cargo build --release --features my-board
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3
```

---

## Adding Components

### Peripheral Trait System

Every peripheral implements the base `Peripheral` trait:

```rust
// src/peripherals/mod.rs
pub trait Peripheral {
    /// Initialize the peripheral (configure pins, send init commands).
    fn init(&mut self) -> Result<(), PeripheralError>;

    /// Check if peripheral is ready to use.
    fn is_ready(&self) -> bool;

    /// Get peripheral name (e.g., "display", "touch", "imu").
    fn name(&self) -> &str;

    /// Shutdown / de-init peripheral.
    fn shutdown(&mut self);
}
```

### Category-Specific Traits

Each peripheral type has a domain trait with specific methods:

```rust
// Display
pub trait DisplayPeripheral: Peripheral {
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn set_brightness(&mut self, level: u8) -> Result<(), PeripheralError>;
    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16);
    fn draw_bitmap(&mut self, x: u16, y: u16, data: &[u8]);
}

// Touch
pub trait TouchPeripheral: Peripheral {
    fn read_touch(&mut self) -> Option<TouchEvent>;
    fn is_touched(&self) -> bool;
}

// IMU
pub trait ImuPeripheral: Peripheral {
    fn read_accel(&mut self) -> [f32; 3];
    fn read_gyro(&mut self) -> [f32; 3];
    fn read_temp(&mut self) -> f32;
}
```

### Registering a Peripheral

```rust
use peripherals::{PeripheralRegistry, display::DisplayPeripheral};

let mut registry = PeripheralRegistry::new();

// Create your peripheral instance
let display = MyDisplayDriver::new(spi_pins);
registry.register(Box::new(display));

// Access by name
if let Some(disp) = registry.get_mut("display") {
    // Use peripheral...
}
```

---

## Creating Custom Sensors

### Step 1: Define Your Sensor

```rust
// src/sensors/my_sensor.rs
use crate::sensors::Sensor;

pub struct MyTempSensor {
    i2c_addr: u8,
    last_reading: Option<f32>,
}

impl MyTempSensor {
    pub fn new(i2c_addr: u8) -> Self {
        Self { i2c_addr, last_reading: None }
    }
}

impl Sensor for MyTempSensor {
    fn name(&self) -> &str { "my_temp_sensor" }
    fn sensor_type(&self) -> &str { "temperature" }

    fn init(&mut self) -> Result<(), SensorError> {
        // Send I2C init commands to sensor
        Ok(())
    }

    fn read(&mut self) -> Result<SensorReading, SensorError> {
        // Read raw data from I2C
        let raw = self.read_i2c_register(0x00)?;
        let temp = self.convert_to_celsius(raw);
        self.last_reading = Some(temp);
        Ok(SensorReading {
            name: "temperature",
            value: temp,
            unit: "°C",
            timestamp_ms: self.get_timestamp(),
        })
    }

    fn is_ready(&self) -> bool {
        self.last_reading.is_some()
    }
}
```

### Step 2: Register in Sensor Registry

```rust
// src/sensors/mod.rs
pub mod my_sensor;

// In main.rs or orchestrator:
let mut sensor_reg = SensorRegistry::new();
sensor_reg.register(Box::new(MyTempSensor::new(0x48)));
```

### Step 3: Expose via Protocol

The sensor automatically appears in telemetry:

```json
{
  "type": "telemetry",
  "msg_id": 42,
  "timestamp_ms": 1234567890,
  "data": {
    "sensors": [
      {
        "name": "my_temp_sensor",
        "type": "temperature",
        "value": 23.5,
        "unit": "°C"
      }
    ]
  }
}
```

Query via HTTP:

```bash
curl -H "Authorization: Bearer <token>" http://<device-ip>/api/sensors
```

---

## Security Model

### Overview

The template uses a **pre-shared token (PSK)** model with rate limiting:

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Porpoise   │────▶│  HTTP/WS     │────▶│  Rate       │
│  IDE        │     │  Server      │     │  Limiter    │
│             │◀────│  (auth check)│◀────│  (5 req/10s)│
└─────────────┘     └──────────────┘     └─────────────┘
       │                    │
       │  Bearer <token>    │  Token verify
       │                    │  (SHA-256 constant-time)
       ▼                    ▼
┌─────────────┐     ┌──────────────┐
│  User       │     │  AuthToken   │
│  copies     │     │  (NVS-stored)│
│  from       │     │              │
│  device UI  │     │              │
└─────────────┘     └──────────────┘
```

### How It Works

1. **First Boot**: Device generates a random 32-byte token, stores in NVS
2. **Display Token**: Token shown on device screen (hex-encoded, 64 chars)
3. **User Copies**: User enters token in Porpoise IDE "Add Device" dialog
4. **Every Request**: IDE sends `Authorization: Bearer <token>` header
5. **Server Verifies**: Constant-time SHA-256 comparison (no timing attacks)
6. **Rate Limiting**: Max 5 auth attempts per 10s per IP; 60s lockout on violation

### Security Features

| Feature | Status | Details |
|---------|--------|---------|
| Token Auth | ✅ | SHA-256 constant-time comparison |
| Rate Limiting | ✅ | 5 req/10s per IP, 60s lockout |
| Input Validation | ✅ | Command name whitelist, payload size limits |
| No Hardcoded Passwords | ✅ | WiFi creds must be provided at build time |
| WPA2 Minimum | ✅ | Panics if WiFi password < 8 chars |
| Auth Deadline | ✅ | WS connections closed after 10s without auth |
| Token Rotation | 🔲 | Planned — regenerate token via API |
| TLS/HTTPS | 🔲 | Requires `esp-mbedtls` (see below) |

### Enabling TLS (Optional)

For production deployments, enable TLS:

```toml
# Cargo.toml
[dependencies]
esp-mbedtls = { version = "0.5", features = ["esp32s3"] }
```

```rust
// src/comms/http_server.rs — use TLS listener
use esp_mbedtls::TlsListener;

let tls = TlsListener::new(443, cert, key);
```

> **Note**: TLS requires ~50KB additional flash and increases boot time by ~2s.

### Disabling Auth (Development Only)

```rust
// main.rs — NOT recommended for production
let http = HttpServer::new(80, false);  // false = no auth
let ws = WebSocketServer::new(81, false);
```

---

## WiFi Configuration

### Station Mode (Connect to Existing Network)

```rust
let config = WifiConfig::sta("MyNetwork", "my-secure-password");
let mut wifi = WifiManager::new(config);
wifi.start()?;
```

### Access Point Mode (Device Creates Network)

```rust
let config = WifiConfig::ap("Porpoise-Setup", "setup-password-123");
let mut wifi = WifiManager::new(config);
wifi.start()?;
```

### Auto-Fallback (STA → AP)

If STA connection fails after `max_reconnect_attempts`, device enters AP mode:

```rust
let mut config = WifiConfig::sta("MyNetwork", "password123");
config.max_reconnect_attempts = 5;
// After 5 failures, device creates AP "Porpoise-Setup"
```

### WiFi Security Rules

- **No hardcoded passwords** — `Default` impl removed; must use `WifiConfig::sta()` or `::ap()`
- **WPA2 minimum** — panics at runtime if password < 8 characters
- **Max connections** — AP mode limits concurrent clients (default: 4)
- **Channel selection** — avoid crowded channels; use WiFi analyzer

---

## Remote Control Protocol

### Message Format (JSON over WebSocket/HTTP)

```json
{
  "type": "command",
  "msg_id": 1,
  "timestamp_ms": 1234567890,
  "data": {
    "target": "display",
    "action": "set_brightness",
    "args": { "value": 128 }
  }
}
```

### Available Commands

| Target | Action | Args | Description |
|--------|--------|------|-------------|
| `display` | `set_brightness` | `{ value: 0-255 }` | Set backlight |
| `display` | `fill_rect` | `{ x, y, w, h, color }` | Draw rectangle |
| `touch` | `read` | `{}` | Get touch coordinates |
| `imu` | `read_accel` | `{}` | Read accelerometer |
| `imu` | `read_gyro` | `{}` | Read gyroscope |
| `camera` | `capture` | `{}` | Take photo |
| `camera` | `start_stream` | `{}` | Start video stream |
| `camera` | `stop_stream` | `{}` | Stop video stream |
| `system` | `reboot` | `{}` | Reboot device |
| `system` | `get_info` | `{}` | Device info |

### HTTP API Endpoints

```bash
# Get device info (requires auth)
curl -H "Authorization: Bearer <token>" http://<ip>/api/info

# Get telemetry
curl -H "Authorization: Bearer <token>" http://<ip>/api/telemetry

# Send command
curl -X POST -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"target":"display","action":"set_brightness","args":{"value":200}}' \
  http://<ip>/api/command

# List sensors
curl -H "Authorization: Bearer <token>" http://<ip>/api/sensors

# List peripherals
curl -H "Authorization: Bearer <token>" http://<ip>/api/peripherals
```

### WebSocket Connection

```javascript
// JavaScript example
const ws = new WebSocket('ws://<device-ip>:81');

ws.onopen = () => {
  // Must authenticate within 10 seconds
  ws.send(JSON.stringify({
    type: 'auth',
    token: '<your-64-char-token>'
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'telemetry') {
    console.log('Sensor data:', msg.data);
  }
};
```

---

## OTA Updates

### Via Porpoise IDE

1. Open Porpoise IDE → Device Manager
2. Select your device
3. Click "Upload Firmware"
4. Select `.bin` file
5. Device reboots with new firmware

### Via HTTP API

```bash
# Upload firmware chunk
curl -X POST -H "Authorization: Bearer <token>" \
  -F "firmware=@target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3.bin" \
  http://<ip>/api/ota
```

### OTA Protocol

```json
// Orchestrator → Device
{
  "type": "ota_start",
  "total_size": 524288,
  "checksum": "sha256:abc123..."
}

// Device → Orchestrator
{
  "type": "ota_progress",
  "offset": 0,
  "received": 4096
}

// Orchestrator → Device (chunk)
{
  "type": "ota_chunk",
  "offset": 0,
  "data": "<base64-encoded-bytes>",
  "total_size": 524288,
  "checksum": "sha256:abc123..."
}
```

---

## Troubleshooting

### Build Errors

| Error | Fix |
|-------|-----|
| `can't find crate for core` | Run `rustup target add xtensa-esp32s3-none-elf` |
| `linker `xtensa-esp32s3-elf-gcc` not found` | Install ESP-IDF toolchain |
| `overflowed the stack` | Increase stack size in `memory.x` |
| `PSRAM not found` | Check `sdkconfig` enables PSRAM |

### Runtime Errors

| Error | Fix |
|-------|-----|
| WiFi won't connect | Check SSID/password; try AP mode first |
| Auth token rejected | Regenerate: reboot device; check token copy |
| Rate limited | Wait 60s; reduce request frequency |
| Display blank | Check pin mapping in `config.toml` |
| Touch not working | Verify I2C address with logic analyzer |

### Debug Logging

```bash
# Enable verbose logging
RUST_LOG=debug espflash flash --monitor target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3

# UART output shows:
# === Porpoise ESP32-S3 Template ===
# Board: my_board
# CPU: 240 MHz
# WiFi: Connected (192.168.1.100)
# Auth: enabled (token: a1b2c3d4...)
# HTTP :80 (auth), WebSocket :81 (auth)
```

---

## License

MIT
