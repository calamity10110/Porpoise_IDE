use std::{path::PathBuf, sync::Arc};

use porpoise_core::error::Result;

use crate::{
    frame::{Frame, FrameFlags},
    message::{Handshake, WireMessage},
    router::Router,
};

#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use crate::transport::pipe::NamedPipeListener;

pub struct RelayServer {
    router: Arc<Router>,
    #[cfg(unix)]
    listener: UnixListener,
    #[cfg(windows)]
    listener: NamedPipeListener,
    _socket_path: PathBuf,
}

impl RelayServer {
    #[cfg(unix)]
    pub async fn bind(path: &PathBuf, router: Arc<Router>) -> Result<Self> {
        if path.exists() {
            tokio::fs::remove_file(path).await.ok();
        }
        let listener = UnixListener::bind(path).map_err(|e| PorpoiseError::Ipc(format!("bind: {e}")))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
                .await
                .map_err(|e| PorpoiseError::Ipc(format!("perms: {e}")))?;
        }
        Ok(Self { listener, router, _socket_path: path.clone() })
    }

    #[cfg(windows)]
    pub async fn bind(path: &PathBuf, router: Arc<Router>) -> Result<Self> {
        let pipe_name = path.to_str().unwrap_or("porpoise");
        let listener = NamedPipeListener::bind(pipe_name);
        Ok(Self { listener, router, _socket_path: path.clone() })
    }

    pub async fn run(&self) -> Result<()> {
        #[cfg(unix)]
        { self.run_unix().await }
        #[cfg(windows)]
        { self.run_windows().await }
    }

    #[cfg(unix)]
    async fn run_unix(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let router = self.router.clone();
                    tokio::spawn(async move { handle_unix(stream, router).await });
                }
                Err(e) => tracing::error!("accept: {e}"),
            }
        }
    }

    #[cfg(windows)]
    async fn run_windows(&self) -> Result<()> {
        let router = self.router.clone();
        self.listener
            .accept(move |transport| {
                let router = router.clone();
                async move { handle_windows(transport, router).await }
            })
            .await
    }
}

#[cfg(unix)]
async fn handle_unix(mut stream: UnixStream, router: Arc<Router>) {
    use porpoise_core::error::PorpoiseError;
    let handshake = WireMessage::Handshake(Handshake::new());
    if let Ok(payload) = serde_json::to_vec(&handshake) {
        let _ = write_frame_unix(&mut stream, &Frame::new(FrameFlags::EVENT, payload)).await;
    }

    loop {
        match read_frame_unix(&mut stream).await {
            Ok(frame) => {
                if !frame.flags.contains(FrameFlags::REQUEST) {
                    continue;
                }
                if let Ok(WireMessage::Request(req)) = serde_json::from_slice(&frame.payload) {
                    let resp = router.dispatch(req).await;
                    let payload = serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                    let _ = write_frame_unix(&mut stream, &Frame::new(FrameFlags::RESPONSE, payload)).await;
                }
            }
            Err(e) => {
                tracing::warn!("client: {e}");
                break;
            }
        }
    }
}

#[cfg(unix)]
async fn read_frame_unix(stream: &mut UnixStream) -> Result<Frame> {
    use crate::frame::HEADER_SIZE;
    use porpoise_core::error::PorpoiseError;
    use tokio::io::AsyncReadExt;
    let mut header = vec![0u8; HEADER_SIZE];
    stream.read_exact(&mut header).await.map_err(|e| PorpoiseError::Ipc(format!("header: {e}")))?;
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let mut payload = vec![0u8; len];
    if len > 0 {
        stream.read_exact(&mut payload).await.map_err(|e| PorpoiseError::Ipc(format!("payload: {e}")))?;
    }
    let mut data = header;
    data.extend(payload);
    Frame::decode(&data)
}

#[cfg(unix)]
async fn write_frame_unix(stream: &mut UnixStream, frame: &Frame) -> Result<()> {
    use porpoise_core::error::PorpoiseError;
    use tokio::io::AsyncWriteExt;
    stream.write_all(&frame.encode()?).await.map_err(|e| PorpoiseError::Ipc(format!("write: {e}")))?;
    Ok(())
}

#[cfg(windows)]
async fn handle_windows(transport: crate::transport::pipe::NamedPipeTransport, router: Arc<Router>) {
    let mut stream = transport;

    let handshake = WireMessage::Handshake(Handshake::new());
    if let Ok(payload) = serde_json::to_vec(&handshake) {
        let _ = stream.send(&Frame::new(FrameFlags::EVENT, payload)).await;
    }

    loop {
        match stream.receive().await {
            Ok(frame) => {
                if !frame.flags.contains(FrameFlags::REQUEST) {
                    continue;
                }
                if let Ok(WireMessage::Request(req)) = serde_json::from_slice(&frame.payload) {
                    let resp = router.dispatch(req).await;
                    let payload = serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                    let _ = stream.send(&Frame::new(FrameFlags::RESPONSE, payload)).await;
                }
            }
            Err(e) => {
                tracing::warn!("client: {e}");
                break;
            }
        }
    }
}
