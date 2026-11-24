# AG-UI Integration

Porpoise agent runs speak [AG-UI 1.0](https://docs.ag-ui.com/spec/1.0) — the
open event protocol that connects any agent to any app. A run started in
Porpoise streams the same typed events the TypeScript and Python SDKs emit,
so any AG-UI client (web app, mobile companion, terminal, CopilotKit) can
render it without a Porpoise-specific adapter.

## Pieces

| Piece | Crate / file | Role |
|-------|--------------|------|
| Protocol layer | `crates/porpoise-agui` | AG-UI 1.0 event types, SSE encoding, the `AgUiSession` translator |
| Output bridge | `crates/porpoise-agent/src/agui.rs` | `AgentOutput` / `ParsedEvent` → `AgUiInput` conversions |
| Run service | `crates/porpoise-server/src/services/agui.rs` | spawn, stream, event log, bus publishing, RPC handlers |
| Bus event | `AgentEvent::AgUi` in `porpoise-core` | one batch per read, forwarded by the WebSocket relay |
| CLI | `porpoise agent agui-run / agui-events / agui-stop` | start, follow (SSE), interrupt |

The event types are a hand-ported subset of the frozen specification
`spec/1.0/schema.json` from [ag-ui-protocol/ag-ui](https://github.com/ag-ui-protocol/ag-ui)
(MIT), field for field — the wire format is the spec's, so foreign clients
validate what we emit.

## Event mapping

One `AgUiSession` per run enforces the protocol rules: exactly one
`RUN_STARTED`, open message streams closed before the terminal event,
exactly one `RUN_FINISHED`/`RUN_ERROR`, and nothing after it.

| Porpoise output | AG-UI events |
|-----------------|--------------|
| run start | `RUN_STARTED` (`protocolVersion: "1.0"`) |
| stdout text | `TEXT_MESSAGE_START` → `TEXT_MESSAGE_CONTENT`* → `TEXT_MESSAGE_END` |
| thinking | `REASONING_MESSAGE_START` → `…CONTENT`* → `…END` (closes the text stream first) |
| tool call | `TOOL_CALL_START` + `TOOL_CALL_ARGS` (whole JSON) + `TOOL_CALL_END` |
| tool result | `TOOL_CALL_RESULT` |
| status line | `STEP_STARTED` + `STEP_FINISHED` (same name) |
| stderr / non-fatal error | `CUSTOM` `porpoise.error` (run continues) |
| file diff | `CUSTOM` `porpoise.diff` (`file_path`, `old_content`, `new_content`, `language`) |
| waiting for input | `RUN_FINISHED` with `outcome: {type:"interrupt", interrupts:[…]}` — the run is paused |
| exit 0 | `RUN_FINISHED` with `outcome: {type:"success"}` |
| exit ≠ 0 | `RUN_ERROR` (`code: "exit_code"`) |
| user stop | `RUN_FINISHED` with `outcome: {type:"cancelled"}` |

`CUSTOM` names under the `porpoise.` prefix are reserved for Porpoise
signals; everything else is standard AG-UI.

## RPC

Three relay methods (same IPC/WS transport as every other `porpoise` RPC):

### `agent_agui_run`

```json
{ "kind": "opencode", "worktree": "C:/src/myrepo", "prompt": "fix the tests", "thread_id": null }
```

`thread_id` is optional — pass a previous thread id to continue that
conversation. Returns immediately:

```json
{
  "run_id": "…", "thread_id": "…", "agent_id": "…",
  "protocol_version": "1.0", "status": "started"
}
```

The prompt reaches the agent through its adapter's CLI arguments
(claude, codex, aider, opencode) or over stdin for bare binaries.

### `agent_agui_events`

```json
{ "run_id": "…", "after": 0 }
```

Polls the run's ordered event log from offset `after`:

```json
{
  "run_id": "…", "thread_id": "…", "agent_id": "…",
  "events": [ { "type": "RUN_STARTED", … }, … ],
  "total": 12, "finished": false
}
```

`finished: true` means the terminal event has been logged; nothing follows
it.

### `agent_agui_stop`

```json
{ "run_id": "…" }
```

Interrupts the live run (SIGINT on Unix). It ends with the `cancelled`
outcome. Returns `{"stopping": true}`, or `stopping: false` when the run
already ended.

## CLI

```bash
# start a run (prints run_id / thread_id)
porpoise agent agui-run opencode --worktree C:/src/myrepo --prompt "fix the tests"

# stream the run as SSE frames until it finishes
porpoise agent agui-events --run-id <run_id> --follow

# poll once from an offset (JSON or --json for pretty output)
porpoise agent agui-events --run-id <run_id> --after 12

# interrupt
porpoise agent agui-stop --run-id <run_id>
```

`--follow` writes one `data: {json}\n\n` frame per event to stdout and
stops at the terminal event — pipe it into any SSE consumer:

```bash
porpoise agent agui-events --run-id "$RUN" --follow | while read -r line; do
  [[ $line == data:* ]] && echo "${line#data: }"
done
```

## Event bus / WebSocket

Every batch is also published as

```rust
SystemEvent::Agent(AgentEvent::AgUi { id, run_id, events })
```

on the `EventBus`, which the WebSocket relay (`PORPOISE_WS_PORT`)
forwards to connected clients. Batches preserve order; the run's terminal
event arrives as the last batch.

## Consuming with the TypeScript client

Any AG-UI consumer can drive a Porpoise run over SSE. Serve the frames
`agent_agui_events --follow` produces (or a future HTTP endpoint) as
`text/event-stream` and point `HttpAgent` at them; for the polling RPC,
replay the collected events into an AG-UI reducer in order — the payload
is already spec-shaped:

```ts
// events from agent_agui_events, in order
for (const event of events) {
  // { type: "RUN_STARTED", threadId, runId, protocolVersion: "1.0", … }
  reducer(event);
}
```

## Design notes

- `porpoise-agui` is a leaf crate: it knows nothing about agents, so the
  protocol layer can be reused by any Porpoise component that needs to
  emit events (and later, a server-side `RunAgentInput` consumer).
- The run task takes the process handle out of the pool
  (`AgentPool::take_handle`) so exactly one task reads the process; the
  handle is restored afterwards, or cleaned up if the pool entry was
  removed mid-run.
- Stop uses `Notify` (a stored permit), so an interrupt that arrives
  before the run task reaches its select is never lost.
- After the process exits, `GenericHandle` reports a single
  `OutputType::Exit(code)` and drains stdout/stderr first, so no output
  can arrive after the terminal event.
