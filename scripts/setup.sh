#!/bin/bash
# Porpoise first-time dev setup
set -e

echo "=== Porpoise Dev Setup ==="

# Check Rust
if ! command -v rustup &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install cargo tools
echo "Installing cargo tools..."
cargo install cargo-watch cargo-audit cargo-llvm-cov

# Setup pre-commit hook
echo "Setting up pre-commit hook..."
ln -sf ../../pre-commit.sh .git/hooks/pre-commit

# Build workspace
echo "Building workspace..."
cargo build --workspace

echo "=== Setup complete! ==="
echo "Run 'just' to see available commands."
