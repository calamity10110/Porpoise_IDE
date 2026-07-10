# Porpoise Skill Plugin SDK

> Build WASM-sandboxed plugins that extend Porpoise's agent orchestration.

## Overview

Porpoise plugins run inside a [wasmtime](https://wasmtime.dev/) sandbox. Each plugin is a WebAssembly module that exports functions which Porpoise calls in response to lifecycle events (hooks). Plugins cannot access the filesystem, network, or spawn processes unless explicitly granted via capability declarations.

## Plugin Structure

A plugin consists of:

| File | Purpose |
|------|---------|
| `*.wat` or `*.wasm` | The compiled WASM module |
| `*.json` | The skill manifest (metadata + capabilities + hooks) |

### Manifest Format

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "0.1.0",
  "description": "What it does",
  "entry": "my_plugin.wat",
  "capabilities": {
    "fs_read": [],
    "fs_write": [],
    "network": [],
    "process": [],
    "ssh": []
  },
  "hooks": ["on_agent_output"],
  "fuel_limit": 1000000
}
```

### Capabilities

All capabilities default to empty (no access). Populate arrays to grant specific permissions:

| Capability | Type | Description |
|------------|------|-------------|
| `fs_read` | `Vec<PathBuf>` | Filesystem paths the plugin can read |
| `fs_write` | `Vec<PathBuf>` | Filesystem paths the plugin can write |
| `network` | `Vec<UrlPattern>` | URL patterns the plugin can access |
| `process` | `Vec<ProcessPattern>` | Binaries the plugin can spawn |
| `ssh` | `Vec<HostPattern>` | SSH hosts the plugin can connect to |

## Hooks

Hooks are lifecycle events that trigger plugin function calls:

| Hook | When | Context Fields |
|------|------|----------------|
| `on_agent_start` | Agent spawns | `agent_id`, `worktree_path` |
| `on_agent_output` | Agent produces output | `agent_id`, `payload.output` |
| `on_agent_exit` | Agent terminates | `agent_id`, `payload.exit_code` |
| `on_terminal_create` | Terminal allocated | `terminal_id` |
| `on_terminal_output` | Terminal writes data | `terminal_id`, `payload.data` |
| `on_terminal_close` | Terminal closed | `terminal_id` |
| `on_worktree_create` | Worktree created | `worktree_path` |
| `on_worktree_delete` | Worktree removed | `worktree_path` |
| `on_config_reload` | Config reloaded | (none) |

## Writing a Plugin (WAT)

```wat
(module
  (memory (export "memory") 1)

  ;; Called when an agent produces output.
  ;; Returns the number of bytes to "replace" (0 = pass-through).
  (func (export "on_agent_output") (param $ptr i32) (param $len i32) (result i32)
    (local $i i32)
    (local $count i32)
    (local.set $i (i32.const 0))
    (local.set $count (i32.const 0))
    (block $done
      (loop $scan
        (br_if $done (i32.ge_s (local.get $i) (local.get $len)))
        ;; your processing logic here
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $scan)))
    (local.get $count))

  (func (export "version") (result i32) (i32.const 1)))
```

## Compiling and Installing

### Compile from WAT to WASM

```rust
use porpoise_skills::pipeline::CompilationPipeline;

let pipeline = CompilationPipeline::new()?;
let skill = pipeline.compile_from_wat("my-plugin", wat_source)?;
```

### Or compile from file

```rust
let skill = pipeline.compile_from_file("my-plugin", Path::new("plugins/my_plugin.wat"))?;
```

### Install via CLI

```bash
porpoise skill install my-plugin
porpoise skill list
porpoise skill uninstall my-plugin
```

## Rust API

### CompilationPipeline

```rust
pub struct CompilationPipeline {
    pub fn new() -> Result<Self>;
    pub fn with_cache_dir(self, dir: PathBuf) -> Self;
    pub fn compile_from_wat(&self, skill_id: &str, wat_source: &str) -> Result<CompiledSkill>;
    pub fn compile_from_wasm(&self, skill_id: &str, wasm_bytes: &[u8]) -> Result<CompiledSkill>;
    pub fn compile_from_file(&self, skill_id: &str, path: &Path) -> Result<CompiledSkill>;
}
```

### HookRegistry

```rust
pub struct HookRegistry {
    pub fn new() -> Self;
    pub async fn register(&self, hook_type: HookType, skill_id: &str, callback: HookCallback);
    pub async fn unregister_skill(&self, skill_id: &str);
    pub async fn fire(&self, ctx: &HookContext) -> HookResults;
    pub async fn fire_agent_start(&self, agent_id: AgentId, worktree_path: &str) -> HookResults;
    pub async fn fire_agent_output(&self, agent_id: AgentId, output: &str) -> HookResults;
    pub async fn fire_terminal_create(&self, terminal_id: TerminalId) -> HookResults;
}
```

### SandboxedRuntime

```rust
pub struct SandboxedRuntime {
    pub fn new() -> Result<Self>;
    pub fn instantiate(&self, wasm_bytes: &[u8], capabilities: &Capabilities, fuel: u64) -> Result<SandboxedInstance>;
}

pub struct SandboxedInstance {
    pub fn call_func(&mut self, name: &str, params: &[Val], required_cap: Option<&str>) -> Result<Vec<Val>>;
}
```

### HotReloadManager

```rust
pub struct HotReloadManager {
    pub fn new(pipeline: CompilationPipeline) -> Self;
    pub async fn add(&self, skill_id: &str, path: &Path) -> Result<()>;
    pub async fn check_for_changes(&self) -> Result<Vec<String>>;
    pub async fn get_bytes(&self, skill_id: &str) -> Option<Vec<u8>>;
}
```

## Example Plugins

See `examples/plugins/` for working examples:

| Plugin | Description |
|--------|-------------|
| `highlighter` | Adds ANSI color codes to keywords in output |
| `lint_checker` | Detects trailing whitespace in code |
| `sentiment_analyzer` | Scores text sentiment (+/- word counting) |

## Fuel Metering

Each plugin call is bounded by a fuel limit. The default is 1,000,000 fuel units per call. Exceeding the limit aborts execution. Adjust via the manifest:

```json
{
  "fuel_limit": 5000000
}
```

## Security Model

1. **No implicit access**: Plugins start with zero capabilities
2. **Capability-gated**: Each host function call checks granted capabilities
3. **CPU-bounded**: Fuel metering prevents infinite loops
4. **Memory-isolated**: Each instance gets its own linear memory
5. **No threads**: Plugins are single-threaded by design

## Debugging

Enable WASM tracing:

```bash
PORPOISE_SKILLS_LOG=debug porpoise skill list
```

Validate a WAT file:

```rust
porpoise_skills::pipeline::validate_wat(wat_source)?;
```

List exports:

```rust
let exports = porpoise_skills::pipeline::list_exports_wat(wat_source)?;
```
