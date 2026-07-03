# Module Design: porpoise-skills

> WASM plugin runtime — wasmtime-based skill system.

## Purpose
A sandboxed plugin system where community-contributed "skills" run as WASM modules. Plugins have no filesystem or network access by default — capabilities are explicitly granted.

## Dependencies
- porpoise-core (Capabilities, EventHandler)
- wasmtime (WASM runtime with `wat` + `component-model` features)

## Key Types
- `WasmRuntime`: compile and run WASM plugins (supports WAT text format)
- `CompiledModule`: compiled WASM module with instantiate()
- `WasmInstance`: instantiated module with call_func()
- `SkillRegistry`: install/list/enable/disable/uninstall
- `SkillManifest`: plugin metadata (id, name, version, enabled)
- `HookSystem`: on_agent_start, on_agent_output, on_terminal_create
- WIT-based API boundary for plugin interface

## WASM Pipeline

```
WAT text ──> wasmtime::Module::new() ──> CompiledModule ──> instantiate() ──> WasmInstance
  │               │
  └── (wat       └── (cranelift JIT)
       feature)
       
Binary WASM ──> wasmtime::Module::new() ──> same pipeline
```

No external CLI tools required — `wasmtime` with `wat` feature compiles WAT to WASM in-process.
The `component-model` feature enables WIT-based component API for plugin interface definitions.

## Implemented Features
| Feature | Status | Details |
|---------|--------|---------|
| Engine creation | ✅ | `wasmtime::Engine::default()` |
| Module compilation | ✅ | `Module::new()` from binary or WAT bytes |
| Module instantiation | ✅ | `Linker::instantiate()` |
| Function calling | ✅ | `call_func()` with typed params/results |
| WAT text compilation | ✅ | `wat` feature enabled |
| Component model | ✅ | `component-model` feature enabled |
| Skill registry | ✅ | `SkillRegistry` with CRUD operations |
| Skill manifest | ✅ | `SkillManifest` with id/name/version |
| WASM compilation pipeline | ✅ | No external CLI tools needed |
| Capability sandboxing | ◐ | `Capabilities` struct defined, enforcement pending |
| Hook system | ◐ | EventBus-based, plugin integration pending |
| Plugin hot-reload | ◐ | File watcher logic pending |

## Bridges
- Hook system subscribes to EventBus for agent/terminal events
- Plugin manifest declares required capabilities
- Server exposes skill management via IPC
