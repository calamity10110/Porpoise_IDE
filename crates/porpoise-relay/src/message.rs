use porpoise_core::types::{event::SystemEvent, id::CorrelationId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    Request(Request),
    Response(Response),
    Event(SystemEvent),
    Handshake(Handshake),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    pub version: u8,
    pub min_version: u8,
    pub server_name: String,
    #[serde(default)]
    pub session_token: Option<String>,
    #[serde(default)]
    pub peer_pid: u32,
}

impl Default for Handshake {
    fn default() -> Self {
        Self::new()
    }
}

impl Handshake {
    pub fn new() -> Self {
        Self {
            version: crate::frame::PROTOCOL_VERSION,
            min_version: crate::frame::MIN_PROTOCOL_VERSION,
            server_name: "porpoise-server".into(),
            session_token: None,
            peer_pid: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: CorrelationId,
    pub method: String,
    pub params: serde_json::Value,
    pub timeout_ms: Option<u64>,
}

impl Request {
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id: CorrelationId::new(),
            method: method.into(),
            params,
            timeout_ms: Some(30_000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: CorrelationId,
    pub status: StatusCode,
    pub body: serde_json::Value,
    pub error: Option<ProtocolError>,
}

impl Response {
    pub fn ok(id: CorrelationId, body: serde_json::Value) -> Self {
        Self {
            id,
            status: StatusCode::Ok,
            body,
            error: None,
        }
    }

    pub fn err(id: CorrelationId, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            id,
            status: StatusCode::InternalError,
            body: serde_json::Value::Null,
            error: Some(ProtocolError {
                code,
                message: message.into(),
                details: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatusCode {
    Ok,
    Created,
    Accepted,
    NoContent,
    BadRequest,
    NotFound,
    Conflict,
    InternalError,
    Timeout,
    Unimplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidParams,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    RateLimited,
    Internal,
    Timeout,
    VersionMismatch,
    ConnectionRefused,
}
