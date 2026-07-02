use std::time::Duration;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use porpoise_core::error::{PorpoiseError, Result};

pub struct WsClient;

impl WsClient {
    pub async fn connect(url: &str) -> Result<WsSession> {
        let (stream, _) = connect_async(url).await
            .map_err(|e| PorpoiseError::Network(format!("ws connect {url}: {e}")))?;
        Ok(WsSession { stream })
    }
}

pub struct WsSession {
    stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
}

impl WsSession {
    pub async fn send(&mut self, text: &str) -> Result<()> {
        self.stream.send(Message::Text(text.into())).await
            .map_err(|e| PorpoiseError::Network(format!("ws send: {e}")))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<String>> {
        match tokio::time::timeout(Duration::from_secs(30), self.stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => Ok(Some(text)),
            Ok(Some(Ok(Message::Close(_)))) => Ok(None),
            Ok(Some(Err(e))) => Err(PorpoiseError::Network(format!("ws recv: {e}"))),
            Ok(None) => Ok(None),
            Ok(Some(Ok(_))) => Ok(None),
            Err(_) => Err(PorpoiseError::Timeout(30_000)),
        }
    }

    pub async fn close(mut self) -> Result<()> {
        self.stream.close(None).await
            .map_err(|e| PorpoiseError::Network(format!("ws close: {e}")))?;
        Ok(())
    }
}
