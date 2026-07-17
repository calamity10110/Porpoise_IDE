use std::path::{Path, PathBuf};

use porpoise_core::error::{PorpoiseError, Result};
use wasmtime::{Engine, Module};

use crate::sandbox::validate_imports;

/// Compiles a WAT (WebAssembly Text) source to WASM binary using wasmtime.
pub fn compile_wat_to_binary(wat_source: &str) -> Result<Vec<u8>> {
    let engine = Engine::default();
    let module = Module::new(&engine, wat_source).map_err(|e| PorpoiseError::Wasm(format!("WAT compile: {e}")))?;
    Ok(module
        .serialize()
        .map_err(|e| PorpoiseError::Wasm(format!("serialize: {e}")))?
        .to_vec())
}

/// Validates a WAT source string.
pub fn validate_wat(wat_source: &str) -> Result<()> {
    let engine = Engine::default();
    Module::new(&engine, wat_source).map_err(|e| PorpoiseError::Wasm(format!("invalid WAT: {e}")))?;
    Ok(())
}

/// Validates WASM binary bytes.
pub fn validate_wasm(wasm_bytes: &[u8]) -> Result<()> {
    let engine = Engine::default();
    Module::from_binary(&engine, wasm_bytes).map_err(|e| PorpoiseError::Wasm(format!("invalid WASM: {e}")))?;
    Ok(())
}

/// Extracts exported function names from a WAT source.
pub fn list_exports_wat(wat_source: &str) -> Result<Vec<String>> {
    let engine = Engine::default();
    let module = Module::new(&engine, wat_source).map_err(|e| PorpoiseError::Wasm(format!("parse: {e}")))?;
    Ok(module.exports().map(|e| e.name().to_string()).collect())
}

pub struct CompilationPipeline {
    engine: Engine,
    cache_dir: Option<PathBuf>,
}

pub struct CompiledSkill {
    pub id: String,
    pub module: Module,
    pub exports: Vec<String>,
}

impl CompilationPipeline {
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::default();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        let engine = Engine::new(&config).map_err(|e| PorpoiseError::Wasm(format!("engine: {e}")))?;
        Ok(Self {
            engine,
            cache_dir: None,
        })
    }

    pub fn with_cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = Some(dir);
        self
    }

    pub fn compile_from_wat(&self, skill_id: &str, wat_source: &str) -> Result<CompiledSkill> {
        let module = Module::new(&self.engine, wat_source)
            .map_err(|e| PorpoiseError::Wasm(format!("compile '{skill_id}': {e}")))?;

        let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();

        if let Some(ref cache_dir) = self.cache_dir {
            std::fs::create_dir_all(cache_dir).ok();
            let serialized = module
                .serialize()
                .map_err(|e| PorpoiseError::Wasm(format!("serialize: {e}")))?;
            let cache_file = cache_dir.join(format!("{skill_id}.cwasm"));
            std::fs::write(&cache_file, &serialized).ok();
        }

        Ok(CompiledSkill {
            id: skill_id.to_string(),
            module,
            exports,
        })
    }

    pub fn compile_from_wasm(&self, skill_id: &str, wasm_bytes: &[u8]) -> Result<CompiledSkill> {
        let module = Module::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| PorpoiseError::Wasm(format!("compile '{skill_id}': {e}")))?;

        // Reject modules with imports at compile time (fail fast)
        validate_imports(&module)?;

        let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
        Ok(CompiledSkill {
            id: skill_id.to_string(),
            module,
            exports,
        })
    }

    pub fn compile_from_file(&self, skill_id: &str, path: &Path) -> Result<CompiledSkill> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "wat" => {
                let content = std::fs::read_to_string(path).map_err(|e| PorpoiseError::Wasm(format!("read: {e}")))?;
                self.compile_from_wat(skill_id, &content)
            }
            "wasm" => {
                let bytes = std::fs::read(path).map_err(|e| PorpoiseError::Wasm(format!("read: {e}")))?;
                self.compile_from_wasm(skill_id, &bytes)
            }
            "cwasm" => {
                let bytes = std::fs::read(path).map_err(|e| PorpoiseError::Wasm(format!("read: {e}")))?;
                let module = unsafe { Module::deserialize(&self.engine, &bytes) }
                    .map_err(|e| PorpoiseError::Wasm(format!("deserialize: {e}")))?;
                let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
                Ok(CompiledSkill {
                    id: skill_id.to_string(),
                    module,
                    exports,
                })
            }
            _ => Err(PorpoiseError::Wasm(format!(
                "unsupported file extension: '{ext}' (expected .wat, .wasm, or .cwasm)"
            ))),
        }
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for CompilationPipeline {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            engine: Engine::default(),
            cache_dir: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WAT: &str = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)
            (func (export "double") (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul)
        )
    "#;

    #[test]
    fn test_validate_wat() {
        assert!(validate_wat(SAMPLE_WAT).is_ok());
        assert!(validate_wat("(invalid wasm").is_err());
    }

    #[test]
    fn test_list_exports_wat() {
        let exports = list_exports_wat(SAMPLE_WAT).unwrap();
        assert!(exports.contains(&"add".to_string()));
        assert!(exports.contains(&"double".to_string()));
    }

    #[test]
    fn test_pipeline_from_wat() {
        let pipeline = CompilationPipeline::new().unwrap();
        let skill = pipeline.compile_from_wat("test", SAMPLE_WAT).unwrap();
        assert_eq!(skill.id, "test");
        assert!(skill.exports.len() >= 2);
    }

    #[test]
    fn test_pipeline_from_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.wat");
        std::fs::write(&path, SAMPLE_WAT).unwrap();

        let pipeline = CompilationPipeline::new().unwrap();
        let skill = pipeline.compile_from_file("test", &path).unwrap();
        assert!(skill.exports.contains(&"add".to_string()));
    }
}
