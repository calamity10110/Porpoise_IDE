//! Orchestration layer — ties peripherals, comms, and board together.
//!
//! This is the main control loop that:
//! 1. Initializes the board and peripherals
//! 2. Starts WiFi and HTTP/WS servers
//! 3. Processes remote commands
//! 4. Sends telemetry
//! 5. Handles OTA updates

pub mod command_dispatcher;
pub mod telemetry;
pub mod ota;
pub mod behavior;

use crate::board::BoardConfig;
use crate::comms::protocol::*;
use crate::peripherals::PeripheralRegistry;

/// System state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    Booting,
    Initializing,
    Running,
    Updating,  // OTA in progress
    Error,
    Rebooting,
}

/// The orchestrator — main control hub.
pub struct Orchestrator {
    state: SystemState,
    uptime_start: u64,
    command_counter: u32,
    /// Runtime-updatable behavior config (survives firmware OTA).
    pub behavior: behavior::BehaviorConfig,
}

impl Orchestrator {
    pub fn new() -> Self {
        // Try loading behavior config from flash; fall back to empty defaults.
        let behavior = behavior::load_from_flash()
            .unwrap_or_else(behavior::BehaviorConfig::new);
        Self {
            state: SystemState::Booting,
            uptime_start: 0,
            command_counter: 0,
            behavior,
        }
    }

    /// Create orchestrator with board-specific default behavior config.
    pub fn with_board_defaults(board_name: &str) -> Self {
        let behavior = behavior::load_from_flash().unwrap_or_else(|| match board_name {
            "waveshare-349-touch" => behavior::default_waveshare_349_touch(),
            "waveshare-349" => behavior::default_waveshare_349(),
            _ => behavior::BehaviorConfig::new(),
        });
        Self {
            state: SystemState::Booting,
            uptime_start: 0,
            command_counter: 0,
            behavior,
        }
    }

    pub fn state(&self) -> SystemState {
        self.state
    }

    /// Initialize the system: board, peripherals, comms.
    pub fn init(&mut self) {
        self.state = SystemState::Initializing;
        // Real init sequence happens in main.rs with hardware access
    }

    /// Mark system as running.
    pub fn set_running(&mut self) {
        self.state = SystemState::Running;
    }

    /// Process an incoming command from the orchestrator.
    pub fn dispatch_command(
        &mut self,
        cmd: &CommandMsg,
        registry: &mut PeripheralRegistry,
    ) -> CommandResultMsg {
        self.command_counter += 1;
        command_dispatcher::dispatch(self.command_counter, cmd, registry)
    }

    /// Build a telemetry frame from current system state.
    pub fn build_telemetry(&self, registry: &PeripheralRegistry) -> TelemetryMsg {
        telemetry::build(self.uptime(), registry)
    }

    /// Get uptime in milliseconds.
    pub fn uptime(&self) -> u64 {
        // Real impl reads embassy time driver
        0
    }

    /// Handle an OTA chunk.
    pub fn handle_ota_chunk(&mut self, chunk: &OtaChunkMsg) -> Result<(), OtaError> {
        self.state = SystemState::Updating;
        ota::process_chunk(chunk)
    }

    /// Finalize OTA and reboot.
    pub fn finalize_ota(&mut self) -> Result<(), OtaError> {
        ota::finalize()
    }

    // -----------------------------------------------------------------------
    // Behavior config management (post-flash component updates)
    // -----------------------------------------------------------------------

    /// Update behavior config with a partial update (merge).
    /// Persists to flash and applies immediately — no reboot needed.
    pub fn update_behavior(
        &mut self,
        update: &behavior::BehaviorConfig,
    ) -> Result<(), behavior::BehaviorError> {
        behavior::apply_update(&mut self.behavior, update)
    }

    /// Get a snapshot of the current behavior config as JSON-like string.
    pub fn behavior_status(&self) -> alloc::string::String {
        use alloc::string::ToString;
        let mut out = alloc::string::String::from("{\"components\":{");
        for (i, comp) in self.behavior.components.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push('"');
            out.push_str(&comp.name);
            out.push_str("\":{\"enabled\":");
            out.push_str(if comp.enabled { "true" } else { "false" });
            out.push_str(",\"params\":{");
            for (j, (key, val)) in comp.params.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(key);
                out.push_str("\":");
                match val {
                    behavior::ConfigValue::Bool(b) => {
                        out.push_str(if *b { "true" } else { "false" });
                    }
                    behavior::ConfigValue::Int(n) => {
                        out.push_str(&n.to_string());
                    }
                    behavior::ConfigValue::Float(f) => {
                        out.push_str(&format!("{:.2}", f));
                    }
                    behavior::ConfigValue::Str(s) => {
                        out.push('"');
                        out.push_str(s);
                        out.push('"');
                    }
                }
            }
            out.push_str("}}");
        }
        out.push_str("}}");
        out
    }

    /// Check if a component is enabled in the behavior config.
    pub fn is_component_enabled(&self, name: &str) -> bool {
        self.behavior.is_enabled(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaError {
    InvalidChecksum,
    WriteFailed,
    SizeMismatch,
    NotStarted,
}
