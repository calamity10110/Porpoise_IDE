use std::{path::PathBuf, time::Duration};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::sync::{Mutex, watch};

#[cfg(windows)]
use crate::transport::pipe::NamedPipeTransport;
#[cfg(unix)]
use crate::transport::unix::UnixSocketTransport;
use crate::{
    auth::ConnectionState,
    frame::{Frame, FrameFlags},
    message::{Handshake, Request, WireMessage},
};

const BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 30_000;

#[cfg(unix)]
type InnerTransport = UnixSocketTransport;
#[cfg(windows)]
type InnerTransport = NamedPipeTransport;

pub struct RelayClient {
    socket_path: PathBuf,
    auth_token: Option<String>,
    transport: Mutex<InnerTransport>,
    connection_state: watch::Sender<ConnectionState>,
}

impl RelayClient {
    pub async fn connect(path: &std::path::Path) -> Result<Self> {
        let (tx, _) = watch::channel(ConnectionState::Disconnected);
        let transport = Self::connect_with_retry(path, false).await?;
        tx.send(ConnectionState::Connected).ok();
        Ok(Self {
            socket_path: path.to_path_buf(),
            auth_token: None,
            transport: Mutex::new(transport),
            connection_state: tx,
        })
    }

    pub async fn connect_with_auth(path: &std::path::Path, token: &str) -> Result<Self> {
        let (tx, _) = watch::channel(ConnectionState::Disconnected);
        let transport = Self::connect_authenticated(path, token, false).await?;
        tx.send(ConnectionState::Connected).ok();
        Ok(Self {
            socket_path: path.to_path_buf(),
            auth_token: Some(token.to_string()),
            transport: Mutex::new(transport),
            connection_state: tx,
        })
    }

    /// The server sends a Handshake on accept and validates the client's
    /// token. Any reconnection MUST repeat the handshake exchange — a plain
    /// reconnect reads the server's handshake frame as a call response and
    /// fails with "unexpected response".
    async fn connect_authenticated(path: &std::path::Path, token: &str, retry: bool) -> Result<InnerTransport> {
        let mut transport = Self::connect_with_retry(path, retry).await?;

        let _frame = transport
            .receive()
            .await
            .map_err(|e| PorpoiseError::Ipc(format!("read handshake: {e}")))?;

        let auth_hs = WireMessage::Handshake(Handshake {
            version: crate::frame::PROTOCOL_VERSION,
            min_version: crate::frame::MIN_PROTOCOL_VERSION,
            server_name: "porpoise-client".into(),
            session_token: Some(token.to_string()),
            peer_pid: std::process::id(),
        });
        let payload = serde_json::to_vec(&auth_hs).map_err(|e| PorpoiseError::Ipc(format!("serialize auth: {e}")))?;
        transport.send(&Frame::new(FrameFlags::EVENT, payload)).await?;
        Ok(transport)
    }

    async fn reconnect(&self) -> Result<InnerTransport> {
        match &self.auth_token {
            Some(token) => Self::connect_authenticated(&self.socket_path, token, true).await,
            None => Self::connect_with_retry(&self.socket_path, true).await,
        }
    }

    pub fn subscribe_state(&self) -> watch::Receiver<ConnectionState> {
        self.connection_state.subscribe()
    }

    async fn connect_with_retry(path: &std::path::Path, retry: bool) -> Result<InnerTransport> {
        if retry {
            let mut delay = BASE_DELAY_MS;
            loop {
                match InnerTransport::connect(path).await {
                    Ok(t) => return Ok(t),
                    Err(e) => {
                        tracing::warn!("reconnect failed (retry in {delay}ms): {e}");
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        delay = (delay * 2).min(MAX_DELAY_MS);
                    }
                }
            }
        } else {
            InnerTransport::connect(path).await
        }
    }

    pub async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req = Request::new(method, params);
        let payload = serde_json::to_vec(&WireMessage::Request(req.clone()))
            .map_err(|e| PorpoiseError::Ipc(format!("serialize: {e}")))?;
        let frame = Frame::new(FrameFlags::REQUEST | FrameFlags::ACK, payload);

        let mut guard = self.transport.lock().await;
        match guard.send(&frame).await {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("send failed, reconnecting: {e}");
                *guard = self.reconnect().await?;
                guard.send(&frame).await?;
            }
        }

        let resp_frame = match guard.receive().await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("receive failed, reconnecting: {e}");
                *guard = self.reconnect().await?;
                guard.send(&frame).await?;
                guard.receive().await?
            }
        };

        let wire: WireMessage =
            serde_json::from_slice(&resp_frame.payload).map_err(|e| PorpoiseError::Ipc(format!("deserialize: {e}")))?;

        match wire {
            WireMessage::Response(resp) => {
                if resp.status == crate::message::StatusCode::Ok {
                    Ok(resp.body)
                } else {
                    let msg = resp.error.map(|e| e.message).unwrap_or_else(|| "unknown".into());
                    Err(PorpoiseError::Ipc(msg))
                }
            }
            _ => Err(PorpoiseError::Ipc("unexpected response".into())),
        }
    }
}
