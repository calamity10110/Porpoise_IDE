# Module Design: porpoise-agent

> Agent integration protocols — Claude Code, Codex, Gemini, and generic CLI agents.

## Purpose
`porpoise-agent` detects, spawns, communicates with, and manages AI coding agents. It provides the `Agent` trait abstraction so all agent kinds work through the same interface.

## Dependencies
- porpoise-core (types, events)
- porpoise-runtime (process management)

## Key Types
- `Agent` trait: `spawn()`, `send_input()`, `read_output()`, `interrupt()`, `shutdown()`
- `AgentDetector`: scans PATH for known agent binaries
- `AgentPool`: manages concurrent agent lifecycle

## Bridges
- porpoise-server creates AgentService from this crate
- porpoise-runtime provides ProcessManager for spawning
- Events flow through EventBus to UI
