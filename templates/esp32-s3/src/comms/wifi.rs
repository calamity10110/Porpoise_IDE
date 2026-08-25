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
///
/// # Security
/// - No hardcoded passwords — must be provided at build time or via NVS.
/// - AP fallback uses a random per-device SSID suffix to prevent collisions.
/// - Minimum WPA2 password length: 8 characters.
#[derive(Debug, Clone)]
pub struct WifiConfig {
    pub mode: WifiMode,
    pub ssid: heapless::String<32>,
    pub password: heapless::String<64>,
    pub ap_ssid: heapless::String<32>,
    pub ap_password: heapless::String<64>,
    pub channel: u8,
    pub max_connections: u8,
    /// Max reconnect attempts before entering AP fallback mode.
    pub max_reconnect_attempts: u8,
    /// Whether to require auth on HTTP/WS after WiFi connects.
    pub require_auth: bool,
}

impl WifiConfig {
    /// Create a STA config (connect to existing network).
    /// Panics if password < 8 chars (WPA2 minimum).
    pub fn sta(ssid: &str, password: &str) -> Self {
        assert!(password.len() >= 8, "WPA2 password must be >= 8 characters");
        let mut s = heapless::String::new();
        let _ = s.push_str(ssid);
        let mut p = heapless::String::new();
        let _ = p.push_str(password);
        Self {
            mode: WifiMode::Station,
            ssid: s,
            password: p,
            ap_ssid: heapless::String::new(),
            ap_password: heapless::String::new(),
            channel: 1,
            max_connections: 1,
            max_reconnect_attempts: 5,
            require_auth: true,
        }
    }

    /// Create an AP config (device creates its own network).
    /// Panics if password < 8 chars.
    pub fn ap(ssid: &str, password: &str) -> Self {
        assert!(password.len() >= 8, "WPA2 password must be >= 8 characters");
        let mut s = heapless::String::new();
        let _ = s.push_str(ssid);
        let mut p = heapless::String::new();
        let _ = p.push_str(password);
        Self {
            mode: WifiMode::AccessPoint,
            ssid: heapless::String::new(),
            password: heapless::String::new(),
            ap_ssid: s,
            ap_password: p,
            channel: 1,
            max_connections: 4,
            max_reconnect_attempts: 5,
            require_auth: true,
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
