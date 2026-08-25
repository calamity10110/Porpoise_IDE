//! Communication layer: WiFi, HTTP server, WebSocket.
//!
//! Provides the remote control interface for the Porpoise IDE to
//! review, orchestrate, and control the ESP32-S3 device.

pub mod wifi;
pub mod http_server;
pub mod websocket;
pub mod protocol;

/// Device identity sent during handshake.
#[derive(Debug, Clone)]
pub struct DeviceIdentity {
    pub device_id: heapless::String<32>,
    pub board_name: heapless::String<64>,
    pub firmware_version: heapless::String<16>,
    pub ip_address: heapless::String<16>,
    pub capabilities: DeviceCapabilities,
}

/// What the device can do (reported to orchestrator).
#[derive(Debug, Clone, Default)]
pub struct DeviceCapabilities {
    pub has_display: bool,
    pub has_touch: bool,
    pub has_mic: bool,
    pub has_speaker: bool,
    pub has_camera: bool,
    pub has_imu: bool,
    pub has_mouse: bool,
    pub has_keyboard: bool,
    pub custom_peripherals: heapless::Vec<heapless::String<32>, 8>,
    pub free_heap: usize,
    pub uptime_ms: u64,
}
