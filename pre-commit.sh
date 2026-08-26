#!/bin/sh
# Porpoise pre-commit hook: staged-only checks with WIP skip
# Install: ln -sf ../../pre-commit.sh .git/hooks/pre-commit

set -e
echo "=== Porpoise pre-commit ==="

# Only check staged .rs files
STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)
if [ -z "$STAGED_RS" ]; then
    echo "No Rust files staged. Skipping checks."
    exit 0
fi

# Format check (fast)
cargo fmt --check || { echo "Run 'cargo fmt' first."; exit 1; }

# Clippy on workspace (medium)
cargo clippy --all-targets -- -D warnings || { echo "Fix clippy warnings."; exit 1; }

# Tests (slow — skip on WIP commits)
if git log -1 --pretty=%B | grep -qi '\[wip\]'; then
    echo "WIP commit — skipping tests."
else
    cargo test --workspace || { echo "Tests failed."; exit 1; }
fi

echo "=== All checks passed ==="
