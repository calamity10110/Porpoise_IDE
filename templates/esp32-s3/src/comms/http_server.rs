//! Minimal HTTP server for remote control.
//!
//! Runs on the ESP32-S3 using a bare-bones TCP listener.
//! Handles REST-like API endpoints for device management.

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

/// Parsed HTTP request.
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: heapless::String<128>,
    pub body: Option<heapless::Vec<u8, 4096>>,
    pub content_type: heapless::String<64>,
}

/// HTTP response to send back.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub content_type: heapless::String<64>,
    pub body: heapless::Vec<u8, 8192>,
}

impl Response {
    pub fn ok_json(json: &[u8]) -> Self {
        let mut ct = heapless::String::new();
        let _ = ct.push_str("application/json");
        let mut body = heapless::Vec::new();
        let _ = body.extend_from_slice(json);
        Self { status: 200, content_type: ct, body }
    }

    pub fn ok_text(text: &str) -> Self {
        let mut ct = heapless::String::new();
        let _ = ct.push_str("text/plain");
        let mut body = heapless::Vec::new();
        let _ = body.extend_from_slice(text.as_bytes());
        Self { status: 200, content_type: ct, body }
    }

    pub fn not_found() -> Self {
        let mut ct = heapless::String::new();
        let _ = ct.push_str("text/plain");
        let mut body = heapless::Vec::new();
        let _ = body.extend_from_slice(b"Not Found");
        Self { status: 404, content_type: ct, body }
    }

    pub fn error(status: u16, msg: &str) -> Self {
        let mut ct = heapless::String::new();
        let _ = ct.push_str("text/plain");
        let mut body = heapless::Vec::new();
        let _ = body.extend_from_slice(msg.as_bytes());
        Self { status, content_type: ct, body }
    }

    /// Serialize response to raw HTTP bytes.
    pub fn to_bytes(&self) -> heapless::Vec<u8, 8384> {
        let mut out = heapless::Vec::new();
        let status_text = match self.status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        let header = format_no_std::format!(
            heapless::String<256>,
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.status,
            status_text,
            self.content_type,
            self.body.len()
        );
        let _ = out.extend_from_slice(header.as_bytes());
        let _ = out.extend_from_slice(&self.body);
        out
    }
}

/// Route handler function type.
pub type RouteHandler = fn(&Request) -> Response;

/// HTTP server with registered routes.
pub struct HttpServer {
    port: u16,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Start listening. This is a blocking call.
    /// Real implementation uses esp-wifi's embedded-svc or raw TCP.
    pub fn listen(&self) -> Result<(), HttpError> {
        // Template placeholder — real impl uses embassy-net TcpListener
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpError {
    BindFailed,
    AcceptFailed,
    ParseError,
    Timeout,
}
