# =============================================================================
# Porpoise ESP32-S3 — Makefile (cross-platform: Linux, WSL, macOS)
# =============================================================================
# Usage:
#   make build BOARD=waveshare-349     Build firmware
#   make flash BOARD=waveshare-349     Build + flash
#   make monitor                       Serial monitor
#   make clean                         Clean artifacts
#   make ota BOARD=waveshare-349       Build OTA binary
#   make behavior FILE=config.json     Behavior config instructions
#   make list                          List boards
#   make help                          Show help
# =============================================================================

.DEFAULT_GOAL := help
SHELL := /bin/bash

# --- Config ---
BOARD       ?= waveshare-349
PORT        ?=
BAUD        ?= 115200
TARGET      := xtensa-esp32s3-none-elf
PART_TABLE  := partitions.csv
CARGO_FLAGS := --release --target $(TARGET) -Z build-std=core,alloc

# --- Board → Feature mapping ---
ifeq ($(BOARD),waveshare-349)
  FEATURE := waveshare-349
else ifeq ($(BOARD),waveshare_lcd_349)
  FEATURE := waveshare-349
else ifeq ($(BOARD),waveshare-349-touch)
  FEATURE := waveshare-349-touch
else ifeq ($(BOARD),waveshare_lcd_349_touch)
  FEATURE := waveshare-349-touch
else ifeq ($(BOARD),custom)
  FEATURE := custom-board
else ifeq ($(BOARD),custom-board)
  FEATURE := custom-board
else
  FEATURE := $(BOARD)
endif

# --- Port detection ---
ifdef PORT
  PORT_FLAG := --port $(PORT)
else
  PORT_FLAG :=
endif

# --- Targets ---
.PHONY: build flash monitor clean ota behavior list help prereqs

prereqs:
	@command -v rustup   >/dev/null 2>&1 || { echo "[ERROR] rustup not found"; exit 1; }
	@command -v cargo    >/dev/null 2>&1 || { echo "[ERROR] cargo not found"; exit 1; }
	@command -v espflash >/dev/null 2>&1 || { echo "[ERROR] espflash not found"; exit 1; }
	@rustup target list --installed | grep -q $(TARGET) || rustup target add $(TARGET)

build: prereqs
	@echo "[INFO] Building for board=$(BOARD) feature=$(FEATURE)"
	cargo build $(CARGO_FLAGS) --features $(FEATURE)
	@echo "[INFO] Build complete: target/$(TARGET)/release/porpoise-esp32s3"

flash: prereqs
	@echo "[INFO] Building + flashing for board=$(BOARD)"
	cargo build $(CARGO_FLAGS) --features $(FEATURE)
	espflash flash $(PORT_FLAG) --partition-table $(PART_TABLE) \
		target/$(TARGET)/release/porpoise-esp32s3
	@echo "[INFO] Flash complete!"

monitor:
	@echo "[INFO] Opening serial monitor (baud=$(BAUD))..."
	espflash monitor $(PORT_FLAG) target/$(TARGET)/release/porpoise-esp32s3

clean:
	@echo "[INFO] Cleaning build artifacts..."
	cargo clean
	@echo "[INFO] Clean complete."

ota: prereqs
	@echo "[INFO] Building OTA binary for board=$(BOARD)"
	cargo build $(CARGO_FLAGS) --features $(FEATURE)
	@cp target/$(TARGET)/release/porpoise-esp32s3 \
	   target/$(TARGET)/release/porpoise-esp32s3-ota.bin
	@echo "[INFO] OTA binary: target/$(TARGET)/release/porpoise-esp32s3-ota.bin"
	@echo "[INFO] Upload via WebSocket: ws://<device-ip>:81"

behavior:
	@if [ -z "$(FILE)" ]; then \
		echo "[INFO] Usage: make behavior FILE=config.json"; \
		echo "Example config.json:"; \
		echo '  {"components":{"imu":{"enabled":true,"sample_rate_hz":100}}}'; \
	else \
		echo "[INFO] Behavior config: $(FILE)"; \
		echo "[INFO] Send via WebSocket: target=system, action=update_behavior"; \
	fi

list:
	@echo "Available boards:"
	@echo "  waveshare-349        Waveshare ESP32-S3 Touch LCD 1.28\" (default)"
	@echo "  waveshare-349-touch  Waveshare ESP32-S3 Touch LCD 3.49\""
	@echo "  custom-board         Custom ESP32-S3 board"
	@echo ""
	@echo "Set: make build BOARD=waveshare-349"

help:
	@echo "Porpoise ESP32-S3 — Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  build [BOARD=]     Build firmware"
	@echo "  flash [BOARD=]     Build + flash"
	@echo "  monitor            Serial monitor"
	@echo "  clean              Clean artifacts"
	@echo "  ota [BOARD=]       Build OTA binary"
	@echo "  behavior FILE=     Behavior config instructions"
	@echo "  list               List boards"
	@echo "  help               This message"
	@echo ""
	@echo "Variables:"
	@echo "  BOARD=waveshare-349|waveshare-349-touch|custom-board"
	@echo "  PORT=/dev/ttyUSB0  Serial port (auto-detect if unset)"
	@echo "  BAUD=115200        Monitor baud rate"
