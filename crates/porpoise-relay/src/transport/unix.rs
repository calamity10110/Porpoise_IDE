use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::frame::{Frame, HEADER_SIZE};
use porpoise_core::error::{PorpoiseError, Result};

pub struct UnixSocketTransport {
    pub stream: UnixStream,
}

impl UnixSocketTransport {
    pub async fn connect(path: &std::path::Path) -> Result<Self> {
        let stream = UnixStream::connect(path).await
            .map_err(|e| PorpoiseError::Ipc(format!("connect: {e}")))?;
        Ok(Self { stream })
    }

    pub async fn send(&mut self, frame: &Frame) -> Result<()> {
        self.stream.write_all(&frame.encode()?).await
            .map_err(|e| PorpoiseError::Ipc(format!("write: {e}")))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Frame> {
        let mut header = vec![0u8; HEADER_SIZE];
        self.stream.read_exact(&mut header).await
            .map_err(|e| PorpoiseError::Ipc(format!("header: {e}")))?;
        let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let mut payload = vec![0u8; len];
        if len > 0 {
            self.stream.read_exact(&mut payload).await
                .map_err(|e| PorpoiseError::Ipc(format!("payload: {e}")))?;
        }
        let mut data = header;
        data.extend(payload);
        Frame::decode(&data)
    }
}
