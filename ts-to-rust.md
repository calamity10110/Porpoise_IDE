# Porpoise: TypeScript → Rust Migration Guide

> This document maps the transition from the original TypeScript/Node.js project (Orca) to the Rust-native implementation (Porpoise).

---

## Why Rust?

| Aspect | TypeScript (Orca) | Rust (Porpoise) |
|--------|-------------------|-----------------|
| **Memory** | Garbage-collected, ~300MB baseline (Electron) | Ownership model, <50MB baseline |
| **Startup** | ~3–5s (V8 init + module graph) | <200ms cold start (native binary) |
| **Type Safety** | Optional (TS), prototype pollution risk | Mandatory, enforced at compile time |
| **Concurrency** | Single-threaded event loop | Tokio async multi-threaded, fearless concurrency |
| **Package Size** | 300MB+ Electron bundle | ~15MB compiled binary |
| **Error Handling** | try/catch exceptions | `Result<T, E>` — errors as values |

---

## Toolchain Migration

| TypeScript Tool | Rust Equivalent | Status |
|-----------------|-----------------|--------|
| `tsconfig.json` | `Cargo.toml` | ✓ Workspace manifest with 17 crates |
| `npm` / `yarn` | `cargo` | ✓ Build, test, run — all in one |
| `jest` / `vitest` | `cargo test` | ✓ 95 tests, native parallel execution |
| `eslint` | `cargo clippy -- -D warnings` | ✓ 500+ lint rules enforced in CI |
| `prettier` | `cargo fmt` | ✓ max_width=120, imports_granularity=Crate |
| `ts-node` | `cargo run` | ✓ Native compilation, no runtime overhead |
| `nodemon` | `cargo watch` | ✓ Available via cargo-watch |
| `npm audit` | `cargo audit` | ✓ Security vulnerability scanning in CI |
| `npx` | `cargo run --example` | ✓ Example plugins in porpoise-skills |

---

## Architecture Migration

| Orca (TypeScript/Electron) | Porpoise (Rust/Tauri) |
|----------------------------|----------------------|
| Electron main/renderer | Tauri core/WebView |
| Node.js child_process | tokio::process |
| IPC via socket.io | porpoise-relay (Unix socket / Named pipe) |
| React frontend | Single HTML + CSS + vanilla JS frontend |
| npm-style plugins | WASM plugins via wasmtime |
| TypeScript classes | Rust structs + traits + enums |
| Prototype-based dispatch | Pattern matching + trait objects |

---

## Code Pattern Migration

### Error Handling

```typescript
// TypeScript — exceptions
function findWorktree(id: string): Worktree {
  const wt = db.worktrees.find(id);
  if (!wt) throw new NotFoundError(`Worktree ${id} not found`);
  return wt;
}
```

```rust
// Rust — errors as values
fn find_worktree(id: &WorktreeId) -> Result<Worktree, PorpoiseError> {
    db.worktrees()
        .find(id)
        .ok_or_else(|| PorpoiseError::NotFound(format!("Worktree {id} not found")))
}
```

### Async Patterns

```typescript
// TypeScript — promise chains
async function spawnAgent(name: string): Promise<AgentHandle> {
  const process = await ProcessManager.spawn(name);
  return new AgentHandle(process);
}
```

```rust
// Rust — tokio async
async fn spawn_agent(name: &str) -> Result<AgentHandle, PorpoiseError> {
    let process = ProcessManager::spawn(name).await?;
    Ok(AgentHandle::new(process))
}
```

### State Management

```typescript
// TypeScript — mutable objects with event emitters
class AppState extends EventEmitter {
  private worktrees: Map<string, WorktreeState> = new Map();
  addWorktree(wt: WorktreeState): void {
    this.worktrees.set(wt.id, wt);
    this.emit('worktree:added', wt);
  }
}
```

```rust
// Rust — Arc<RwLock<>> + EventBus broadcast
struct AppState {
    worktrees: Arc<RwLock<HashMap<WorktreeId, WorktreeState>>>,
    bus: EventBus,
}

impl AppState {
    async fn add_worktree(&self, wt: WorktreeState) -> Result<(), PorpoiseError> {
        self.worktrees.write().await.insert(wt.id.clone(), wt.clone());
        self.bus.publish(SystemEvent::WorktreeAdded(wt)).await;
        Ok(())
    }
}
```

---

## Crate Mapping

| Orca Concept | Porpoise Crate | LOC |
|-------------|----------------|-----|
| Worktree manager | `porpoise-git` | 745 |
| Terminal emulation | `porpoise-terminal` | 953 |
| Process management | `porpoise-runtime` | 1005 |
| SSH connections | `porpoise-ssh` | 471 |
| Browser engine | `porpoise-browser` | 80 |
| HTTP/WS networking | `porpoise-network` | 340 |
| Agent integrations | `porpoise-agent` | 1,104 |
| Plugin system | `porpoise-skills` | 727 |
| IPC protocol | `porpoise-relay` | 684 |
| Database | `porpoise-db` | 671 |
| CLI | `porpoise-cli` | 671 |
| Server daemon | `porpoise-server` | 764 |
| Desktop app | `porpoise-app` | 277 |
| Core types | `porpoise-core` | 1,277 |
| Credential store | `porpoise-credentials` | 195 |
| Workflow engine | `porpoise-automation` | 309 |
| GUI automation | `porpoise-computer-use` | 95 |
| **Total** | **17 crates** | **~10,367** |

---

## Key Benefits Realized

1. **Zero unsafe code** outside audited PTY modules — memory safety guaranteed at compile time
2. **~20x smaller binaries** — 15MB vs 300MB Electron
3. **Deterministic performance** — no GC pauses, predictable latency
4. **Cross-platform** — Windows + macOS + Linux from a single codebase
5. **17 independent crates** — each independently testable, minimal cross-crate coupling
6. **Fearless concurrency** — Tokio async runtime with Send/Sync trait bounds
7. **WASM-sandboxed plugins** — no prototype pollution, capability-based security

---

*See [ARCHITECTURE.md](./ARCHITECTURE.md) for the system design and [AGENTS.md](./AGENTS.md) for Rust migration guidelines for AI agents.*