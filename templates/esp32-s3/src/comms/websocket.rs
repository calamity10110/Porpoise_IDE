//! WebSocket handler for real-time bidirectional communication.
//!
//! Used for live telemetry streaming, remote control commands,
//! and OTA update progress reporting.

/// WebSocket message types.
#[derive(Debug)]
pub enum WsMessage {
    Text(heapless::String<1024>),
    Binary(heapless::Vec<u8, 4096>),
    Ping,
    Pong,
    Close,
}

/// WebSocket connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Telemetry data sent periodically to the orchestrator.
#[derive(Debug, Clone)]
pub struct TelemetryFrame {
    pub timestamp_ms: u64,
    pub free_heap: usize,
    pub cpu_usage: u8,
    pub wifi_rssi: i8,
    pub uptime_ms: u64,
    pub peripheral_status: heapless::Vec<(heapless::String<16>, bool), 16>,
}

/// Command received from orchestrator.
#[derive(Debug, Clone)]
pub struct RemoteCommand {
    pub id: u32,
    pub target: heapless::String<32>,  // peripheral name or "system"
    pub action: heapless::String<32>,  // e.g., "set_brightness", "capture_frame"
    pub payload: heapless::Vec<u8, 1024>,
}

/// Response to a remote command.
#[derive(Debug, Clone)]
pub struct CommandResponse {
    pub id: u32,
    pub success: bool,
    pub payload: heapless::Vec<u8, 4096>,
    pub error: Option<heapless::String<128>>,
}

/// WebSocket server for real-time control with token-based auth.
pub struct WebSocketServer {
    port: u16,
    state: WsState,
    auth_enabled: bool,
    auth_deadline_ms: u64,
}

impl WebSocketServer {
    pub fn new(port: u16, auth_enabled: bool) -> Self {
        Self {
            port,
            state: WsState::Disconnected,
            auth_enabled,
            auth_deadline_ms: 10_000, // 10s to authenticate after connect
        }
    }

    pub fn state(&self) -> WsState {
        self.state
    }

    /// Whether auth is required for this server.
    pub fn requires_auth(&self) -> bool {
        self.auth_enabled
    }

    /// Start WebSocket listener.
    pub fn listen(&mut self) -> Result<(), WsError> {
        self.state = WsState::Connecting;
        // Template — real impl uses embassy-net + tungstenite-like WS
        Ok(())
    }

    /// Send a telemetry frame to all connected clients.
    pub fn broadcast_telemetry(&mut self, _frame: &TelemetryFrame) -> Result<(), WsError> {
        Ok(())
    }

    /// Send a command response back to the requesting client.
    pub fn send_response(&mut self, _resp: &CommandResponse) -> Result<(), WsError> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsError {
    HandshakeFailed,
    SendFailed,
    RecvFailed,
    ConnectionClosed,
}
