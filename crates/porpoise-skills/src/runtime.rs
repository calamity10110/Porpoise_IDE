use porpoise_core::error::{PorpoiseError, Result};
use wasmtime::{Engine, Module, Store, Linker};

pub struct WasmRuntime {
    engine: Engine,
}

impl WasmRuntime {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<CompiledModule> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| PorpoiseError::Wasm(format!("compile: {e}")))?;
        Ok(CompiledModule { module, engine: self.engine.clone() })
    }
}

pub struct CompiledModule {
    module: Module,
    engine: Engine,
}

impl CompiledModule {
    pub fn instantiate(&self) -> Result<WasmInstance> {
        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, &self.module)
            .map_err(|e| PorpoiseError::Wasm(format!("instantiate: {e}")))?;
        Ok(WasmInstance { store, instance })
    }
}

pub struct WasmInstance {
    store: Store<()>,
    instance: wasmtime::Instance,
}

impl WasmInstance {
    pub fn call_func(&mut self, name: &str, params: &[wasmtime::Val]) -> Result<Vec<wasmtime::Val>> {
        let func = self.instance.get_func(&mut self.store, name)
            .ok_or_else(|| PorpoiseError::Wasm(format!("func '{name}' not found")))?;
        let ty = func.ty(&self.store);
        let mut results = vec![wasmtime::Val::I32(0); ty.results().len()];
        func.call(&mut self.store, params, &mut results)
            .map_err(|e| PorpoiseError::Wasm(format!("call '{name}': {e}")))?;
        Ok(results)
    }
}
