#!/bin/sh
# Porpoise pre-commit hook: run clippy + fmt + test on staged Rust files
# Install: ln -s ../../pre-commit.sh .git/hooks/pre-commit

set -e

echo "=== Porpoise pre-commit check ==="

# Check formatting
cargo fmt --check 2>/dev/null || {
    echo "Formatting issues found. Run 'cargo fmt' to fix."
    exit 1
}

# Run clippy on workspace
cargo clippy --all-targets -- -D warnings 2>/dev/null || {
    echo "Clippy warnings found. Fix them before committing."
    exit 1
}

# Run tests
cargo test --workspace 2>/dev/null || {
    echo "Tests failed. Fix them before committing."
    exit 1
}

echo "=== All checks passed ==="
