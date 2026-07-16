use std::{net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use porpoise_core::{
    bus::EventBus,
    error::{PorpoiseError, Result},
};
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
    auth_token: Option<String>,
    event_bus: Option<EventBus>,
}

impl WsRelayServer {
    pub fn new(addr: SocketAddr, router: Arc<Router>) -> Self {
        Self { router, addr, auth_token: None, event_bus: None }
    }

    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
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
                    let auth_token = self.auth_token.clone();
                    let event_bus = self.event_bus.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_ws(stream, peer, router, auth_token, event_bus).await {
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
    auth_token: Option<String>,
    event_bus: Option<EventBus>,
) -> Result<()> {
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| PorpoiseError::Ipc(format!("ws upgrade: {e}")))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let handshake = WireMessage::Handshake(Handshake::new());
    if let Ok(payload) = serde_json::to_vec(&handshake) {
        let frame = Frame::new(FrameFlags::EVENT, payload);
        #[allow(clippy::collapsible_if)]
        if let Ok(encoded) = frame.encode() {
            if let Ok(msg) = serde_json::to_string(&encoded) {
                let _ = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await;
            }
        }
    }

    let mut event_rx = event_bus.as_ref().map(|bus| bus.subscribe());

    loop {
        tokio::select! {
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        if let Some(ref expected_token) = auth_token
                            && text.trim() != expected_token.as_str()
                        {
                            tracing::warn!("ws client {peer}: auth failed");
                            let _ = ws_sender.send(
                                tokio_tungstenite::tungstenite::Message::Text(
                                    r#"{"error":"auth_required"}"#.into()
                                )
                            ).await;
                            break;
                        }

                        let frame = match serde_json::from_str::<Vec<u8>>(&text) {
                            Ok(bytes) => Frame::decode(&bytes),
                            Err(_) => Frame::decode(text.as_bytes()),
                        };
                        if let Ok(frame) = frame {
                            if !frame.flags.contains(FrameFlags::REQUEST) {
                                continue;
                            }
                            if let Ok(WireMessage::Request(req)) = serde_json::from_slice(&frame.payload) {
                                let resp = router.dispatch(req).await;
                                let payload = serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                                let resp_frame = Frame::new(FrameFlags::RESPONSE, payload);
                                let Ok(encoded) = resp_frame.encode() else { continue; };
                                let Ok(msg) = serde_json::to_string(&encoded) else { continue; };
                                let _ = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await;
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
            event = async {
                match event_rx.as_mut() {
                    Some(rx) => rx.recv().await.ok(),
                    None => std::future::pending().await,
                }
            } => {
                if let Some(system_event) = event {
                    let Ok(payload) = serde_json::to_vec(&system_event) else { continue; };
                    let frame = Frame::new(FrameFlags::EVENT, payload);
                    let Ok(encoded) = frame.encode() else { continue; };
                    let Ok(msg) = serde_json::to_string(&encoded) else { continue; };
                    let _ = ws_sender.send(tokio_tungstenite::tungstenite::Message::Text(msg)).await;
                }
            }
        }
    }

    Ok(())
}
