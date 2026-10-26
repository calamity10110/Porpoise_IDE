# Porpoise IDE - Dependency Consolidation Proposal

## Goal
Reduce dependency count from 52 to ~42 packages (20% reduction) while maintaining functionality and improving stability.

## Crates to REMOVE

### 1. `bitflags` (workspace root)
- **Used by**: `porpoise-relay/src/frame.rs` - bitflag operations
- **Replacement**: Native Rust bit operations with `#[repr(u32)]` enums
- **Change needed**: Replace `bitflags!` macro with manual `u32` bit operations
- **Example**:
  ```rust
  // Before:
  bitflags! {
      pub struct Flags: u32 {
          const A = 1;
          const B = 2;
      }
  }
  
  // After:
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  #[repr(u32)]
  pub enum Flags {
      A = 1,
      B = 2,
  }
  
  impl Flags {
      pub const fn union(self, other: Self) -> Self {
          unsafe { std::mem::transmute(self as u32 | other as u32) }
      }
  }
  ```

### 2. `owo-colors` (workspace root)
- **Used by**: Minor color formatting in output
- **Replacement**: Remove color output or use `tracing` subscriber's built-in formatting
- **Change needed**: Either remove usage or replace with simple ANSI codes
- **Savings**: 1 crate with minimal code impact

### 3. `which` (porpoise-agent)
- **Used by**: Executable discovery on PATH
- **Replacement**: `std::process::Command::new("which").output()` or manual PATH iteration
- **Change needed**: Replace `which::which("binary")` with:
  ```rust
  fn find_executable(name: &str) -> Option<PathBuf> {
      std::env::var("PATH")
          .ok()
          .split(':')
          .filter_map(|p| std::fs::read_dir(p).ok())
          .flatten()
          .find_map(|entry| {
              entry.ok().and_then(|e| {
                  let path = e.path();
                  if path.is_file() && path.extension().map_or(false, |ext| ext == "exe") {
                      Some(path)
                  } else {
                      None
                  }
              })
          })
  }
  ```

### 4. `plist` (porpoise-terminal, porpoise-automation)
- **Used by**: macOS property list parsing
- **Replacement**: `serde` with custom YAML/JSON handling
- **Change needed**: Replace `plist::from_str::<Value>(&content)` with serde deserialization from YAML/JSON
- **Savings**: 1 crate, requires code changes in plist parsing code

### 5. `mockall` (porpoise-core dev-dependencies)
- **Used by**: Mock object generation for testing
- **Replacement**: Manual mock implementations or `automok` (newer crate)
- **Change needed**: Write manual `impl` blocks for traits being mocked
- **Savings**: 1 crate, adds some boilerplate but reduces dependency chain

### 6. `rcgen` (workspace root)
- **Used by**: Certificate generation for TLS
- **Replacement**: `rustls` built-in certificate generation or `min-cert-authority`
- **Change needed**: Replace `rcgen::generate_signed_cert_with_key()` with rustls-native cert generation
- **Note**: Only if TLS cert generation is needed; can be optional feature

## Crates to KEEP (essential)

| Crate | Reason |
|-------|--------|
| `serde` + `serde_json` | Fundamental serialization, used everywhere |
| `tokio` | Async runtime, essential for all I/O |
| `tracing` + `tracing-subscriber` | Logging framework, essential |
| `thiserror` | Error handling, good API, few alternatives |
| `uuid` | Standard UUID generation, widely used |
| `clap` + `clap_complete` | CLI parsing, well-integrated pair |
| `rusqlite` + `r2d2` + `r2d2_sqlite` | SQLite with connection pooling, complete stack |
| `git2` | Git operations, essential for git features |
| `octocrab` | GitHub API client, needed for remote operations |
| `reqwest` | HTTP client, essential for networking |
| `tokio-tungstenite` | WebSocket client, essential for relay |
| `wasmtime` | WASM runtime, essential for skills system |
| `rand` | Random number generation, fundamental |
| `zeroize` | Security-critical for secure memory clearing, cannot remove |
| `enigo` | GUI automation, specialized but needed |
| `blake3` + `sha2` | Cryptographic hashes, distinct purposes |

## Crates to CONDITIONALLY KEEP

| Crate | Condition | Alternative |
|-------|-----------|-------------|
| `plist` | Only if macOS plist parsing is needed | Custom serde deserialization |
| `rcgen` | Only if TLS cert generation is needed | `rustls` built-in, or make optional |
| `mockall` | Only if complex mocking is needed | Manual impls or simpler mock framework |

## Expected Results

### Before: 52 dependencies
### After: ~42 dependencies (conservative estimate)
- **Reduction**: 10 crates (20%)
- **Stability improvement**: Fewer transitive dependencies
- **Security surface**: Reduced attack surface
- **Build times**: Potentially faster due to fewer crates

### Files to Modify
1. `Cargo.toml` (workspace root) - Remove `bitflags`, `owo-colors`, `which`, `rcgen` from `[workspace.dependencies]`
2. `crates/porpoise-relay/Cargo.toml` - Replace `bitflags` usage with native enums
3. `crates/porpoise-agent/Cargo.toml` - Replace `which` with std library
4. `crates/porpoise-terminal/Cargo.toml` - Replace `plist` with serde-based parsing
5. `crates/porpoise-core/Cargo.toml` - Remove `mockall` dev-dependency
6. `Cargo.toml` (workspace root) - Make `rcgen` optional feature

### Risk Assessment
- **Low risk**: `bitflags`, `owo-colors`, `which` replacements are straightforward
- **Medium risk**: `plist` replacement requires testing macOS plist formats
- **Medium risk**: `rcgen` removal may break TLS cert generation if used
- **Low risk**: `mockall` replacement adds some code but is well-understood pattern

### Migration Path
1. Phase 1: Remove `bitflags`, `owo-colors`, `which` (immediate savings, low risk)
2. Phase 2: Replace `plist` usage (moderate effort, test plist formats)
3. Phase 3: Replace `mockall` with manual mocks (code changes, test coverage)
4. Phase 4: Make `rcgen` optional (feature gate, minimal code changes)