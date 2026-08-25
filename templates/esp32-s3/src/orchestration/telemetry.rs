//! Telemetry builder — periodic system health reporting.

use crate::comms::protocol::*;
use crate::peripherals::PeripheralRegistry;

/// Build a telemetry frame from current system state.
pub fn build(uptime_ms: u64, registry: &PeripheralRegistry) -> TelemetryMsg {
    let mut peripherals = Vec::new();

    for info in registry.list() {
        peripherals.push(PeripheralStatusMsg {
            name: info.name.clone(),
            category: info.category.clone(),
            ready: info.ready,
            error: None,
            data: None,
        });
    }

    TelemetryMsg {
        timestamp_ms: uptime_ms,
        free_heap: 0, // filled by caller with real value
        cpu_temp: None,
        wifi_rssi: 0,
        uptime_ms,
        peripherals,
    }
}
