use std::{path::PathBuf, time::Duration};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::sync::Mutex;

use crate::{
    frame::{Frame, FrameFlags},
    message::{Handshake, Request, WireMessage},
};

#[cfg(unix)]
use crate::transport::unix::UnixSocketTransport;
#[cfg(windows)]
use crate::transport::pipe::NamedPipeTransport;

const BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 30_000;

#[cfg(unix)]
type InnerTransport = UnixSocketTransport;
#[cfg(windows)]
type InnerTransport = NamedPipeTransport;

pub struct RelayClient {
    socket_path: PathBuf,
    transport: Mutex<InnerTransport>,
}

impl RelayClient {
    pub async fn connect(path: &std::path::Path) -> Result<Self> {
        let transport = Self::connect_with_retry(path, false).await?;
        Ok(Self {
            socket_path: path.to_path_buf(),
            transport: Mutex::new(transport),
        })
    }

    pub async fn connect_with_auth(path: &std::path::Path, token: &str) -> Result<Self> {
        let mut transport = Self::connect_with_retry(path, false).await?;

        // Read server's handshake first (server sends it on accept)
        let _frame = transport
            .receive()
            .await
            .map_err(|e| PorpoiseError::Ipc(format!("read handshake: {e}")))?;

        // Respond with our handshake containing the auth token
        let auth_hs = WireMessage::Handshake(Handshake {
            version: crate::frame::PROTOCOL_VERSION,
            min_version: crate::frame::MIN_PROTOCOL_VERSION,
            server_name: "porpoise-cli".into(),
            session_token: Some(token.to_string()),
            peer_pid: std::process::id(),
        });
        let payload = serde_json::to_vec(&auth_hs)
            .map_err(|e| PorpoiseError::Ipc(format!("serialize auth: {e}")))?;
        transport
            .send(&Frame::new(FrameFlags::EVENT, payload))
            .await?;

        Ok(Self {
            socket_path: path.to_path_buf(),
            transport: Mutex::new(transport),
        })
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
                *guard = Self::connect_with_retry(&self.socket_path, true).await?;
                guard.send(&frame).await?;
            }
        }

        let resp_frame = match guard.receive().await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("receive failed, reconnecting: {e}");
                *guard = Self::connect_with_retry(&self.socket_path, true).await?;
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
