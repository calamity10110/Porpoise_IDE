# Porpoise development commands
# Usage: just <command>

# Build all crates
build:
    cargo build --workspace

# Build release
build-release:
    cargo build --release --workspace

# Run all tests
test:
    cargo test --workspace

# Run clippy lints
clippy:
    cargo clippy --all-targets -- -D warnings

# Check formatting
fmt:
    cargo fmt --check

# Fix formatting
fmt-fix:
    cargo fmt

# Build docs
doc:
    cargo doc --no-deps

# Full CI check (build + test + clippy + fmt)
check: build test clippy fmt

# Run a specific crate
run *args:
    cargo run -p {{args}}

# Build specific crate
build-p *crate:
    cargo build -p {{crate}}

# Run tests for specific crate
test-p *crate:
    cargo test -p {{crate}}

# Clean build artifacts
clean:
    cargo clean

# Audit dependencies
audit:
    cargo audit

# Watch mode for development
watch:
    cargo watch -x check

# Generate knowledge graph
graphify:
    /graphify .

# Show outdated dependencies
outdated:
    cargo outdated

default:
    @just --list
