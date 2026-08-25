//! Porpoise device protocol — JSON messages over WebSocket/HTTP.
//!
//! Defines the wire format for device ↔ orchestrator communication.

use serde::{Deserialize, Serialize};

/// Message envelope (all messages use this format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub msg_id: u32,
    pub msg_type: MsgType,
    pub payload: serde_json::Value,
}

/// Message type discriminator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgType {
    // Handshake
    Hello,
    Welcome,
    // Telemetry
    Telemetry,
    // Commands
    Command,
    CommandResult,
    // Peripheral control
    PeripheralRead,
    PeripheralWrite,
    PeripheralCommand,
    PeripheralStatus,
    // OTA
    OtaBegin,
    OtaChunk,
    OtaEnd,
    OtaStatus,
    // System
    Reboot,
    GetInfo,
    GetLogs,
    // Errors
    Error,
}

/// Hello message (device → orchestrator on connect).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloMsg {
    pub device_id: String,
    pub board: String,
    pub firmware_version: String,
    pub protocol_version: u8,
    pub capabilities: CapabilitiesMsg,
}

/// Device capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesMsg {
    pub peripherals: Vec<PeripheralInfoMsg>,
    pub free_heap: usize,
    pub total_heap: usize,
    pub psram_size: usize,
    pub flash_size: usize,
    pub cpu_mhz: u32,
}

/// Peripheral info for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralInfoMsg {
    pub name: String,
    pub category: String, // "display", "touch", "audio_in", "audio_out", "camera", "imu", "mouse", "keyboard", "custom"
    pub ready: bool,
    pub commands: Vec<String>,
}

/// Command request (orchestrator → device).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMsg {
    pub target: String,   // peripheral name or "system"
    pub action: String,   // e.g., "read", "write", "set_brightness"
    pub args: serde_json::Value,
}

/// Command result (device → orchestrator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResultMsg {
    pub command_id: u32,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

/// Telemetry frame (device → orchestrator, periodic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMsg {
    pub timestamp_ms: u64,
    pub free_heap: usize,
    pub cpu_temp: Option<f32>,
    pub wifi_rssi: i8,
    pub uptime_ms: u64,
    pub peripherals: Vec<PeripheralStatusMsg>,
}

/// Peripheral status in telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralStatusMsg {
    pub name: String,
    pub ready: bool,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// OTA chunk (orchestrator → device).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtaChunkMsg {
    pub offset: u32,
    pub data: Vec<u8>,
    pub total_size: u32,
    pub checksum: String,
}
