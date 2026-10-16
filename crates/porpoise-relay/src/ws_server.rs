use std::{net::SocketAddr, sync::Arc};

use futures_util::{Sink, SinkExt, StreamExt};
use porpoise_core::{
    bus::EventBus,
    error::{PorpoiseError, Result},
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{
    frame::{Frame, FrameFlags},
    message::{Handshake, Request, WireMessage},
    router::Router,
};

pub struct WsRelayServer {
    listener: TcpListener,
    router: Arc<Router>,
    auth_token: Option<String>,
    event_bus: Option<EventBus>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl WsRelayServer {
    /// Bind the relay listener. Fails fast when the port is unavailable so
    /// callers surface the conflict at startup instead of at first `run()`.
    pub async fn bind(addr: SocketAddr, router: Arc<Router>) -> Result<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| PorpoiseError::Ipc(format!("ws bind: {e}")))?;
        Ok(Self {
            listener,
            router,
            auth_token: None,
            event_bus: None,
            tls_acceptor: None,
        })
    }

    /// The bound address; useful for tests and port-0 listeners.
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn with_tls(mut self, acceptor: TlsAcceptor) -> Self {
        self.tls_acceptor = Some(acceptor);
        self
    }

    pub async fn run(&self) -> Result<()> {
        let scheme = if self.tls_acceptor.is_some() { "wss" } else { "ws" };
        let addr = self
            .local_addr()
            .map_err(|e| PorpoiseError::Ipc(format!("ws local addr: {e}")))?;
        tracing::info!("WebSocket relay listening on {scheme}://{addr}");

        loop {
            match self.listener.accept().await {
                Ok((stream, peer)) => {
                    let router = self.router.clone();
                    let auth_token = self.auth_token.clone();
                    let event_bus = self.event_bus.clone();
                    let tls_acceptor = self.tls_acceptor.clone();
                    tokio::spawn(async move {
                        if let Some(tls) = tls_acceptor {
                            match tls.accept(stream).await {
                                Ok(tls_stream) => {
                                    if let Err(e) = handle_ws(tls_stream, peer, router, auth_token, event_bus).await {
                                        tracing::warn!("wss client {peer}: {e}");
                                    }
                                }
                                Err(e) => tracing::warn!("tls handshake from {peer}: {e}"),
                            }
                        } else if let Err(e) = handle_ws(stream, peer, router, auth_token, event_bus).await {
                            tracing::warn!("ws client {peer}: {e}");
                        }
                    });
                }
                Err(e) => tracing::error!("ws accept: {e}"),
            }
        }
    }
}

async fn handle_ws<S>(
    stream: S,
    peer: SocketAddr,
    router: Arc<Router>,
    auth_token: Option<String>,
    event_bus: Option<EventBus>,
) -> Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| PorpoiseError::Ipc(format!("ws upgrade: {e}")))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let handshake = WireMessage::Handshake(Handshake::new());
    if let Ok(payload) = serde_json::to_vec(&handshake) {
        let frame = Frame::new(FrameFlags::EVENT, payload);
        if let Ok(encoded) = frame.encode()
            && let Ok(msg) = serde_json::to_string(&encoded)
        {
            let _ = ws_sender.send(Message::Text(msg)).await;
        }
    }

    let mut event_rx = event_bus.as_ref().map(|bus| bus.subscribe());
    let mut authenticated = auth_token.is_none();

    loop {
        tokio::select! {
            ws_msg = ws_receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        if !authenticated {
                            let accepted = auth_token
                                .as_deref()
                                .is_some_and(|expected| token_matches(&text, expected));
                            if !accepted {
                                tracing::warn!("ws client {peer}: auth failed");
                                let _ = ws_sender
                                    .send(Message::Text(r#"{"error":"auth_required"}"#.into()))
                                    .await;
                                let _ = ws_sender.send(Message::Close(None)).await;
                                break;
                            }
                            authenticated = true;
                            let _ = ws_sender
                                .send(Message::Text(r#"{"type":"auth","ok":true}"#.into()))
                                .await;
                            continue;
                        }

                        if dispatch_plain_json(&text, &router, &mut ws_sender).await {
                            continue;
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
                                let payload =
                                    serde_json::to_vec(&WireMessage::Response(resp)).unwrap_or_default();
                                let resp_frame = Frame::new(FrameFlags::RESPONSE, payload);
                                let Ok(encoded) = resp_frame.encode() else { continue; };
                                let Ok(msg) = serde_json::to_string(&encoded) else { continue; };
                                let _ = ws_sender.send(Message::Text(msg)).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
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
                    let _ = ws_sender.send(Message::Text(msg)).await;
                }
            }
        }
    }

    Ok(())
}

/// First-message authentication: accept the bare token, or the mobile
/// companion's `{"type":"auth","token":"…"}` JSON. Anything else fails closed.
fn token_matches(text: &str, expected: &str) -> bool {
    let trimmed = text.trim();
    if trimmed == expected {
        return true;
    }
    serde_json::from_str::<serde_json::Value>(trimmed)
        .ok()
        .is_some_and(|value| {
            value.get("type").and_then(serde_json::Value::as_str) == Some("auth")
                && value.get("token").and_then(serde_json::Value::as_str) == Some(expected)
        })
}

/// Dispatch a mobile-companion plain-JSON RPC — `{"method","params","id"}` —
/// and reply in kind (`{"id","status","body"[,"error"]}`), echoing the
/// caller's `id`. Returns false when the message is not that shape, leaving
/// it to the framed desktop protocol.
async fn dispatch_plain_json<S>(text: &str, router: &Router, sender: &mut S) -> bool
where
    S: Sink<Message> + Unpin,
{
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return false;
    };
    let Some(method) = value.get("method").and_then(serde_json::Value::as_str) else {
        return false;
    };
    let params = value.get("params").cloned().unwrap_or(serde_json::Value::Null);
    let id = value.get("id").cloned().unwrap_or(serde_json::Value::Null);

    let response = router.dispatch(Request::new(method, params)).await;
    let mut reply = serde_json::json!({ "id": id, "status": response.status, "body": response.body });
    if let Some(err) = response.error
        && let Ok(encoded) = serde_json::to_value(&err)
    {
        reply["error"] = encoded;
    }
    if let Ok(msg) = serde_json::to_string(&reply) {
        let _ = sender.send(Message::Text(msg)).await;
    }
    true
}

#[cfg(test)]
mod tests {
    use porpoise_core::{config::AppConfig, state::AppState};

    use super::*;

    const TOKEN: &str = "test-token-1234";

    type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

    async fn spawn_server(with_auth: bool) -> SocketAddr {
        let state = AppState::new(AppConfig::default(), EventBus::new(16));
        let mut router = Router::new(state);
        router.register("test/echo", Arc::new(|req, _| Box::pin(async move { Ok(req.params) })));
        let server = WsRelayServer::bind("127.0.0.1:0".parse().unwrap(), Arc::new(router))
            .await
            .unwrap();
        let addr = server.local_addr().unwrap();
        let server = if with_auth {
            server.with_auth(TOKEN.into())
        } else {
            server
        };
        tokio::spawn(async move { server.run().await });
        addr
    }

    async fn connect(addr: SocketAddr) -> WsStream {
        let (stream, _) = tokio_tungstenite::connect_async(format!("ws://{addr}")).await.unwrap();
        stream
    }

    async fn next_text(stream: &mut WsStream) -> String {
        match stream.next().await {
            Some(Ok(Message::Text(text))) => text,
            other => panic!("expected a text message, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn json_auth_then_plain_json_rpc_round_trips() {
        let addr = spawn_server(true).await;
        let mut stream = connect(addr).await;
        let _handshake = next_text(&mut stream).await;

        stream
            .send(Message::Text(r#"{"type":"auth","token":"test-token-1234"}"#.into()))
            .await
            .unwrap();
        assert_eq!(next_text(&mut stream).await, r#"{"type":"auth","ok":true}"#);

        stream
            .send(Message::Text(
                r#"{"method":"test/echo","params":{"x":7},"id":"id-1"}"#.into(),
            ))
            .await
            .unwrap();
        let reply: serde_json::Value = serde_json::from_str(&next_text(&mut stream).await).unwrap();
        assert_eq!(reply["id"], "id-1");
        assert_eq!(reply["status"], "Ok");
        assert_eq!(reply["body"]["x"], 7);
    }

    #[tokio::test]
    async fn bare_token_auth_is_still_accepted() {
        let addr = spawn_server(true).await;
        let mut stream = connect(addr).await;
        let _handshake = next_text(&mut stream).await;

        stream.send(Message::Text(TOKEN.into())).await.unwrap();
        assert_eq!(next_text(&mut stream).await, r#"{"type":"auth","ok":true}"#);
    }

    #[tokio::test]
    async fn wrong_token_is_rejected_and_connection_closed() {
        let addr = spawn_server(true).await;
        let mut stream = connect(addr).await;
        let _handshake = next_text(&mut stream).await;

        stream
            .send(Message::Text(r#"{"type":"auth","token":"wrong"}"#.into()))
            .await
            .unwrap();
        assert_eq!(next_text(&mut stream).await, r#"{"error":"auth_required"}"#);
        match stream.next().await {
            Some(Ok(Message::Close(_))) | None => {}
            other => panic!("expected the connection to close, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unauthenticated_server_accepts_rpc_without_auth() {
        let addr = spawn_server(false).await;
        let mut stream = connect(addr).await;
        let _handshake = next_text(&mut stream).await;

        stream
            .send(Message::Text(
                r#"{"method":"test/echo","params":{"ok":true},"id":"id-2"}"#.into(),
            ))
            .await
            .unwrap();
        let reply: serde_json::Value = serde_json::from_str(&next_text(&mut stream).await).unwrap();
        assert_eq!(reply["id"], "id-2");
        assert_eq!(reply["body"]["ok"], true);
    }
}
