pub mod services;

use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use porpoise_agent::AgentPool;
use porpoise_core::{
    bus::EventBus,
    config::AppConfig,
    error::{PorpoiseError, Result},
    state::AppState,
    types::event::{AgentEvent, NotificationSeverity, SystemEvent, TerminalEvent},
};
use porpoise_db::DbPool;
use porpoise_relay::{RelayServer, Router, WsRelayServer, auth::SessionTokenStore, tls};
use porpoise_runtime::PtyManager;
use tokio::sync::broadcast;

use crate::services::notifications::NotificationService;

/// Truncate a notification body to at most 200 characters, appending "..."
/// when truncation occurs. Character-aware (NOT byte-aware): iterating by char
/// boundary avoids panicking when the cut point falls inside a multibyte UTF-8
/// sequence (emoji / CJK / accented letters). The previous byte-slice
/// `&trimmed[..200]` panicked with `byte index 200 is not a char boundary` in
/// that case (P1-8). See regression `truncate_body_mid_multibyte_no_panic`.
fn truncate_body(input: &str) -> String {
    const MAX_LEN: usize = 200;
    if input.chars().count() > MAX_LEN {
        let mut s: String = input.chars().take(MAX_LEN).collect();
        s.push_str("...");
        s
    } else {
        input.to_string()
    }
}

pub struct Daemon {
    pub state: AppState,
    pub db: DbPool,
    pub server: Option<RelayServer>,
    pub ws_server: Option<WsRelayServer>,
    pub socket_path: PathBuf,
    pub agent_pool: Arc<AgentPool>,
    pub start_time: DateTime<Utc>,
    pub notification_service: Arc<NotificationService>,
    pub generation_id: String,
    pub tls_assets: Option<tls::TlsAssets>,
    session_token: String,
    pidfile_path: Option<PathBuf>,
}

impl Daemon {
    pub async fn new(config: AppConfig, db_path: PathBuf) -> Result<Self> {
        let event_bus = EventBus::new(config.core.event_bus_capacity);
        let state = AppState::new(config.clone(), event_bus.clone());
        let db = DbPool::open(&db_path)?;
        porpoise_db::migration::run_migrations(&db)?;

        let notification_service = Arc::new(NotificationService::new(db.clone()));

        let data_dir = match config.core.data_dir.clone() {
            Some(dir) => dir,
            // ponytail: fall back to the same default the CLI/relay clients use
            // (AppConfig::default_data_dir) instead of env::temp_dir, which
            // diverged from clients and broke IPC (os error 2). Upgrade path:
            // shared const in porpoise-core if more callers appear.
            None => AppConfig::default_data_dir()?,
        };
        let socket_path = data_dir.join("porpoise.sock");

        let token_store = SessionTokenStore::create(&data_dir)?;
        let session_token = token_store.token.clone();

        let agent_pool = Arc::new(AgentPool::new(config.agent.max_concurrent_agents as usize));
        let generation_id = uuid::Uuid::now_v7().to_string();

        Ok(Self {
            state,
            db,
            socket_path,
            agent_pool,
            session_token,
            start_time: Utc::now(),
            server: None,
            ws_server: None,
            notification_service,
            generation_id,
            tls_assets: None,
            pidfile_path: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.write_pidfile()?;

        if let Ok(conn) = self.db.get() {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO server_metadata (key, value) VALUES ('generation_id', ?1)",
                rusqlite::params![&self.generation_id],
            );
            let _ = conn.execute(
                "UPDATE sessions SET ended_at = datetime('now'), status = 'orphaned' WHERE generation_id != ?1 AND (status IS NULL OR status != 'orphaned')",
                rusqlite::params![&self.generation_id],
            );
        }

        let mut router = Router::new(self.state.clone());
        let mut pty = PtyManager::new(self.state.event_bus().clone());
        pty.set_generation_id(Some(self.generation_id.clone()));
        let pty_manager = Arc::new(pty);
        services::register_all(
            &mut router,
            self.start_time,
            pty_manager,
            self.agent_pool.clone(),
            self.state.clone(),
            self.session_token.clone(),
            self.tls_assets.as_ref().map(|a| a.fingerprint_sha256.clone()),
        );
        let router = Arc::new(router);
        let server = RelayServer::bind(&self.socket_path, router.clone(), self.session_token.clone()).await?;
        tracing::info!("porpoise-server listening on {}", self.socket_path.display());
        self.server = Some(server);

        if let Ok(port_str) = std::env::var("PORPOISE_WS_PORT")
            && let Ok(port) = port_str.parse::<u16>()
        {
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            let mut ws_server = WsRelayServer::new(addr, router.clone())
                .with_auth(self.session_token.clone())
                .with_event_bus(self.state.event_bus().clone());

            if std::env::var("PORPOISE_WS_TLS").as_deref() == Ok("1") {
                let data_dir_for_tls = self
                    .state
                    .config()
                    .await
                    .core
                    .data_dir
                    .clone()
                    .unwrap_or_else(std::env::temp_dir);
                let hostnames =
                    std::env::var("PORPOISE_WS_HOSTS").unwrap_or_else(|_| "localhost,127.0.0.1".to_string());
                let host_list: Vec<&str> = hostnames.split(',').map(|s| s.trim()).collect();
                match tls::load_or_generate(&data_dir_for_tls, &host_list) {
                    Ok(assets) => match tls::build_tls_acceptor(&assets) {
                        Ok(acceptor) => {
                            tracing::info!("mobile WSS cert fingerprint: sha256:{}", assets.fingerprint_sha256);
                            self.tls_assets = Some(assets);
                            ws_server = ws_server.with_tls(acceptor);
                        }
                        Err(e) => tracing::warn!("TLS acceptor build failed, serving plain WS: {e}"),
                    },
                    Err(e) => tracing::warn!("TLS asset load/generate failed, serving plain WS: {e}"),
                }
            }

            tracing::info!("mobile WebSocket listening on port {port}");
            self.ws_server = Some(ws_server);
        }

        self.spawn_notification_subscriber();

        let db_for_prune = self.db.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Ok(conn) = db_for_prune.get() {
                    let _ = conn.execute(
                        "DELETE FROM notifications WHERE id IN (SELECT id FROM notifications ORDER BY created_at ASC LIMIT MAX(0, (SELECT COUNT(*) - 500 FROM notifications)))",
                        [],
                    );
                }
            }
        });

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        if let Some(ref server) = self.server {
            if let Some(ref ws) = self.ws_server {
                let ws_fut = ws.run();
                tokio::pin!(ws_fut);
                tokio::select! {
                    result = server.run() => result,
                    _ = &mut ws_fut => Ok(()),
                    _ = Self::shutdown_signal() => {
                        tracing::info!("shutdown signal received");
                        self.shutdown().await;
                        Ok(())
                    }
                }
            } else {
                tokio::select! {
                    result = server.run() => result,
                    _ = Self::shutdown_signal() => {
                        tracing::info!("shutdown signal received");
                        self.shutdown().await;
                        Ok(())
                    }
                }
            }
        } else {
            Self::idle_loop().await
        }
    }

    async fn idle_loop() -> Result<()> {
        tokio::select! {
            _ = Self::shutdown_signal() => {
                tracing::info!("shutdown signal received");
                tracing::info!("shutdown complete");
                std::process::exit(0);
            }
        }
    }

    async fn shutdown(&self) {
        tracing::info!("shutting down");
        self.agent_pool.shutdown_all().await.ok();
        self.cleanup_pidfile();
        tracing::info!("shutdown complete");
        std::process::exit(0);
    }

    fn spawn_notification_subscriber(&self) {
        let svc = self.notification_service.clone();
        let state = self.state.clone();
        let mut rx = self.state.event_bus().subscribe();

        tokio::spawn(async move {
            tracing::info!("notification subscriber started");
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        tracing::trace!(?event, "notification: received event");
                        Daemon::handle_notification_event(&svc, &state, &event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "notification subscriber lagged, skipping old events");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("notification subscriber: event bus closed, exiting");
                        break;
                    }
                }
            }
        });
    }

    async fn handle_notification_event(svc: &NotificationService, state: &AppState, event: &SystemEvent) {
        match event {
            SystemEvent::Terminal(term) => Daemon::handle_terminal_event(svc, term).await,
            SystemEvent::Agent(agent) => Daemon::handle_agent_event(svc, state, agent).await,
            SystemEvent::System(kind) => {
                use porpoise_core::types::event::SystemEventKind;
                if let SystemEventKind::Notification { title, body, severity } = kind
                    && let Err(e) = svc.notify(title, body, severity.clone(), "system").await
                {
                    tracing::warn!(error = %e, "notification: failed to record system notification");
                }
            }
            _ => {}
        }
    }

    async fn handle_terminal_event(svc: &NotificationService, event: &TerminalEvent) {
        if let TerminalEvent::Output { id, data, .. } = event {
            let text = String::from_utf8_lossy(data);
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return;
            }
            let body = truncate_body(trimmed);
            if let Err(e) = svc
                .notify(
                    &format!("Terminal [{id}]"),
                    &body,
                    NotificationSeverity::Info,
                    "terminal",
                )
                .await
            {
                tracing::warn!(error = %e, "notification: terminal output failed");
            }
        } else if let TerminalEvent::Bell { id } = event
            && let Err(e) = svc
                .notify(
                    "Terminal Bell",
                    &format!("Bell in terminal {id}"),
                    NotificationSeverity::Warning,
                    "terminal",
                )
                .await
        {
            tracing::warn!(error = %e, "notification: terminal bell failed");
        }
    }

    async fn handle_agent_event(svc: &NotificationService, state: &AppState, event: &AgentEvent) {
        match event {
            AgentEvent::Output { text, kind, .. } => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return;
                }
                let body = truncate_body(trimmed);
                if let Err(e) = svc
                    .notify(
                        &format!("Agent Output ({kind:?})"),
                        &body,
                        NotificationSeverity::Info,
                        "agent",
                    )
                    .await
                {
                    tracing::warn!(error = %e, "notification: agent output failed");
                }
            }
            AgentEvent::Error { id, message } => {
                let agent_name = state
                    .agents()
                    .await
                    .get(id)
                    .map(|a| a.kind.clone())
                    .unwrap_or_else(|| "agent".to_string());
                if let Err(e) = svc.notify_agent_error(&agent_name, message).await {
                    tracing::warn!(error = %e, "notification: agent error failed");
                }
            }
            AgentEvent::Exited { id, exit_code } => {
                let agents = state.agents().await;
                let agent_name = agents
                    .get(id)
                    .map(|a| a.kind.clone())
                    .unwrap_or_else(|| "agent".to_string());
                let wt_id = agents.get(id).and_then(|a| a.worktree_id);
                drop(agents);

                let worktree_name = match wt_id {
                    Some(wt_id) => state
                        .worktrees()
                        .await
                        .get(&wt_id)
                        .map(|wt| wt.name.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    None => "unknown".to_string(),
                };

                if let Err(e) = svc
                    .notify_agent_completion(&agent_name, &worktree_name, *exit_code)
                    .await
                {
                    tracing::warn!(error = %e, "notification: agent completion failed");
                }
            }
            _ => {}
        }
    }

    async fn shutdown_signal() {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut term = signal(SignalKind::terminate()).expect("sigterm");
            let mut int = signal(SignalKind::interrupt()).expect("sigint");
            tokio::select! {
                _ = term.recv() => {},
                _ = int.recv() => {},
            }
        }
        #[cfg(windows)]
        {
            tokio::signal::ctrl_c().await.ok();
        }
    }

    fn write_pidfile(&mut self) -> Result<()> {
        let path = self.socket_path.with_extension("pid");
        let pid = std::process::id();
        if path.exists() {
            #[cfg(unix)]
            {
                let old_pid = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok());
                if let Some(old) = old_pid {
                    // SAFETY: kill(pid, 0) checks whether a process exists without
                    // sending any signal. `old` is parsed from the pidfile. This is the
                    // POSIX-standard way to test process existence.
                    if unsafe { libc::kill(old as i32, 0) == 0 } {
                        return Err(PorpoiseError::Runtime(format!("daemon already running (PID {old})")));
                    }
                }
            }
        }
        std::fs::write(&path, pid.to_string()).map_err(|e| PorpoiseError::Runtime(format!("write pidfile: {e}")))?;
        self.pidfile_path = Some(path);
        Ok(())
    }

    fn cleanup_pidfile(&self) {
        if let Some(ref path) = self.pidfile_path {
            std::fs::remove_file(path).ok();
        }
    }
}



#[cfg(test)]
mod tests {
    use super::truncate_body;

    /// P1-8 regression: byte 200 must land inside a 4-byte emoji so the OLD
    /// `&s[..200]` byte-slice would panic. 199 ASCII bytes + a 4-byte emoji
    /// (U+1F600) occupy bytes 199..203 -> byte 200 is mid-emoji. The trailing
    /// text makes the body >200 chars so the truncation+ellipsis path is also
    /// exercised. Must NOT panic and must keep whole characters.
    #[test]
    fn truncate_body_mid_multibyte_no_panic() {
        let mut input = String::with_capacity(252);
        input.push_str(&"a".repeat(199)); // bytes 0..199
        input.push('\u{1F600}');         // 4-byte emoji, bytes 199..203
        input.push_str(&"a".repeat(50));  // total 250 chars / 252 bytes
        let body = truncate_body(&input); // OLD code panicked here
        assert!(body.ends_with("..."));
        let content = body.strip_suffix("...").unwrap();
        assert_eq!(content.chars().count(), 200); // 199 'a' + emoji
        // the 4-byte emoji survived whole (never byte-split):
        assert_eq!(content.chars().last(), Some('\u{1F600}'));
    }

    /// Exactly 200 chars: no truncation, body equals input.
    #[test]
    fn truncate_body_at_limit_unchanged() {
        let input = "a".repeat(200);
        let body = truncate_body(&input);
        assert_eq!(body, input);
        assert!(!body.ends_with("..."));
    }

    /// 201 ASCII chars -> truncate content to 200 + "...".
    #[test]
    fn truncate_body_just_over_limit_ascii() {
        let input = "a".repeat(201);
        let body = truncate_body(&input);
        assert!(body.ends_with("..."));
        let content = body.strip_suffix("...").unwrap();
        assert_eq!(content, "a".repeat(200));
        assert_eq!(body.chars().count(), 203); // 200 content + ellipsis
    }

    /// Truncation across 3-byte CJK: byte 200 is mid-char (would panic
    /// previously); char-aware keeps whole characters.
    #[test]
    fn truncate_body_cjk_multibyte_no_panic() {
        let input = "中".repeat(201); // 201 chars / 603 bytes
        let body = truncate_body(&input);
        assert!(body.ends_with("..."));
        let content = body.strip_suffix("...").unwrap();
        assert_eq!(content, "中".repeat(200));
    }

    /// Short / empty input: passthrough.
    #[test]
    fn truncate_body_short_passthrough() {
        assert_eq!(truncate_body("hello"), "hello");
        assert_eq!(truncate_body(""), "");
    }
}
