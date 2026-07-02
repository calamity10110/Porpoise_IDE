use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::frame::{Frame, FrameFlags, PROTOCOL_MAGIC};
use crate::message::{Request, WireMessage};
use crate::transport::unix::UnixSocketTransport;
use porpoise_core::error::{PorpoiseError, Result};

const BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 30_000;

pub struct RelayClient {
    socket_path: PathBuf,
    transport: Mutex<UnixSocketTransport>,
}

impl RelayClient {
    pub async fn connect(path: &PathBuf) -> Result<Self> {
        let transport = Self::connect_with_retry(path, false).await?;
        Ok(Self { socket_path: path.clone(), transport: Mutex::new(transport) })
    }

    async fn connect_with_retry(path: &PathBuf, retry: bool) -> Result<UnixSocketTransport> {
        if retry {
            let mut delay = BASE_DELAY_MS;
            loop {
                match UnixSocketTransport::connect(path).await {
                    Ok(t) => return Ok(t),
                    Err(e) => {
                        tracing::warn!("reconnect failed (retry in {delay}ms): {e}");
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        delay = (delay * 2).min(MAX_DELAY_MS);
                    }
                }
            }
        } else {
            UnixSocketTransport::connect(path).await
        }
    }

    pub async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req = Request::new(method, params);
        let payload = serde_json::to_vec(&WireMessage::Request(req.clone()))
            .map_err(|e| PorpoiseError::Ipc(format!("serialize: {e}")))?;
        let frame = Frame::new(FrameFlags::REQUEST | FrameFlags::ACK, payload);

        let mut guard = self.transport.lock().await;
        match guard.send(&frame).await {
            Ok(_) => {},
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

        let wire: WireMessage = serde_json::from_slice(&resp_frame.payload)
            .map_err(|e| PorpoiseError::Ipc(format!("deserialize: {e}")))?;

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
