# =============================================================================
# Porpoise ESP32-S3 — Build Script (Windows PowerShell)
# =============================================================================
# Usage:
#   .\build.ps1 build [board]       Build firmware (default: waveshare-349)
#   .\build.ps1 flash [board]       Build + flash to device
#   .\build.ps1 monitor             Flash + open serial monitor
#   .\build.ps1 clean               Clean build artifacts
#   .\build.ps1 ota [board]         Build OTA-updateable binary
#   .\build.ps1 behavior <json>     Update behavior config over serial
#   .\build.ps1 list                List available boards
#   .\build.ps1 help                Show this help
#
# Environment:
#   $env:ESP_BOARD  — override board selection
#   $env:ESP_PORT   — serial port (e.g., COM3, auto-detected if unset)
#   $env:ESP_BAUD   — monitor baud rate (default: 115200)
# =============================================================================

param(
    [Parameter(Position=0)] [string]$Command = "help",
    [Parameter(Position=1)] [string]$Arg = ""
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

# --- Defaults ---
$BOARD       = if ($env:ESP_BOARD) { $env:ESP_BOARD } else { "waveshare-349" }
$PORT        = if ($env:ESP_PORT)  { $env:ESP_PORT  } else { "" }
$BAUD        = if ($env:ESP_BAUD)  { $env:ESP_BAUD  } else { "115200" }
$TARGET      = "xtensa-esp32s3-none-elf"
$PART_TABLE  = "partitions.csv"

# --- Helpers ---
function Write-Info  { param($msg) Write-Host "[INFO]  $msg" -ForegroundColor Green }
function Write-Warn  { param($msg) Write-Host "[WARN]  $msg" -ForegroundColor Yellow }
function Write-Err   { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }

function Get-BoardFeature {
    param($board)
    switch -Regex ($board) {
        "waveshare.349.touch|waveshare_lcd_349_touch" { return "waveshare-349-touch" }
        "waveshare.349|waveshare_lcd_349"             { return "waveshare-349" }
        "custom"                                       { return "custom-board" }
        default                                        { return $board }
    }
}

function Test-Prereqs {
    $missing = @()
    if (-not (Get-Command rustup   -ErrorAction SilentlyContinue)) { $missing += "rustup" }
    if (-not (Get-Command cargo    -ErrorAction SilentlyContinue)) { $missing += "cargo" }
    if (-not (Get-Command espflash -ErrorAction SilentlyContinue)) { $missing += "espflash" }

    $installed = rustup target list --installed 2>$null
    if ($installed -notmatch [regex]::Escape($TARGET)) {
        Write-Warn "Target $TARGET not installed. Adding..."
        rustup target add $TARGET
    }

    if ($missing.Count -gt 0) {
        Write-Err "Missing tools: $($missing -join ', ')"
        Write-Host "Install with:"
        Write-Host "  winget install Rustlang.Rustup"
        Write-Host "  rustup target add $TARGET"
        Write-Host "  cargo install espflash"
        exit 1
    }
}

function Get-SerialPort {
    if ($PORT) { return $PORT }
    $ports = [System.IO.Ports.SerialPort]::GetPortNames()
    if ($ports.Count -gt 0) { return $ports[0] }
    # Fallback: common ESP32 ports
    foreach ($p in @("COM3","COM4","COM5","COM6","COM7","COM8")) {
        try {
            $sp = New-Object System.IO.Ports.SerialPort($p)
            $sp.Open(); $sp.Close()
            return $p
        } catch { }
    }
    return ""
}

# --- Commands ---
function Invoke-Build {
    param($board)
    $feature = Get-BoardFeature $board
    Write-Info "Building for board: $board (feature: $feature)"
    cargo build --release --features $feature --target $TARGET -Z build-std=core,alloc
    Write-Info "Build complete: target/$TARGET/release/porpoise-esp32s3.exe"
}

function Invoke-Flash {
    param($board)
    $feature = Get-BoardFeature $board
    $port = Get-SerialPort
    Write-Info "Building + flashing for board: $board"
    $portArg = @()
    if ($port) {
        Write-Info "Serial port: $port"
        $portArg = @("--port", $port)
    } else {
        Write-Warn "No serial port detected — espflash will auto-detect"
    }
    cargo build --release --features $feature --target $TARGET -Z build-std=core,alloc
    espflash flash @portArg --partition-table $PART_TABLE "target/$TARGET/release/porpoise-esp32s3"
    Write-Info "Flash complete!"
}

function Invoke-Monitor {
    $port = Get-SerialPort
    $portArg = @()
    if ($port) { $portArg = @("--port", $port) }
    Write-Info "Opening serial monitor (baud: $BAUD)..."
    espflash monitor @portArg "target/$TARGET/release/porpoise-esp32s3"
}

function Invoke-Clean {
    Write-Info "Cleaning build artifacts..."
    cargo clean
    Write-Info "Clean complete."
}

function Invoke-Ota {
    param($board)
    $feature = Get-BoardFeature $board
    Write-Info "Building OTA binary for board: $board"
    cargo build --release --features $feature --target $TARGET -Z build-std=core,alloc
    $bin = "target/$TARGET/release/porpoise-esp32s3"
    $otaBin = "target/$TARGET/release/porpoise-esp32s3-ota.bin"
    if (Test-Path $bin) {
        Copy-Item $bin $otaBin -Force
        Write-Info "OTA binary: $otaBin"
        Write-Info "Upload via WebSocket: ws://<device-ip>:81"
    }
}

function Invoke-Behavior {
    param($jsonFile)
    if (-not $jsonFile -or -not (Test-Path $jsonFile)) {
        Write-Err "Usage: .\build.ps1 behavior <config.json>"
        Write-Host @'
Example config.json:
  {
    "components": {
      "imu": { "enabled": true, "sample_rate_hz": 100 },
      "display": { "brightness": 80, "rotation": 0 }
    }
  }
'@
        exit 1
    }
    Write-Info "Behavior config: $jsonFile"
    Write-Info "Send via WebSocket command (target=system, action=update_behavior)"
}

function Show-List {
    Write-Info "Available boards:"
    Write-Host "  waveshare-349        Waveshare ESP32-S3 Touch LCD 1.28`" (default)"
    Write-Host "  waveshare-349-touch  Waveshare ESP32-S3 Touch LCD 3.49`""
    Write-Host "  custom-board         Custom ESP32-S3 board"
    Write-Host ""
    Write-Info "Set board: `$env:ESP_BOARD='waveshare-349'; .\build.ps1 build"
}

function Show-Help {
    Write-Host @"
Porpoise ESP32-S3 Build Script (Windows)

Commands:
  build [board]       Build firmware (default: waveshare-349)
  flash [board]       Build + flash to device
  monitor             Open serial monitor
  clean               Clean build artifacts
  ota [board]         Build OTA-updateable binary
  behavior <json>     Show behavior config update instructions
  list                List available boards
  help                Show this help

Environment:
  `$env:ESP_BOARD     Board selection (waveshare-349, waveshare-349-touch, custom-board)
  `$env:ESP_PORT      Serial port (e.g., COM3)
  `$env:ESP_BAUD      Monitor baud rate (default: 115200)
"@
}

# --- Main ---
switch ($Command) {
    "build"     { Test-Prereqs; Invoke-Build $BOARD }
    "flash"     { Test-Prereqs; Invoke-Flash $BOARD }
    "monitor"   { Test-Prereqs; Invoke-Monitor }
    "clean"     { Invoke-Clean }
    "ota"       { Test-Prereqs; Invoke-Ota $BOARD }
    "behavior"  { Invoke-Behavior $Arg }
    "list"      { Show-List }
    "help"      { Show-Help }
    default     { Show-Help }
}
