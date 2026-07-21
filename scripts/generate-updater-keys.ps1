# Generate Tauri updater signing key
# IMPORTANT: Keep this key secret! It's used to sign update manifests.
# Store the public key in tauri.conf.json as `plugins.updater.pubkey`

# Check if tauri-cli is available
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo is required. Install from https://rustup.rs"
    exit 1
}

# Generate the private key
Write-Host "Generating Tauri updater signing key..." -ForegroundColor Yellow
$keyPath = Join-Path $env:USERPROFILE ".tauri\porpoise-updater.key"
$env:TAURI_UPDATER_PRIVATE_KEY = ""
$env:TAURI_UPDATER_KEY_PASSWORD = ""

cargo tauri signer generate --force -w $keyPath

if ($LASTEXITCODE -ne 0) {
    # Fallback: generate manually
    Write-Host "Tauri CLI not available - manual generation needed" -ForegroundColor Red
    Write-Host "Install tauri-cli and run: cargo tauri signer generate -w ~/.tauri/porpoise-updater.key"
    exit 1
}

Write-Host "`nPrivate key saved to: $keyPath" -ForegroundColor Green
Write-Host "`nIMPORTANT: Set these environment variables in CI:" -ForegroundColor Yellow
Write-Host "  TAURI_UPDATER_PRIVATE_KEY: contents of $keyPath"
Write-Host "  TAURI_UPDATER_KEY_PASSWORD: <your-password>"
Write-Host "`nExtract the public key and add to tauri.conf.json:" -ForegroundColor Yellow
Write-Host "  cargo tauri signer export -k $keyPath"
