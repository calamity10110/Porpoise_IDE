#!/bin/bash
# Install Porpoise binaries to /usr/local/bin
set -e

echo "=== Installing Porpoise ==="

# Build release
cargo build --release --workspace

# Copy binaries
echo "Installing binaries..."
sudo cp target/release/porpoise /usr/local/bin/
sudo cp target/release/porpoise-server /usr/local/bin/

echo "=== Installation complete! ==="
echo "Run 'porpoise --help' to get started."
