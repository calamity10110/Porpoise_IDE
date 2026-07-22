use std::{path::Path, sync::Arc};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions},
};

use crate::frame::{Frame, HEADER_SIZE};

pub enum NamedPipeStream {
    Client(tokio::net::windows::named_pipe::NamedPipeClient),
    Server(NamedPipeServer),
}

impl NamedPipeStream {
    pub async fn send(&mut self, frame: &Frame) -> Result<()> {
        let data = frame.encode()?;
        match self {
            NamedPipeStream::Client(s) => s.write_all(&data).await,
            NamedPipeStream::Server(s) => s.write_all(&data).await,
        }
        .map_err(|e| PorpoiseError::Ipc(format!("pipe write: {e}")))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Frame> {
        let mut header = vec![0u8; HEADER_SIZE];
        let read_result = match self {
            NamedPipeStream::Client(s) => s.read_exact(&mut header).await,
            NamedPipeStream::Server(s) => s.read_exact(&mut header).await,
        };
        read_result.map_err(|e| PorpoiseError::Ipc(format!("pipe read header: {e}")))?;

        let length = u32::from_le_bytes(
            header[4..8]
                .try_into()
                .map_err(|_| PorpoiseError::Ipc("header slice misaligned".into()))?,
        ) as usize;
        let mut payload = vec![0u8; length];
        if length > 0 {
            let read_result = match self {
                NamedPipeStream::Client(s) => s.read_exact(&mut payload).await,
                NamedPipeStream::Server(s) => s.read_exact(&mut payload).await,
            };
            read_result.map_err(|e| PorpoiseError::Ipc(format!("pipe read payload: {e}")))?;
        }

        let mut frame_data = header;
        frame_data.extend_from_slice(&payload);
        Frame::decode(&frame_data)
    }
}

pub struct NamedPipeTransport {
    stream: NamedPipeStream,
}

impl NamedPipeTransport {
    pub async fn connect(path: &Path) -> Result<Self> {
        let stream = ClientOptions::new()
            .open(path)
            .map_err(|e| PorpoiseError::Ipc(format!("pipe connect: {e}")))?;
        Ok(Self {
            stream: NamedPipeStream::Client(stream),
        })
    }

    pub fn from_server(server: NamedPipeServer) -> Self {
        Self {
            stream: NamedPipeStream::Server(server),
        }
    }

    pub async fn send(&mut self, frame: &Frame) -> Result<()> {
        self.stream.send(frame).await
    }

    pub async fn receive(&mut self) -> Result<Frame> {
        self.stream.receive().await
    }
}

pub struct NamedPipeListener {
    path: String,
}

impl NamedPipeListener {
    pub fn bind(name: &str) -> Self {
        Self {
            path: format!(r"\\.\pipe\{name}"),
        }
    }

    pub async fn accept<F, Fut>(&self, handler: F) -> Result<()>
    where
        F: Fn(NamedPipeTransport) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let handler = Arc::new(handler);
        loop {
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&self.path)
                .map_err(|e| PorpoiseError::Ipc(format!("pipe create: {e}")))?;

            server
                .connect()
                .await
                .map_err(|e| PorpoiseError::Ipc(format!("pipe wait: {e}")))?;

            let transport = NamedPipeTransport::from_server(server);
            let h = handler.clone();
            tokio::spawn(async move {
                h(transport).await;
            });
        }
    }
}
