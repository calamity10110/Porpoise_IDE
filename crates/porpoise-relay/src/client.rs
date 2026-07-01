use std::path::PathBuf;
use tokio::sync::Mutex;

use crate::frame::{Frame, FrameFlags};
use crate::message::{Request, WireMessage};
use crate::transport::unix::UnixSocketTransport;
use porpoise_core::error::{PorpoiseError, Result};

pub struct RelayClient {
    transport: Mutex<UnixSocketTransport>,
}

impl RelayClient {
    pub async fn connect(path: &PathBuf) -> Result<Self> {
        Ok(Self { transport: Mutex::new(UnixSocketTransport::connect(path).await?) })
    }

    pub async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req = Request::new(method, params);
        let payload = serde_json::to_vec(&WireMessage::Request(req.clone()))
            .map_err(|e| PorpoiseError::Ipc(format!("serialize: {e}")))?;
        let frame = Frame::new(FrameFlags::REQUEST | FrameFlags::ACK, payload);

        let mut guard = self.transport.lock().await;
        guard.send(&frame).await?;
        let resp_frame = guard.receive().await?;

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
