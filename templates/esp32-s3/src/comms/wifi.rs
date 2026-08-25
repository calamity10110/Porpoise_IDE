//! WiFi connection manager.
//!
//! Handles STA (station) and AP (access point) modes.
//! Auto-reconnect on disconnect.

use esp_hal::peripherals::WIFI;

/// WiFi operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiMode {
    /// Station mode — connect to existing network.
    Station,
    /// Access Point mode — create own network.
    AccessPoint,
    /// Both STA and AP simultaneously.
    Both,
}

/// WiFi configuration.
#[derive(Debug, Clone)]
pub struct WifiConfig {
    pub mode: WifiMode,
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
    pub ap_ssid: heapless::String<32>,
    pub ap_password: heapless::String<64>,
    pub channel: u8,
    pub max_connections: u8,
}

impl Default for WifiConfig {
    fn default() -> Self {
        let mut ssid = heapless::String::new();
        let _ = ssid.push_str("Porpoise-ESP32S3");
        let mut password = heapless::String::new();
        let _ = password.push_str("porpoise123");
        let mut ap_ssid = heapless::String::new();
        let _ = ap_ssid.push_str("Porpoise-Setup");
        let mut ap_password = heapless::String::new();
        let _ = ap_password.push_str("setup1234");
        Self {
            mode: WifiMode::AccessPoint,
            ssid,
            password,
            ap_ssid,
            ap_password,
            channel: 1,
            max_connections: 4,
        }
    }
}

/// WiFi connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    Disconnected,
    Connecting,
    Connected,
    ApActive,
    Error,
}

/// WiFi manager — wraps esp-wifi for STA/AP.
pub struct WifiManager {
    config: WifiConfig,
    state: WifiState,
}

impl WifiManager {
    pub fn new(config: WifiConfig) -> Self {
        Self {
            config,
            state: WifiState::Disconnected,
        }
    }

    pub fn state(&self) -> WifiState {
        self.state
    }

    pub fn config(&self) -> &WifiConfig {
        &self.config
    }

    /// Initialize WiFi hardware and start connection.
    /// Must be called after esp-alloc and embassy are initialized.
    pub fn start(&mut self) -> Result<(), WifiError> {
        // Actual WiFi init requires esp-wifi + esp-hal peripherals
        // This is a template — real init happens in main.rs with peripheral access
        self.state = WifiState::Connecting;
        Ok(())
    }

    /// Get current IP address (after connected).
    pub fn ip_address(&self) -> Option<heapless::String<16>> {
        // Placeholder — real implementation reads from esp-wifi stack
        None
    }

    /// Disconnect from current network.
    pub fn disconnect(&mut self) {
        self.state = WifiState::Disconnected;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiError {
    InitFailed,
    ConnectFailed,
    ApFailed,
    Timeout,
}
