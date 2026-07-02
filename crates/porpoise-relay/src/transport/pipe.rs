use std::path::Path;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::frame::{Frame, HEADER_SIZE};
use porpoise_core::error::{PorpoiseError, Result};

pub struct NamedPipeTransport {
    stream: tokio::net::windows::named_pipe::NamedPipeClient,
}

impl NamedPipeTransport {
    pub async fn connect(path: &Path) -> Result<Self> {
        let stream = ClientOptions::new()
            .open(path)
            .map_err(|e| PorpoiseError::Ipc(format!("pipe connect: {e}")))?;
        Ok(Self { stream })
    }

    pub async fn send(&mut self, frame: &Frame) -> Result<()> {
        let data = frame.encode()?;
        self.stream.write_all(&data).await
            .map_err(|e| PorpoiseError::Ipc(format!("pipe write: {e}")))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Frame> {
        let mut header = vec![0u8; HEADER_SIZE];
        self.stream.read_exact(&mut header).await
            .map_err(|e| PorpoiseError::Ipc(format!("pipe read header: {e}")))?;

        let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        let mut payload = vec![0u8; length];
        if length > 0 {
            self.stream.read_exact(&mut payload).await
                .map_err(|e| PorpoiseError::Ipc(format!("pipe read payload: {e}")))?;
        }

        let mut frame_data = header;
        frame_data.extend_from_slice(&payload);
        Frame::decode(&frame_data)
    }
}
