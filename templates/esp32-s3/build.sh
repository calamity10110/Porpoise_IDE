#!/usr/bin/env bash
# =============================================================================
# Porpoise ESP32-S3 — Build Script (Linux / WSL)
# =============================================================================
# Usage:
#   ./build.sh build [board]       Build firmware (default: waveshare-349)
#   ./build.sh flash [board]       Build + flash to device
#   ./build.sh monitor             Flash + open serial monitor
#   ./build.sh clean               Clean build artifacts
#   ./build.sh ota [board]         Build OTA-updateable binary
#   ./build.sh behavior <json>     Update behavior config over serial
#   ./build.sh list                List available boards
#   ./build.sh help                Show this help
#
# Environment:
#   ESP_BOARD    — override board selection (e.g., ESP_BOARD=custom-board)
#   ESP_PORT     — serial port (e.g., /dev/ttyUSB0, auto-detected if unset)
#   ESP_BAUD     — monitor baud rate (default: 115200)
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# --- Defaults ---
BOARD="${ESP_BOARD:-waveshare-349}"
PORT="${ESP_PORT:-}"
BAUD="${ESP_BAUD:-115200}"
TARGET="xtensa-esp32s3-none-elf"
PARTITION_TABLE="partitions.csv"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# --- Board feature mapping ---
board_to_feature() {
    case "$1" in
        waveshare-349|waveshare_lcd_349)       echo "waveshare-349" ;;
        waveshare-349-touch|waveshare_lcd_349_touch) echo "waveshare-349-touch" ;;
        custom|custom-board)                   echo "custom-board" ;;
        *)                                     echo "$1" ;;
    esac
}

# --- Prereq check ---
check_prereqs() {
    local missing=()
    command -v rustup   &>/dev/null || missing+=("rustup")
    command -v cargo    &>/dev/null || missing+=("cargo")
    command -v espflash &>/dev/null || missing+=("espflash")

    if ! rustup target list --installed | grep -q "$TARGET"; then
        warn "Target $TARGET not installed. Adding..."
        rustup target add "$TARGET"
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        error "Missing tools: ${missing[*]}"
        echo "Install with:"
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "  rustup target add $TARGET"
        echo "  cargo install espflash"
        exit 1
    fi
}

# --- Auto-detect serial port ---
detect_port() {
    if [ -n "$PORT" ]; then
        echo "$PORT"
        return
    fi
    for p in /dev/ttyUSB* /dev/ttyACM* /dev/cu.usbserial* /dev/cu.SLAB_USBtoUART*; do
        if [ -e "$p" ]; then
            echo "$p"
            return
        fi
    done
    echo ""
}

# --- Commands ---
cmd_build() {
    local feature
    feature="$(board_to_feature "$BOARD")"
    info "Building for board: $BOARD (feature: $feature)"
    cargo build --release --features "$feature" --target "$TARGET" \
        -Z build-std=core,alloc
    info "Build complete: target/$TARGET/release/porpoise-esp32s3"
}

cmd_flash() {
    local feature port_path
    feature="$(board_to_feature "$BOARD")"
    port_path="$(detect_port)"
    info "Building + flashing for board: $BOARD"
    local port_arg=""
    if [ -n "$port_path" ]; then
        port_arg="--port $port_path"
        info "Serial port: $port_path"
    else
        warn "No serial port detected — espflash will auto-detect"
    fi
    cargo build --release --features "$feature" --target "$TARGET" \
        -Z build-std=core,alloc
    espflash flash $port_arg \
        --partition-table "$PARTITION_TABLE" \
        "target/$TARGET/release/porpoise-esp32s3"
    info "Flash complete!"
}

cmd_monitor() {
    local port_path
    port_path="$(detect_port)"
    local port_arg=""
    if [ -n "$port_path" ]; then
        port_arg="--port $port_path"
    fi
    info "Opening serial monitor (baud: $BAUD)..."
    espflash monitor $port_arg "target/$TARGET/release/porpoise-esp32s3" || \
        serial-monitor "${port_path:-/dev/ttyUSB0}" "$BAUD"
}

cmd_clean() {
    info "Cleaning build artifacts..."
    cargo clean
    info "Clean complete."
}

cmd_ota() {
    local feature
    feature="$(board_to_feature "$BOARD")"
    info "Building OTA binary for board: $BOARD"
    cargo build --release --features "$feature" --target "$TARGET" \
        -Z build-std=core,alloc
    local bin="target/$TARGET/release/porpoise-esp32s3"
    local ota_bin="target/$TARGET/release/porpoise-esp32s3-ota.bin"
    if [ -f "$bin" ]; then
        cp "$bin" "$ota_bin"
        info "OTA binary: $ota_bin"
        info "Upload via WebSocket: ws://<device-ip>:81 (command target=system, action=ota_begin)"
        info "Or flash OTA slot: espflash flash --partition-table $PARTITION_TABLE --ota $ota_bin"
    fi
}

cmd_behavior() {
    local json_file="$1"
    if [ -z "$json_file" ] || [ ! -f "$json_file" ]; then
        error "Usage: ./build.sh behavior <config.json>"
        echo "Example config.json:"
        echo '  {'
        echo '    "components": {'
        echo '      "imu": { "enabled": true, "sample_rate_hz": 100 },'
        echo '      "display": { "brightness": 80, "rotation": 0 }'
        echo '    }'
        echo '  }'
        exit 1
    fi
    info "Behavior config: $json_file"
    info "Send via WebSocket to update post-flash:"
    echo "  python3 -c \""
    echo "import websocket, json"
    echo "ws = websocket.create_connection('ws://<device-ip>:81')"
    echo "ws.send(json.dumps({"
    echo "  'type': 'command', 'target': 'system',"
    echo "  'action': 'update_behavior',"
    echo "  'params': $(cat "$json_file")"
    echo "}))\""
}

cmd_list() {
    info "Available boards:"
    echo "  waveshare-349        Waveshare ESP32-S3 Touch LCD 1.28\" (default)"
    echo "  waveshare-349-touch  Waveshare ESP32-S3 Touch LCD 3.49\""
    echo "  custom-board         Custom ESP32-S3 board"
    echo ""
    info "Set board: ESP_BOARD=<name> ./build.sh build"
    info "Or edit:   .cargo/config.toml → default features"
}

cmd_help() {
    head -15 "$0" | tail -13
}

# --- Main ---
case "${1:-help}" in
    build)      check_prereqs; cmd_build ;;
    flash)      check_prereqs; cmd_flash ;;
    monitor)    check_prereqs; cmd_monitor ;;
    clean)      cmd_clean ;;
    ota)        check_prereqs; cmd_ota ;;
    behavior)   cmd_behavior "${2:-}" ;;
    list)       cmd_list ;;
    help|*)     cmd_help ;;
esac
