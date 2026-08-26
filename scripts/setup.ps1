# Porpoise first-time dev setup (Windows)
$ErrorActionPreference = "Stop"

Write-Host "=== Porpoise Dev Setup ===" -ForegroundColor Green

# Check Rust
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

# Install cargo tools
Write-Host "Installing cargo tools..."
cargo install cargo-watch cargo-audit cargo-llvm-cov

# Build workspace
Write-Host "Building workspace..."
cargo build --workspace

Write-Host "=== Setup complete! ===" -ForegroundColor Green
Write-Host "Run 'just' to see available commands."
