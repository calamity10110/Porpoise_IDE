default: build

# Build everything
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --release --workspace

# Run all tests
test:
    cargo test --workspace

# Lint
clippy:
    cargo clippy --workspace -- -D warnings

# Format
fmt:
    cargo fmt --check

fix:
    cargo fmt
    cargo clippy --fix --allow-dirty

# Build documentation
doc:
    cargo doc --no-deps

# Run the CLI
cli *args:
    cargo run --release -p porpoise-cli -- {{args}}

# Run the server
server:
    cargo run --release -p porpoise-server

# Clean build artifacts
clean:
    cargo clean
