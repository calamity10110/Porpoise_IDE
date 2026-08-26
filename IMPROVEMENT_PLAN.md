# Porpoise IDE — Improvement Plan

> Based on project analysis + proven Rust workspace patterns (ripgrep, rust-analyzer, Tauri, Embassy)

---

## Current State Assessment

| Area | Status | Score |
|------|--------|-------|
| Build system | justfile (68 lines, basic) | ⭐⭐ |
| Install docs | docs/INSTALL.md (330 lines) + platform variants | ⭐⭐⭐⭐ |
| CI/CD | 3 workflows (ci, release, android) | ⭐⭐⭐ |
| .gitignore | 31 lines, missing patterns | ⭐⭐ |
| Scripts | 2 PS1 scripts (dev cert, updater keys) | ⭐⭐ |
| Pre-commit | pre-commit.sh (27 lines) | ⭐⭐ |
| Cross-compilation | .cargo/config.toml (ARM targets) | ⭐⭐⭐ |
| Release profile | LTO + strip + codegen-units=1 | ⭐⭐⭐⭐ |

---

## Priority 1: Build System (justfile overhaul)

**Pattern**: ripgrep, rust-analyzer, Tauri all use `just` with rich task graphs.

### Current gaps:
- No `install` target (copy binaries to PATH)
- No `dev` target (watch + rebuild)
- No `docker` targets
- No `release` target with version bump
- No `deploy` targets
- No ESP32-S3 cross targets
- No `setup` target for first-time contributors

### Proposed justfile (expanded):

```just
# === Setup ===
setup: install-deps setup-hooks
install-deps:
    cargo install cargo-watch cargo-audit cargo-llvm-cov tauri-cli
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
```

---

## Priority 2: .gitignore (expand)

**Pattern**: ripgrep, rust-analyzer use comprehensive .gitignore.

### Current gaps:
- Missing `*.bak`, `*.tmp`, `*.old`, `*.orig`, `*.swp`
- Missing `*.log`
- Missing `node_modules/`
- Missing `.env`, `.env.local`
- Missing `*.pem`, `*.key` (secrets)
- Missing `*.firmware`, `*.ota.bin`
- Missing `managed_components/` (ESP-IDF)
- Has trailing whitespace on line 29-31

### Proposed .gitignore:

```gitignore
# === Rust / Cargo ===
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# === IDE / Editor ===
.vscode/
.idea/
*.swp
*.swo
*~
\#*\#
.#*

# === OS ===
Thumbs.db
Desktop.ini
.DS_Store

# === Build artifacts ===
*.firmware
*.ota.bin
*.log

# === Secrets ===
*.pem
*.key
*.pfx
.env
.env.local

# === Node (if any JS tooling) ===
node_modules/

# === ESP-IDF ===
.esp-idf/
managed_components/
dependencies.lock
sdkconfig.old

# === Graphify ===
**/graphify-out/cache/
**/graphify-out/.graphify_*.json
**/graphify-out/.needs_update
**/graphify-out/graph.html
**/graphify-out/graph.json
**/graphify-out/GRAPH_REPORT.md
crates/graphify-out/

# === Sisyphus ===
.sisyphus/

# === Flutter mobile ===
mobile/android/
mobile/ios/
mobile/web/
mobile/linux/
mobile/macos/
mobile/windows/
mobile/build/
mobile/.dart_tool/
mobile/.packages
mobile/.flutter-plugins
mobile/.flutter-plugins-dependencies

# === Chrome extension ===
extension/chrome/*.zip
extension/chrome/.DS_Store

# === Temp ===
temp/
```

---

## Priority 3: CI/CD Improvements

**Pattern**: Tauri, rust-analyzer use matrix builds + caching + coverage.

### Current gaps:
- No `cargo-deny` (license + advisory checking)
- No `cargo-msrv` (MSRV verification)
- No release signing for Linux/macOS
- No changelog generation
- No dependency caching for Flutter/Android

### Proposed additions:

#### ci.yml additions:
```yaml
  msrv:
    name: MSRV Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.85.0
      - run: cargo check --workspace

  deny:
    name: Deny (licenses + advisories)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```

#### release.yml additions:
```yaml
  linux:
    name: Linux (AppImage)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
      - run: cargo install tauri-cli --version "^2.0"
      - run: cargo tauri build --verbose
        working-directory: crates/porpoise-app
      - uses: actions/upload-artifact@v4
        with:
          name: porpoise-linux-appimage
          path: crates/porpoise-app/target/release/bundle/appimage/*.AppImage

  macos:
    name: macOS (DMG)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install tauri-cli --version "^2.0"
      - run: cargo tauri build --verbose
        working-directory: crates/porpoise-app
      - uses: actions/upload-artifact@v4
        with:
          name: porpoise-macos-dmg
          path: crates/porpoise-app/target/release/bundle/dmg/*.dmg
```

---

## Priority 4: Pre-commit Hook (upgrade)

**Pattern**: rust-analyzer uses `pre-commit` framework with multiple hooks.

### Current gaps:
- Runs full test suite on every commit (slow)
- No staged-file-only checking
- No `cargo deny` check

### Proposed pre-commit.sh:

```bash
#!/bin/sh
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
if git log -1 --pretty=%B | grep -q '\[wip\]'; then
    echo "WIP commit — skipping tests."
else
    cargo test --workspace || { echo "Tests failed."; exit 1; }
fi

echo "=== All checks passed ==="
```

---

## Priority 5: Scripts (expand)

**Pattern**: Tauri, ripgrep have scripts/ with setup, build, release helpers.

### Current:
- `scripts/generate-dev-cert.ps1` (1050 bytes)
- `scripts/generate-updater-keys.ps1` (1351 bytes)

### Proposed additions:

| Script | Purpose |
|--------|---------|
| `scripts/setup.sh` | First-time dev setup (install deps, hooks, toolchain) |
| `scripts/setup.ps1` | Windows equivalent |
| `scripts/bump-version.sh` | Bump version in all Cargo.toml files |
| `scripts/changelog.sh` | Generate changelog from git log |
| `scripts/docker-build.sh` | Build Docker image |
| `scripts/install.sh` | Install binaries to /usr/local/bin |

---

## Priority 6: Cleanup (old files)

### Files to delete:
| File | Reason |
|------|--------|
| `temp/` directory | Temporary files, should be in .gitignore |
| `*.bak_*` files | Backup files (e.g., `lib.rs.bak_p18`) |
| `graphify-out/` | Generated output, should be in .gitignore |

### Files to verify:
| File | Action |
|------|--------|
| `examples/*.wat` | Keep if used by wasmtime integration |
| `examples/*.json` | Keep if used by skill system |
| `docs/MARKETING.md` | Keep if relevant, remove if outdated |

---

## Priority 7: Documentation (consolidate)

### Current docs/:
| File | Lines | Status |
|------|-------|--------|
| INSTALL.md | 330 | ✅ Good |
| INSTALL_WINDOWS.md | 100 | ✅ Good |
| INSTALL_ANDROID.md | 73 | ✅ Good |
| DEPLOYMENT.md | ~300 | ✅ Good |
| DEVELOPMENT_PLAN.md | ~400 | ✅ Good |
| INTEGRATION.md | ~250 | ✅ Good |
| SDK.md | ~170 | ✅ Good |
| MARKETING.md | ~200 | ⚠️ Review |

### Proposed additions:
| File | Purpose |
|------|---------|
| `docs/CONTRIBUTING.md` | Contribution guidelines |
| `docs/ARCHITECTURE.md` | High-level architecture diagram |
| `docs/TROUBLESHOOTING.md` | Common issues and solutions |

---

## Implementation Order

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 1 | Expand .gitignore | 10 min | High |
| 2 | Upgrade justfile | 30 min | High |
| 3 | Upgrade pre-commit.sh | 15 min | Medium |
| 4 | Add CI improvements | 30 min | High |
| 5 | Add scripts/ | 45 min | Medium |
| 6 | Clean old files | 15 min | Medium |
| 7 | Add docs/ | 60 min | Medium |

**Total estimated time**: ~3 hours

---

## Proven Patterns Reference

| Project | Pattern | Applied |
|---------|---------|---------|
| ripgrep | justfile with 30+ targets | ✅ Priority 2 |
| rust-analyzer | cargo-deny + MSRV check | ✅ Priority 4 |
| Tauri | Multi-platform release workflow | ✅ Priority 4 |
| Embassy | ESP32 cross-compilation | ✅ esp32-s3 template |
| tokio | Comprehensive .gitignore | ✅ Priority 1 |
| serde | Pre-commit with staged-only check | ✅ Priority 3 |
