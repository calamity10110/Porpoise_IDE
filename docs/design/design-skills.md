# Module Design: porpoise-skills

> WASM plugin runtime — wasmtime-based skill system.

## Purpose
A sandboxed plugin system where community-contributed "skills" run as WASM modules. Plugins have no filesystem or network access by default — capabilities are explicitly granted.

## Dependencies
- porpoise-core (Capabilities, EventHandler)
- wasmtime (WASM runtime)

## Key Types
- `WasmRuntime`: compile and run WASM plugins
- `SkillRegistry`: install/list/enable/disable/uninstall
- `HookSystem`: on_agent_start, on_agent_output, on_terminal_create
- WIT-based API boundary for plugin interface

## Bridges
- Hook system subscribes to EventBus for agent/terminal events
- Plugin manifest declares required capabilities
- Server exposes skill management via IPC
