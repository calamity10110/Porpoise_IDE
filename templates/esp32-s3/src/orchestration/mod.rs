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
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            state: SystemState::Booting,
            uptime_start: 0,
            command_counter: 0,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtaError {
    InvalidChecksum,
    WriteFailed,
    SizeMismatch,
    NotStarted,
}
