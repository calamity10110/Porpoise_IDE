# Agent Ability Layer — Architecture Plan

> Methodology: subsystem-by-subsystem review with an explicit **CALL / REWRITE / DISCARD**
> verdict per component, each backed by acceptance tests. Written against the codebase
> state as of commit `f1dde3f`.

## 1. Goal

An AI agent (Claude Code, Codex, OpenCode, the mobile companion, or any external client)
can, through one uniform API:

1. **Discover** abilities (`ability/list`) with typed input/output schemas.
2. **Invoke** abilities (`ability/invoke`) regardless of where the implementation lives —
   Rust builtin, WASM plugin, MCP server tool, or automation workflow.
3. **Add / modify** abilities without touching daemon internals (drop-in WASM skill,
   MCP config entry, or workflow YAML).
4. **Remember** — persistent, scoped memory (`memory/*`) that survives daemon restarts.
5. Do all of this over the existing authenticated bridges: named pipe / Unix socket
   (local) and WSS (remote), with capability-based permissions.

## 2. Current State (verified)

| Fact | Evidence |
|---|---|
| 45 RPC methods registered on a `Router` | `crates/porpoise-server/src/services/relay_handlers.rs` |
| WSS bridge works end-to-end (TLS + token auth) | `wss://0.0.0.0:9876` in `server.log`; `WsRelayServer` in `crates/porpoise-relay/src/ws_server.rs` |
| Skills runtime is a stub: `install_simulated`, **empty WASM linker** | `crates/porpoise-skills/src/` |
| `HookRegistry` defines 9 hook types, none fired | `crates/porpoise-skills` |
| **No MCP code anywhere in the workspace** | grep across `crates/` |
| **No agent memory** (12 tables, none semantic/persistent per agent) | `crates/porpoise-db/src/migration.rs` |
| Credentials: AES-256-GCM + PBKDF2, name-based lookup | `crates/porpoise-credentials/` |
| Automation engine: 7 step types incl. `AgentCall`, `ApiCall` | `crates/porpoise-automation/` |
| Capability types exist (read/write/process/network, `merged()`, `normalize_lexical()`) | `crates/porpoise-core/src/types/capabilities.rs` |

## 3. Subsystem Verdicts (Morphling)

### 3.1 Relay Router + protocol — **CALL**

The `Request{id, method, params}` → `Response{id, status, body, error}` envelope with a
string-method router *is* the ability dispatch core. No rewrite needed; abilities become
a method namespace (`ability/*`, `memory/*`) plus a descriptor registry beside it.

**Acceptance test:** existing 45 methods keep passing `cargo test -p porpoise-server`;
new `ability/list` returns ≥1 descriptor after registering a builtin.

### 3.2 WsRelayServer bridge — **CALL**

Already carries TLS (self-signed + fingerprint), exact-token auth, and EventBus
broadcast. External agents (desktop, mobile, CLI agents) connect over this today.

**Acceptance test:** a WSS client authenticates and completes an `ability/invoke`
round-trip; wrong token is rejected (covered by existing auth tests, extend to new
methods).

### 3.3 porpoise-skills WASM runtime — **REWRITE (small, surgical)**

The wasmtime 49 runtime is solid (128 MB cap, fuel metering, BLAKE3-signed modules) but
the linker is empty and install is simulated. Rewrite the *linking* layer only:
- Real host functions: `log`, `http_fetch` (permission-gated), `kv_get/kv_set`
  (memory table), `emit_event` (EventBus).
- `handle_install` materializes the `.cwasm` + manifest into the data dir and preinstantiates.

**Acceptance test:** a test module compiled with `wat` exports `invoke(input) -> output`,
calls `log` + `kv_set`, passes the cap/fuel limits, and is rejected when unsigned.

### 3.4 HookRegistry — **CALL, then wire**

Nine hook types already modeled. Wire firing points: pre/post `ability/invoke`,
`agent/spawn`, `terminal/send`. No design change.

**Acceptance test:** a registered hook receives an event when an ability is invoked.

### 3.5 Memory — **DISCARD-and-build (nothing to keep)**

No persistent agent memory exists. `event_log` is telemetry, not memory. Build:

```sql
CREATE TABLE agent_memory (
  id TEXT PRIMARY KEY,
  scope TEXT NOT NULL,          -- 'agent:<id>' | 'session:<id>' | 'global'
  key TEXT NOT NULL,
  value TEXT NOT NULL,          -- JSON
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(scope, key)
);
```

RPC: `memory/get {scope, key}`, `memory/set {scope, key, value}`,
`memory/list {scope, prefix?}`, `memory/delete {scope, key}`.
A KV surface (not vector search) for v1 — semantic recall is a later crate
(`porpoise-memory`) and should NOT block the API.

**Acceptance test:** set → restart daemon → get returns value; scope isolation
(agent A cannot read agent B's scope without permission).

### 3.6 MCP integration — **BUILD (new crate `porpoise-mcp`)**

Two directions, one crate:
- **Client**: connect to external MCP servers (stdio and Streamable HTTP via the
  `rmcp` official SDK), list their tools, and **project each tool as an ability**
  (`mcp/<server>/<tool>`) into the registry.
- **Server**: expose Porpoise abilities as MCP tools over stdio, so any MCP-capable
  agent (Claude Code, Codex desktop) can drive Porpoise natively.

Config (data dir): `mcp_servers.json` → `[{ "name": "github", "transport": "stdio",
"command": "npx", "args": ["-y", "@modelcontextprotocol/server-github"], "env": {} }]`.

**Acceptance test:** with a fixture MCP server, `ability/list` includes its tools and
`ability/invoke {ability: "mcp/fixture/echo"}` returns the echoed payload.

### 3.7 Agent framework (porpoise-agent) — **CALL**

Adapters (Claude, Codex, OpenCode, Aider) spawn CLI agents with PTY + resume. Add one
narrow seam: a `--tool-prompt` block appended to prompts that tells CLI agents how to
call abilities via `porpoise` CLI (their only reliable egress), i.e. agents gain
abilities without in-process tool protocols.

**Acceptance test:** a spawned agent executes `porpoise ability invoke ...` and the
result is observable in the terminal transcript.

### 3.8 Credentials — **CALL**

Abilities never receive secrets; they reference them by name and the host resolves via
the existing AES-256-GCM store at invoke time (same pattern as the automation engine's
`CredentialLookup` step).

**Acceptance test:** invoking an ability whose descriptor declares
`required_credentials: ["github-token"]` without the credential present fails with a
typed error, not a panic.

### 3.9 Automation engine — **CALL**

Workflows are composed abilities. Register each workflow template as
`workflow/<name>` in the ability registry; invoke = run with params.

**Acceptance test:** a 2-step YAML workflow appears in `ability/list` and executes.

## 4. Ability Model (core types, porpoise-core)

```rust
pub struct AbilityDescriptor {
    pub id: String,              // "builtin/terminal.send", "mcp/github/create_issue",
                                 // "wasm/<skill>", "workflow/<name>"
    pub name: String,
    pub version: String,
    pub input_schema: serde_json::Value,   // JSON Schema
    pub output_schema: serde_json::Value,
    pub source: AbilitySource,   // Builtin | Wasm | Mcp { server } | Workflow
    pub required_capabilities: Capabilities,
    pub required_credentials: Vec<String>,
}

pub trait AbilityProvider: Send + Sync {
    fn descriptors(&self) -> Vec<AbilityDescriptor>;
    fn invoke(&self, id: &str, params: serde_json::Value, ctx: InvokeContext)
        -> impl Future<Output = Result<serde_json::Value>> + Send;
}
```

Four providers implement it (`builtin`, `wasm`, `mcp`, `workflow`); the registry merges
them; `ability/list` and `ability/invoke` are thin Router methods over the registry.
Capability checks run *before* invoke; hook registry fires pre/post.

## 5. Phasing

| Phase | Deliverable | Depends on |
|---|---|---|
| P1 | Registry + `ability/list` / `ability/invoke`; builtins = curated existing RPC methods | 3.1 |
| P2 | `agent_memory` table + `memory/*` RPC | 3.5 |
| P3 | `porpoise-mcp` client projection | 3.6 |
| P4 | WASM linker host functions + real install | 3.3, P1 |
| P5 | MCP server exposure + hook wiring + WSS scope tokens | 3.2, 3.4, P1–P3 |

## 6. Risk Notes

- **Do not** add an HTTP/REST server in v1 — WSS already covers external transport;
  a second transport is duplicate auth surface.
- WASM host functions are the largest attack surface: every host call re-checks the
  module's declared capabilities (the `Capabilities` type already supports subset
  matching — use `matches()`).
- MCP projection must namespace-isolate: two servers exposing the same tool name must
  both be reachable (`mcp/<server>/<tool>` guarantees it).
