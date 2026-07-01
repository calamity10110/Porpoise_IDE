# Module Design: porpoise-core

> Foundation layer — types, traits, errors, config, and event bus.
> Every other crate depends on this module.

---

## Purpose

`porpoise-core` defines the shared vocabulary of the entire Porpoise system. It provides the **type system**, **error hierarchy**, **configuration model**, **event bus**, and **core traits** that all other crates use to communicate with each other. Think of it as the "lingua franca" — no other crate should need to re-define concepts like `WorktreeId`, `PorpoiseError`, or `SystemEvent` because they live here.

**What problem it solves:** Without a shared core, every crate would need to define its own ID types, error enums, and event schemas. This would create brittle coupling between crates and make it impossible for them to interoperate safely. `porpoise-core` is the contract they all share.

---

## Dependencies

### External (crates.io)

| Crate | Version | Why |
|-------|---------|-----|
| `serde` / `serde_derive` | 1.x | Serialization for IPC, config, DB |
| `serde_json` | 1.x | JSON output for CLI, config values |
| `thiserror` | 2.x | `#[derive(Error)]` for `PorpoiseError` |
| `uuid` | 1.x | UUID v7 generation for entity IDs |
| `tokio` | 1.x | `sync::broadcast` for event bus |
| `chrono` | 0.4 | Timestamps `DateTime<Utc>` |
| `tracing` | 0.1 | Structured logging |
| `config` | 0.15 | Hierarchical config loading (TOML + env) |
| `bitflags` | 2.x | Bitflags for `FrameFlags`, `Capabilities` |

### Internal (workspace crates)

**None.** Core is the bottom of the dependency graph. All other crates depend on it.

---

## Required Input

`porpoise-core` is purely a **definition library** — it has no runtime input in the traditional sense. However, it accepts:

| Input | Source | Mechanism | Format |
|-------|--------|-----------|--------|
| Config file path | CLI / env var `PORPOISE_CONFIG` | `discover_config_path()` | `PathBuf` |
| Config TOML content | Filesystem | `config::Config::load()` | TOML string |
| Environment variables | OS | `config` crate auto-reads env | `PORPOISE_*` vars |
| No runtime data flow | — | — | — |

---

## Required Output

| Output | Consumer | Mechanism | Format |
|--------|----------|-----------|--------|
| `PorpoiseError` types | All crates | `Result<T>` propagation | `thiserror` enum |
| ID types | All crates | UUID v7 newtypes | `WorktreeId`, `TerminalId`, etc. |
| `AppConfig` | Server startup | Deserialized TOML | `config::Config` |
| `SystemEvent` enum | EventBus subscribers | `broadcast::Receiver` | Typed enum |
| `EventBus` | Server core | `clone()`-able handle | `broadcast::Sender` |
| `AppState` | Server tasks | `Arc<RwLock<>>` via `clone()` | Shared state handle |
| Core traits (`Command`, `EventHandler`, `StatePersister`) | All crates | Trait impls | Rust trait objects |

---

## Ownership

| Owns | Does Not Own |
|------|-------------|
| Type definitions (IDs, enums, structs) | Runtime state (worktrees, agents) |
| Error variant definitions | Process lifecycle |
| Config schema and defaults | Database connections |
| EventBus channel capacity | Socket listening |
| Trait definitions | Protocol state machines |
| Serialization format choices | File descriptors |

**Key principle:** Core owns *definitions*, not *instances*. It defines what a `Worktree` looks like but does not store or manage any actual worktrees. This keeps it a zero-cost abstraction — no runtime overhead, no mutable state.

---

## Program Flow

Since core is a definition library, it has no execution flow. The "flow" is purely at compile-time:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Compile-Time Dependency Flow                  │
│                                                                 │
│  porpoise-core provides:                                         │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │  Types: WorktreeId, TerminalId, AgentId, SessionId,     │  │
│    │         PageId, CorrelationId                            │  │
│    │  Errors: PorpoiseError + helpers                         │  │
│    │  Config: AppConfig + sub-configs (Db, Runtime, etc.)     │  │
│    │  Events: SystemEvent, EventBus                           │  │
│    │  State: AppState (Arc<RwLock<>>)                         │  │
│    │  Traits: Command, EventHandler, StatePersister           │  │
│    └─────────────────────────────────────────────────────────┘  │
│                        │                                         │
│                        ▼                                         │
│              ┌──────────────────┐                                │
│              │  All other       │                                │
│              │  porpoise crates │                                │
│              │  import and use  │                                │
│              │  these types     │                                │
│              └──────────────────┘                                │
└─────────────────────────────────────────────────────────────────┘
```

When the server starts, the *runtime flow* using core types looks like:

```
1. CLI starts → reads config → creates AppConfig
2. AppConfig passed to porpoise-server::Builder
3. Server creates: EventBus, AppState, DB pool
4. Service crates receive AppState handle via clone()
5. Services use AppState.event_bus().publish() to emit events
6. Subscribers receive typed SystemEvent via broadcast::Receiver
```

---

## Bridge to Other Modules

| Module | Bridge Type | What Flows | Direction |
|--------|-------------|------------|-----------|
| porpoise-db | Import + trait | `PorpoiseError`, DB config types | core → db |
| porpoise-cli | Import + type usage | `OutputFormat`, `CliConfig`, `Command` trait | core → cli |
| porpoise-runtime | Import + event | `ProcessHandle`, `RuntimeConfig`, events | core → runtime |
| porpoise-server | Import + state | `AppState`, `EventBus`, `AppConfig` | core → server |
| porpoise-relay | Import + types | `CorrelationId`, IPC message types | core → relay |
| porpoise-git | Import + error | `GitConfig`, `PorpoiseError::Git*` | core → git |
| porpoise-terminal | Import + event | `TerminalEvent`, terminal state | core → terminal |
| porpoise-agent | Import + event | `AgentEvent`, `AgentStatusKind` | core → agent |
| porpoise-ssh | Import + error | `SshConfig`, `PorpoiseError::Ssh*` | core → ssh |
| porpoise-browser | Import + event | `BrowserEvent`, `BrowserConfig` | core → browser |
| porpoise-app | Import + type | `OutputFormat`, shared types | core → app |
| porpoise-skills | Import + trait | `Capabilities`, `EventHandler` | core → skills |

**Direction convention:** `core → X` means porpoise-core is imported by crate X. Core never imports any other Porpoise crate.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| UUID v7 for IDs | Time-ordered, no central coordinator, chronologically sortable |
| `thiserror` over `anyhow` | Precise error types for library code; callers decide how to present |
| `tokio::sync::broadcast` | Multiple subscribers, last-value caching not needed (vs watch), lossy is OK for events |
| `Arc<RwLock<>>` for state | Interior mutability for read-heavy workloads; `RwLock` over `Mutex` because reads dominate |
| Definition-only | Keeps compile times fast, prevents circular dependencies, enables separate compilation |

---

*Core changes ripple everywhere. Any modification to types/errors/config must be validated across all consuming crates. Prefer additive changes (new enum variants, optional fields) over breaking ones.*
