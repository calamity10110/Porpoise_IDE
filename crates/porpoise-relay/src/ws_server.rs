use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use porpoise_core::error::{PorpoiseError, Result};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::{
    frame::{Frame, FrameFlags},
    message::{Handshake, WireMessage},
    router::Router,
};

pub struct WsRelayServer {
    router: Arc<Router>,
    addr: SocketAddr,
}

impl WsRelayServer {
    pub fn new(addr: SocketAddr, router: Arc<Router>) -> Self {
        Self { router, addr }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| PorpoiseError::Ipc(format!("ws bind: {e}")))?;
        tracing::info!("WebSocket relay listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let router = self.router.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_ws(stream, peer, router).await {
                            tracing::warn!("ws client {peer}: {e}");
                        }
                    });
                }
                Err(e) => tracing::error!("ws accept: {e}"),
            }
        }
    }
}

async fn handle_ws(
    stream: tokio::net::TcpStream,
    peer: SocketAddr,
    router: Arc<Router>,
) -> Result<()> {
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| PorpoiseError::Ipc(format!("ws upgrade: {e}")))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let handshake = WireMessage::Handshake(Handshake::new());
    if let Ok(payload) = serde_json::to_vec(&handshake) {
        let frame = Frame::new(FrameFlags::EVENT, payload);
        let msg = serde_json::to_string(&frame.encode().map_err(|e| PorpoiseError::Ipc(format!("encode: {e}")))?)
            .map_err(|e| PorpoiseError::Ipc(format!("serialize: {e}")))?;
        let _ = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await;
    }

    loop {
        match ws_receiver.next().await {
            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                let frame = match serde_json::from_str::<Vec<u8>>(&text) {
                    Ok(bytes) => Frame::decode(&bytes),
                    Err(_) => Frame::decode(text.as_bytes()),
                };
                match frame {
                    Ok(frame) => {
                        if !frame.flags.contains(FrameFlags::REQUEST) {
                            continue;
                        }
                        if let Ok(WireMessage::Request(req)) = serde_json::from_slice(&frame.payload) {
                            let resp = router.dispatch(req).await;
                            let payload = serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                            let resp_frame = Frame::new(FrameFlags::RESPONSE, payload);
                            let encoded = resp_frame.encode().map_err(|e| PorpoiseError::Ipc(format!("encode: {e}")))?;
                            if let Ok(msg) = serde_json::to_string(&encoded) {
                                let _ = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("ws frame decode: {e}");
                    }
                }
            }
            Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => break,
            Some(Err(e)) => {
                tracing::warn!("ws error from {peer}: {e}");
                break;
            }
            None => break,
            _ => {}
        }
    }

    Ok(())
}
