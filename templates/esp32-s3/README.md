# Porpoise ESP32-S3 Template

A `no_std` Rust template for ESP32-S3 boards with modular peripherals, remote control via WiFi, and integration with the Porpoise IDE.

## Features

- **Modular Peripherals**: Display, Touch, Audio (mic/speaker), Camera, IMU, Mouse, Keyboard, Custom
- **Board Config**: TOML-based pin mapping — edit one file for your hardware
- **Remote Control**: HTTP REST + WebSocket for live telemetry and command dispatch
- **OTA Updates**: Over-the-air firmware updates via WebSocket
- **Sensor Fusion**: IMU orientation filter, touch gesture recognition
- **Porpoise IDE Integration**: Device auto-discovery, live dashboard, remote orchestration

## Quick Start

### Prerequisites

```bash
# Install Rust + Xtensa target
rustup target add xtensa-esp32s3-none-elf
cargo install espflash

# Or use espup
espup install
```

### Build & Flash

```bash
cd templates/esp32-s3

# Build for Waveshare LCD 349 (default)
cargo build --release

# Flash
espflash flash target/xtensa-esp32s3-none-elf/release/porpoise-esp32s3

# Monitor serial output
espflash monitor
```

### Use a Different Board

1. Copy `boards/waveshare_lcd_349/` to `boards/your_board/`
2. Edit `boards/your_board/config.toml` with your pin mapping
3. Edit `boards/your_board/mod.rs` with your board struct
4. Update `Cargo.toml` features to select your board
5. Update `src/main.rs` to import your board

## Project Structure

```
esp32-s3/
├── .cargo/config.toml          # Xtensa build target
├── Cargo.toml                   # Dependencies + features
├── build.rs                     # Linker script setup
├── boards/
│   └── waveshare_lcd_349/
│       ├── config.toml          # Pin mapping (edit this!)
│       └── mod.rs               # Board struct
├── src/
│   ├── main.rs                  # Entry point + boot sequence
│   ├── lib.rs                   # Module declarations
│   ├── board/                   # Board config trait
│   ├── peripherals/             # Peripheral trait system
│   │   ├── mod.rs               # PeripheralRegistry
│   │   ├── display/             # Display trait + GC9A01
│   │   ├── touch/               # Touch trait + CST816S
│   │   ├── audio/               # Mic + Speaker traits
│   │   ├── camera/              # Camera trait
│   │   ├── imu/                 # IMU trait
│   │   ├── input/               # Mouse + Keyboard traits
│   │   └── custom/              # User-defined peripherals
│   ├── comms/                   # Communication layer
│   │   ├── wifi.rs              # WiFi STA/AP manager
│   │   ├── http_server.rs       # REST API server
│   │   ├── websocket.rs         # WebSocket for live control
│   │   └── protocol.rs          # Wire format definitions
│   ├── orchestration/           # Remote control logic
│   │   ├── command_dispatcher.rs # Routes commands to peripherals
│   │   ├── telemetry.rs         # System health reporting
│   │   └── ota.rs               # OTA update handler
│   └── sensors/                 # Higher-level sensor processing
│       ├── orientation.rs       # IMU complementary filter
│       └── gesture.rs           # Touch gesture recognition
└── README.md
```

## Peripheral Trait System

Each peripheral implements a common trait. Users enable/disable peripherals in `config.toml`:

| Trait | Description | Example Drivers |
|-------|-------------|-----------------|
| `DisplayPeripheral` | Pixel framebuffer, brightness | GC9A01, ST7789, ILI9341 |
| `TouchPeripheral` | Touch events, calibration | CST816S, FT6336, GT911 |
| `MicrophonePeripheral` | Audio capture, streaming | I2S ADC, SPH0645 |
| `SpeakerPeripheral` | Audio playback, volume | I2S DAC, MAX98357A |
| `CameraPeripheral` | Frame capture, streaming | OV2640, OV5640 |
| `ImuPeripheral` | Accel/gyro/mag readings | QMI8658, BMI270, MPU6050 |
| `MousePeripheral` | Relative movement, buttons | USB HID, optical sensor |
| `KeyboardPeripheral` | Key events, LEDs | USB HID, matrix scan |
| `CustomPeripheral` | User-defined commands | Anything! |

## Remote Control Protocol

### HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/info` | Device info + capabilities |
| GET | `/api/telemetry` | Current system telemetry |
| POST | `/api/command` | Send command to peripheral |
| GET | `/api/peripherals` | List all peripherals |
| POST | `/api/ota` | Upload firmware binary |

### WebSocket Messages

Connect to `ws://<device-ip>:81` for real-time control:

```json
{"msg_id": 1, "msg_type": "Command", "payload": {"target": "display", "action": "set_brightness", "args": {"value": 128}}}
```

## Adding a New Board

See `boards/waveshare_lcd_349/` for reference. Key steps:

1. Create `boards/your_board/config.toml` with pin assignments
2. Create `boards/your_board/mod.rs` implementing `BoardConfig`
3. Add a Cargo feature in `Cargo.toml`
4. Import your board in `main.rs`

## License

MIT
