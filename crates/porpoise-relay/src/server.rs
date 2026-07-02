use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

use crate::frame::{Frame, FrameFlags, HEADER_SIZE, PROTOCOL_VERSION};
use crate::message::{WireMessage, Handshake};
use crate::router::Router;
use porpoise_core::error::{PorpoiseError, Result};

pub struct RelayServer {
    listener: UnixListener,
    router: Arc<Router>,
    _socket_path: PathBuf,
}

impl RelayServer {
    pub async fn bind(path: &PathBuf, router: Arc<Router>) -> Result<Self> {
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
        let listener = UnixListener::bind(path)
            .map_err(|e| PorpoiseError::Ipc(format!("bind: {e}")))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).await
                .map_err(|e| PorpoiseError::Ipc(format!("perms: {e}")))?;
        }
        Ok(Self { listener, router, _socket_path: path.clone() })
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((mut stream, _)) => {
                    let router = self.router.clone();
                    tokio::spawn(async move {
                        // Send handshake on connect
                        let handshake = WireMessage::Handshake(Handshake::new());
                        if let Ok(payload) = serde_json::to_vec(&handshake) {
                            let _ = write_frame(&mut stream, &Frame::new(FrameFlags::EVENT, payload)).await;
                        }

                        loop {
                            match read_frame(&mut stream).await {
                                Ok(frame) => {
                                    if !frame.flags.contains(FrameFlags::REQUEST) { continue; }
                                    if let Ok(WireMessage::Request(req)) = serde_json::from_slice(&frame.payload) {
                                        let resp = router.dispatch(req).await;
                                        let payload = serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                                        let _ = write_frame(&mut stream, &Frame::new(FrameFlags::RESPONSE, payload)).await;
                                    }
                                }
                                Err(e) => { tracing::warn!("client: {e}"); break; }
                            }
                        }
                    });
                }
                Err(e) => tracing::error!("accept: {e}"),
            }
        }
    }
}

async fn read_frame(stream: &mut tokio::net::UnixStream) -> Result<Frame> {
    let mut header = vec![0u8; HEADER_SIZE];
    stream.read_exact(&mut header).await.map_err(|e| PorpoiseError::Ipc(format!("header: {e}")))?;
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let mut payload = vec![0u8; len];
    if len > 0 { stream.read_exact(&mut payload).await.map_err(|e| PorpoiseError::Ipc(format!("payload: {e}")))?; }
    let mut data = header; data.extend(payload);
    Frame::decode(&data)
}

async fn write_frame(stream: &mut tokio::net::UnixStream, frame: &Frame) -> Result<()> {
    stream.write_all(&frame.encode()?).await.map_err(|e| PorpoiseError::Ipc(format!("write: {e}")))?;
    Ok(())
}
