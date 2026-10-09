# agent.md — Using and Extending Porpoise Abilities

> Manual for AI agents (Claude Code, Codex, OpenCode, or any automated client) that
> drive Porpoise. Architecture background: [docs/architecture/ABILITY_LAYER.md](./docs/architecture/ABILITY_LAYER.md).

## 1. Connecting

All ability operations go through the daemon's RPC surface.

**Local (same machine)** — the `porpoise` CLI is the simplest client:

```bash
porpoise launch                 # start daemon (and app), print pairing QR
porpoise status                 # daemon reachable?
```

**Remote (phone / another machine)** — WebSocket:

```
wss://<pc-lan-ip>:9876?token=<token>     # token: `porpoise launch` prints it
```

Wire protocol: JSON frames, `{"method": "...", "params": {...}, "id": "1", "token": "..."}` →
`{"id": "1", "status": "ok", "body": {...}}`. Auth: send the token as the `token` field
on the first frame (or `{"type": "auth", "token": "..."}`).

## 2. Discovering and Invoking Abilities

```bash
# list every ability (builtin, wasm skill, mcp tool, workflow)
porpoise ability list

# invoke one
porpoise ability invoke --name mcp/github/create_issue --params '{"title": "bug", "body": "..."}'
porpoise ability invoke --name memory/set --params '{"scope": "agent:codex-1", "key": "goal", "value": "ship v0.2"}'
```

Raw RPC (any client):

```json
{"method": "ability/list",  "params": {}, "id": "1"}
{"method": "ability/invoke", "params": {"ability": "memory/get", "args": {"scope": "global", "key": "onboarding"}}, "id": "2"}
```

Every descriptor carries `input_schema` / `output_schema` (JSON Schema) — validate before
invoking; `required_credentials` names secrets the host resolves for you (never pass
secret values in params).

## 3. Memory

Persistent KV scoped per agent/session/global. Survives daemon restarts (SQLite).

```json
{"method": "memory/set", "params": {"scope": "agent:codex-1", "key": "progress", "value": {"step": 3}}, "id": "1"}
{"method": "memory/get", "params": {"scope": "agent:codex-1", "key": "progress"}, "id": "2"}
{"method": "memory/list", "params": {"scope": "agent:codex-1", "prefix": "step"}, "id": "3"}
{"method": "memory/delete", "params": {"scope": "agent:codex-1", "key": "progress"}, "id": "4"}
```

Scope rules: use `agent:<your-agent-id>` for private state, `session:<id>` for
task-scoped state, `global` sparingly (shared conventions only). Agents cannot read
other agents' scopes unless granted the capability.

## 4. Adding Abilities — three ways

### 4.1 Builtin (Rust, shipped with the daemon)

1. Add a handler in `crates/porpoise-server/src/services/relay_handlers.rs`
   (`router.register("ability/<name>", ...)` — keep logic in a `services/*` module).
2. Register an `AbilityDescriptor` in the ability registry (input/output JSON Schema,
   required capabilities + credentials).
3. Add tests: invoke round-trip + capability-denied path. `cargo clippy --all-targets -- -D warnings` must stay clean.

### 4.2 WASM skill (sandboxed, no daemon rebuild)

Write a module exporting `invoke(input: string) -> string`, declare capabilities in the
manifest (network hosts, memory access), build to `.cwasm`, BLAKE3-sign it, then:

```bash
porpoise skill install ./my-skill.cwasm
porpoise ability invoke --name wasm/my-skill --params '{"x": 1}'
```

Sandbox limits: 128 MB memory, fuel-metered CPU, no ambient I/O — every host call
(`log`, `http_fetch`, `kv_get/kv_set`, `emit_event`) is checked against the manifest.

### 4.3 MCP server (external tools)

Add an entry to `mcp_servers.json` in the Porpoise data dir
(`%LOCALAPPDATA%\porpoise\` on Windows, `~/.local/share/porpoise/` on Linux/macOS):

```json
[{ "name": "github", "transport": "stdio", "command": "npx",
   "args": ["-y", "@modelcontextprotocol/server-github"], "env": {} }]
```

Restart the daemon (`porpoise launch --server`). Every tool the server exposes appears
as `mcp/github/<tool>` in `ability/list`.

## 5. Procedures (workflows)

Multi-step abilities are YAML workflows (7 step types: `AgentCall`, `WebAction`,
`ComputerAction`, `ApiCall`, `CredentialLookup`, `Delay`, `Condition`):

```bash
porpoise workflow validate ./deploy.yaml
porpoise workflow run ./deploy.yaml
```

A validated workflow automatically registers as `workflow/<id>` and is invocable like
any ability. See README "Automation & Workflow Engine" for the full YAML schema.

## 6. Rules of Conduct for Agents

1. **Validate before invoke.** Check the descriptor's schema; don't guess params.
2. **Never inline secrets.** Declare them in `required_credentials`; the host injects.
3. **Scope your memory.** `agent:<you>` for private, `global` only for shared conventions.
4. **Prefer composing** existing abilities (or a workflow) over adding builtins.
5. **Hooks are observed.** Pre/post-invoke hooks fire on every `ability/invoke`; assume
   actions are logged to the event log.
6. **Capabilities are deny-by-default.** If an invoke fails with a capability error,
   request the grant — do not work around it.
7. Daemon lifecycle: `porpoise launch` (both), `--server` / `--app` for one side;
   server logs land in `server.log` inside the data dir.
