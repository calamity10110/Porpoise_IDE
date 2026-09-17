# Porpoise IDE - Functional Programming Pattern Enhancement Plan

## Overview
This plan outlines enhancements to the existing functional programming patterns in the Porpoise IDE codebase, focusing on reducing redundancy, improving type safety, and increasing composability without breaking changes.

## Current State Assessment

### **Strong Existing FP Patterns** (well-implemented):
- **Result/Either pattern**: `thiserror::Error` + `Result<T>` throughout core
- **Event Bus**: `tokio::sync::broadcast` with O(1) clone in `bus.rs`
- **Trait Strategy**: `Command` trait with associated `Output` type in `traits/command.rs`
- **Event Handler**: `interested_in()` filter pattern in `traits/event_handler.rs`
- **Newtype IDs**: `id_type!` macro with V7 UUIDs in `types/id.rs`
- **Enum State Machines**: Exhaustive `SystemEvent` enum in `types/event.rs`
- **Immutable Structs**: `AppState`/`WorktreeState`/`AgentState` with `Arc<RwLock<>>`
- **Serialization**: `to_json`, `from_json`, `to_bincode`, `from_bincode` in `serialization.rs`
- **Platform Enums**: `Platform` with `is_unix()` + `Display` in `platform.rs`

### **Areas for Enhancement**:

## Plan Phases

### **Phase 1: Error Handling Enhancement** 
**Priority**: 🟠 High impact, lowest risk  
**Goal**: Consistent error context & source chaining  

**Changes**:
- Add optional `source: Option<Box<dyn std::error::Error + Send + Sync>>` to select `PorpoiseError` variants that wrap domain errors
- Add `porpoise_error::wrap_error()` helper function
- Ensure all `From<T>` impls properly chain sources
- Add `cause()` accessor method to retrieve wrapped source error

**Affected Variants** (add source field):
- `Db(String)` → `Db { source: Option<Box<dyn Error + Send + Sync>>, msg: String }` 
  Actually, simpler: keep String but add separate `source` field to top-level enum

**More practical approach**: Add a `from_error()` method and `source()` accessor:
```rust
// In error.rs, add:
impl PorpoiseError {
    pub fn source(&self) -> Option<&(dyn std::error::Error + Send + Sync)> {
        // extract if wrapped
        None  // future expansion
    }
    
    pub fn wrap<E: std::error::Error + Send + Sync + 'static>(msg: E) -> Self {
        // Could wrap, but for now just use existing String variant
        PorpoiseError::Internal(msg.to_string())
    }
}
```

**Files**: `crates/porpoise-core/src/error.rs`  
**Breaking change**: None (optional field, backward compatible)  
**Tests**: Add `test_error_source_chain()` in `error.rs`

---

### **Phase 2: Event Handling Pipeline**
**Priority**: 🟡 Medium impact, backward compatible  
**Goal**: More composable event processing, reduce handler redundancy  

**Changes**:
- Add default methods to `EventHandler` trait:
  - `handle_map<F>(self, f: F) -> Pin<Box<dyn Future<Output = Result<()>>>>`
    where `F: Fn(&SystemEvent) -> bool + Send + Sync + 'static`
  - `then_handle<F, G>(self, f: F, g: G) -> Pin<Box<dyn Future<Output = Result<()>>>>`
    where `F: Fn(&SystemEvent) -> bool`, `G: Fn(&SystemEvent) -> Result<()> + Send + Sync + 'static`
- Add `EventProcessor` struct for fluent chaining:
  ```rust
  EventProcessor::new(bus)
    .filter(|event| matches!(event, SystemEvent::Agent(_)))
    .then_handle(|event| { /* handle */ })
    .await
  ```

**Files**: 
- `crates/porpoise-core/src/traits/event_handler.rs` - Add default methods
- `crates/porpoise-server/src/lib.rs` - Use pipeline in notification handler  
- `crates/porpoise-terminal/src/lib.rs` - Use pipeline in terminal events

**Breaking change**: None (default trait methods, fully backward compatible)  
**Tests**: Add `test_event_pipeline()` in relevant test modules

---

### **Phase 3: Capability Set Operations**
**Priority**: 🟡 Medium impact, backward compatible  
**Goal**: Functional capability operations, reduce manual checking loops  

**Changes**:
- Add to `Capabilities` struct methods:
  - `union(&self, other: &Capabilities) -> Capabilities`
  - `intersect(&self, other: &Capabilities) -> Capabilities`
  - `difference(&self, other: &Capabilities) -> Capabilities`
  - `matches(&self, other: &Capabilities) -> bool` (is subset/check if self >= other)
- Add to `CapabilityScope` methods:
  - `matches_path(&self, path: &Path) -> bool`
  - `matches_url(&self, url: &str) -> bool`
- Add convenience: `Capabilities::all_read(path: PathBuf) -> Capabilities`, `Capabilities::all_network(url: String) -> Capabilities`

**Files**: 
- `crates/porpoise-core/src/types/capabilities.rs` - Add set operations
- `crates/porpoise-skills/src/lib.rs` - Replace manual loops with `cap.matches(plugin_caps)`
- `crates/porpoise-ssh/src/lib.rs` - Replace manual loops with `cap.matches(user_caps)`

**Breaking change**: None (new methods on existing structs)  
**Tests**: Add `test_capability_operations()` in `capabilities.rs`

---

### **Phase 4: ID Type System Enhancement**
**Priority**: 🟢 Low impact, backward compatible  
**Goal**: More flexible ID creation, better error messages  

**Changes**:
- Enhance `id_type!` macro to add:
  - `try_from_str(s: &str) -> Result<Self>` associated function
  - `into_string(self) -> String` consuming method
  - `static is_valid_prefix(prefix: &str) -> bool` prefix validation
- Add `IdRange` struct for batch ID generation

**Files**: 
- `crates/porpoise-core/src/types/id.rs` - Enhance macro

**Breaking change**: None (backward compatible extensions)  
**Tests**: Add `test_id_methods()` in `id.rs` test module

---

### **Phase 5: Serialization Expansion**
**Priority**: 🟢 Low impact, backward compatible  
**Goal**: More automatic serialization, reduce boilerplate  

**Changes**:
- Add to `serialization.rs`:
  - `to_json_pretty_lazy() -> Pin<Box<dyn Stream<Item = Result<String>>>>` (for streaming)
  - `from_bincode_options(config: bincode::config::Configuration) -> ...`
  - `to_value<T: Serialize>(val: &T) -> Result<serde_json::Value>`
  - `from_value<T: DeserializeOwned>(val: serde_json::Value) -> Result<T>`
- Add `json!` macro alternative for test data

**Files**: 
- `crates/porpoise-core/src/serialization.rs` - Add helper functions

**Breaking change**: None (new functions, existing unchanged)  
**Tests**: Add `test_serialization_lazy()` and `test_json_helpers()` in `serialization.rs` tests

---

## **Execution Order & Dependencies**

```
Phase 1 (Error Handling) → Phase 2 (Event Pipeline) → Phase 3 (Capabilities)
      ↑                                                    |
      |-- All backward compatible → Phase 4 (IDs)      |
                                                Phase 5 (Serialization)
```

**Recommended order**: Start with Phase 1 (safest, no breaking changes), then Phase 2 (default methods), then Phases 3-5 in any order.

---

## **Risk Assessment**

| Phase | Breaking? | Effort | Test Coverage | Rollback |
|-------|-----------|--------|---------------|----------|
| 1 | None | Low (1 file) | Add 3-5 tests | Trivial (remove source field) |
| 2 | None | Medium (3 files) | Add 5-8 tests | Trivial (remove default methods) |
| 3 | None | Medium (1 file + 2 crates) | Add 4-6 tests | Trivial (remove methods) |
| 4 | None | Low (1 file) | Add 3-4 tests | Trivial (remove macro extensions) |
| 5 | None | Low (1 file) | Add 3-5 tests | Trivial (remove helpers) |

**All phases are fully backward compatible** - can be disabled by not calling new methods/functions.

---

## **Success Metrics**

After implementation:
- ✅ Error sources chain properly (Phase 1)
- ✅ Event handlers can opt-in to pipeline (Phase 2)
- ✅ Capability checking is `cap.matches(...)` instead of manual loops (Phase 3)
- ✅ ID creation has more methods (Phase 4)
- ✅ Serialization has flexible options (Phase 5)
- ✅ 0% test failures from changes
- ✅ All existing integration tests pass

## **Next Steps**

1. **Start with Phase 1**: Error handling enhancement - edit `crates/porpoise-core/src/error.rs`
2. **Follow with Phase 2**: Event pipeline - edit `crates/porpoise-core/src/traits/event_handler.rs`
3. **Proceed with Phases 3-5** based on priority

**Each phase can be implemented independently** and verified with `cargo test --workspace` before moving to the next.