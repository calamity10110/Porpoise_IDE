# Porpoise development commands
# Usage: just <command>

# === Setup ===
setup: install-deps setup-hooks
    @echo "Setup complete!"

install-deps:
    cargo install cargo-watch cargo-audit cargo-llvm-cov

setup-hooks:
    ln -sf ../../pre-commit.sh .git/hooks/pre-commit

# === Build ===
build: build-workspace
build-workspace:
    cargo build --workspace
build-release:
    cargo build --release --workspace
build-cli:
    cargo build --release -p porpoise-cli
build-server:
    cargo build --release -p porpoise-server
build-app:
    cargo tauri build --verbose
build-esp32 BOARD="waveshare_lcd_349":
    cd templates/esp32-s3 && ./build.sh build {{BOARD}}

# === Test ===
test: test-workspace
test-workspace:
    cargo test --workspace
test-p *crate:
    cargo test -p {{crate}}
test-coverage:
    cargo llvm-cov --workspace --lcov --output-path lcov.info

# === Quality ===
clippy:
    cargo clippy --all-targets -- -D warnings
fmt:
    cargo fmt --check
fmt-fix:
    cargo fmt
audit:
    cargo audit
check: build test clippy fmt

# === Dev ===
dev:
    cargo watch -x check
dev-cli:
    cargo watch -x 'run -p porpoise-cli'
dev-server:
    cargo watch -x 'run -p porpoise-server'

# === Docker ===
docker-build:
    docker build -t porpoise .
docker-run:
    docker run -d -p 9876:9876 -v ~/.config/porpoise:/root/.config/porpoise porpoise

# === Release ===
release version:
    #!/usr/bin/env bash
    set -e
    echo "Releasing v{{version}}..."
    sed -i "s/^version = .*/version = \"{{version}}\"/" Cargo.toml
    cargo build --release --workspace
    git add -A
    git commit -m "release: v{{version}}"
    git tag "v{{version}}"
    git push origin main --tags

# === Install ===
install: build-release
    cp target/release/porpoise /usr/local/bin/
    cp target/release/porpoise-server /usr/local/bin/

# === Clean ===
clean:
    cargo clean
clean-all: clean
    rm -rf target/ mobile/build/ graphify-out/

# === Info ===
default:
    @just --list
