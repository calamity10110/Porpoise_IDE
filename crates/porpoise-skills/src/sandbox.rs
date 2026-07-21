use porpoise_core::{
    error::{PorpoiseError, Result},
    types::capabilities::Capabilities,
};
use wasmtime::{Engine, Linker, Module, Store};

/// Validate WASM module imports against allowed capabilities.
///
/// Currently denies ALL imports (secure by omission).
/// Returns `Ok` only if the module has zero imports.
/// In future, this will check each import against the granted capability set.
pub fn validate_imports(module: &Module) -> Result<()> {
    if let Some(import) = module.imports().next() {
        return Err(PorpoiseError::WasmImportDenied {
            module: import.module().to_string(),
            import: import.name().to_string(),
        });
    }
    Ok(())
}

/// Build a `Linker` with capability-gated host functions.
///
/// Currently returns an empty linker (no host functions).
/// In future, host functions will be added based on `Capabilities` fields.
pub fn build_capability_linker(engine: &Engine, _capabilities: &Capabilities) -> Linker<()> {
    Linker::new(engine)
}

/// Sandboxed WASM runtime that enforces capability-based permissions.
///
/// Each sandboxed instance is created with a set of `Capabilities`. The
/// `wasmtime::Linker` is configured to only expose host functions that match
/// the granted capabilities. Any attempt to call an unpermitted function
/// returns a `PluginCapabilityDenied` error.
pub struct SandboxedRuntime {
    engine: Engine,
}

impl SandboxedRuntime {
    pub fn new() -> Result<Self> {
        // Create engine with fuel metering for CPU limits
        let mut config = wasmtime::Config::default();
        config.consume_fuel(true);
        config.static_memory_maximum_size(128 * 1024 * 1024);
        config.max_wasm_stack(1024 * 1024);
        let engine = Engine::new(&config).map_err(|e| PorpoiseError::Wasm(format!("engine: {e}")))?;
        Ok(Self { engine })
    }

    /// Compiles and instantiates a WASM module with the given capabilities.
    ///
    /// The returned `SandboxedInstance` enforces the capability set at runtime:
    /// - `fs_read` / `fs_write`: filesystem access is gated
    /// - `network`: outbound connections are blocked unless explicitly allowed
    /// - `process`: process spawning is blocked unless explicitly allowed
    /// - `ssh`: SSH connections are blocked unless explicitly allowed
    pub fn instantiate(&self, wasm_bytes: &[u8], capabilities: &Capabilities, fuel: u64) -> Result<SandboxedInstance> {
        let module = Module::new(&self.engine, wasm_bytes).map_err(|e| PorpoiseError::Wasm(format!("compile: {e}")))?;

        // Validate imports before instantiation (fail fast)
        validate_imports(&module)?;

        let mut store = Store::new(&self.engine, ());
        store
            .set_fuel(fuel)
            .map_err(|e| PorpoiseError::Wasm(format!("fuel: {e}")))?;

        // Build a linker gated on the granted capabilities
        let linker = build_capability_linker(&self.engine, capabilities);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PorpoiseError::Wasm(format!("instantiate: {e}")))?;

        Ok(SandboxedInstance {
            store,
            instance,
            capabilities: capabilities.clone(),
            fuel_initial: fuel,
        })
    }
}

/// A WASM instance with enforced capability-based sandboxing.
///
/// Calls to exported functions that require unpermitted capabilities are
/// rejected at runtime with a `PluginCapabilityDenied` error.
pub struct SandboxedInstance {
    store: Store<()>,
    instance: wasmtime::Instance,
    capabilities: Capabilities,
    #[allow(dead_code)]
    fuel_initial: u64,
}

impl SandboxedInstance {
    /// Calls an exported function, checking capability requirements first.
    ///
    /// The `required_cap` parameter names the capability required (e.g.
    /// "network", "fs_write", "process"). If the capability is not in the
    /// granted set, the call is rejected without executing any WASM code.
    pub fn call_func(
        &mut self,
        name: &str,
        params: &[wasmtime::Val],
        required_cap: Option<&str>,
    ) -> Result<Vec<wasmtime::Val>> {
        // Check capability before executing
        if let Some(cap) = required_cap {
            self.check_capability(cap)?;
        }

        let func = self
            .instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| PorpoiseError::Wasm(format!("func '{name}' not found")))?;

        let ty = func.ty(&self.store);
        let mut results = vec![wasmtime::Val::I32(0); ty.results().len()];

        func.call(&mut self.store, params, &mut results)
            .map_err(|e| PorpoiseError::Wasm(format!("call '{name}': {e}")))?;

        Ok(results)
    }

    fn check_capability(&self, cap: &str) -> Result<()> {
        let granted = match cap {
            "fs_read" => !self.capabilities.fs_read.is_empty(),
            "fs_write" => !self.capabilities.fs_write.is_empty(),
            "network" => !self.capabilities.network.is_empty(),
            "process" => !self.capabilities.process.is_empty(),
            "ssh" => !self.capabilities.ssh.is_empty(),
            "all" => false, // "all" is never implicitly granted
            _ => false,
        };
        if !granted {
            return Err(PorpoiseError::PluginCapabilityDenied {
                id: "plugin".into(),
                capability: cap.into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Engine;

    #[test]
    fn test_no_imports_succeeds() {
        let engine = Engine::default();
        let wat = r#"(module (func (export "run") (result i32) i32.const 42))"#;
        let module = Module::new(&engine, wat).unwrap();
        assert!(validate_imports(&module).is_ok());
    }

    #[test]
    fn test_with_import_denied() {
        let engine = Engine::default();
        let wat = r#"(module (import "env" "log" (func (param i32))))"#;
        let module = Module::new(&engine, wat).unwrap();
        let err = validate_imports(&module).unwrap_err();
        match err {
            PorpoiseError::WasmImportDenied { ref module, ref import } => {
                assert_eq!(module, "env");
                assert_eq!(import, "log");
            }
            _ => panic!("expected WasmImportDenied"),
        }
    }
}
