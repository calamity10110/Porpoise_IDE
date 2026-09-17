//! ESP32-S3 device management service.
//!
//! Provides API endpoints for the Porpoise IDE to:
//! - List connected ESP32 devices
//! - Send commands to device peripherals
//! - Stream telemetry data
//! - Push OTA firmware updates
//! - Manage board configurations

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use porpoise_core::error::Result;
use serde::{Deserialize, Serialize};

/// Connected device registry (shared state).
pub type DeviceRegistry = Arc<RwLock<HashMap<String, DeviceInfo>>>;

/// Info about a connected ESP32 device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub board_name: String,
    pub ip_address: String,
    pub firmware_version: String,
    pub uptime_ms: u64,
    pub free_heap: usize,
    pub wifi_rssi: i8,
    pub peripherals: Vec<PeripheralInfo>,
    pub connected_at: String,
    pub last_telemetry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralInfo {
    pub name: String,
    pub category: String,
    pub ready: bool,
    pub error: Option<String>,
}

/// Create a new device registry.
pub fn new_registry() -> DeviceRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

/// List all connected devices.
pub async fn handle_list(registry: &DeviceRegistry) -> Result<serde_json::Value> {
    let devices = registry
        .read()
        .map_err(|e| porpoise_core::error::PorpoiseError::Internal(format!("registry lock: {e}")))?;
    let list: Vec<&DeviceInfo> = devices.values().collect();
    Ok(serde_json::json!({ "devices": list }))
}

/// Get details for a specific device.
pub async fn handle_get(registry: &DeviceRegistry, device_id: &str) -> Result<serde_json::Value> {
    let devices = registry
        .read()
        .map_err(|e| porpoise_core::error::PorpoiseError::Internal(format!("registry lock: {e}")))?;
    match devices.get(device_id) {
        Some(info) => Ok(serde_json::json!({ "device": info })),
        None => Err(porpoise_core::error::PorpoiseError::invalid_id("device", device_id)),
    }
}

/// Send a command to a device.
pub async fn handle_command(
    device_id: &str,
    target: &str,
    action: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    // In production this forwards via WebSocket to the device.
    // For now, return a placeholder acknowledging the command.
    Ok(serde_json::json!({
        "status": "queued",
        "device_id": device_id,
        "command": {
            "target": target,
            "action": action,
            "args": args,
        }
    }))
}

/// Push OTA firmware to a device.
pub async fn handle_ota_push(device_id: &str, firmware_url: &str, checksum: &str) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "ota_started",
        "device_id": device_id,
        "firmware_url": firmware_url,
        "checksum": checksum,
    }))
}

/// Get available board templates.
pub async fn handle_board_templates() -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "templates": [
            {
                "id": "waveshare_lcd_349",
                "name": "Waveshare ESP32-S3 Touch LCD 1.28\"",
                "description": "240x240 IPS LCD, CST816S touch, QMI8658 IMU, 8MB PSRAM",
                "peripherals": ["display", "touch", "imu", "battery"],
            },
            {
                "id": "custom",
                "name": "Custom Board",
                "description": "Start from scratch with your own pin mapping",
                "peripherals": [],
            }
        ]
    }))
}
