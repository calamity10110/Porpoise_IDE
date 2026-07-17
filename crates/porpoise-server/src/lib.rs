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
use porpoise_relay::{auth::SessionTokenStore, RelayServer, Router};
use porpoise_runtime::PtyManager;
use tokio::sync::broadcast;

use crate::services::notifications::NotificationService;

pub struct Daemon {
    pub state: AppState,
    pub db: DbPool,
    pub server: Option<RelayServer>,
    pub socket_path: PathBuf,
    pub agent_pool: Arc<AgentPool>,
    pub start_time: DateTime<Utc>,
    pub notification_service: Arc<NotificationService>,
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

        let data_dir = config
            .core
            .data_dir
            .clone()
            .unwrap_or_else(std::env::temp_dir);
        let socket_path = data_dir.join("porpoise.sock");

        let token_store = SessionTokenStore::create(&data_dir)?;
        let session_token = token_store.token.clone();

        let agent_pool = Arc::new(AgentPool::new(config.agent.max_concurrent_agents as usize));

        Ok(Self {
            state,
            db,
            socket_path,
            agent_pool,
            session_token,
            start_time: Utc::now(),
            server: None,
            notification_service,
            pidfile_path: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.write_pidfile()?;
        let mut router = Router::new(self.state.clone());
        let pty_manager = Arc::new(PtyManager::new(self.state.event_bus().clone()));
        services::register_all(
            &mut router,
            self.start_time,
            pty_manager,
            self.agent_pool.clone(),
            self.state.clone(),
        );
        let router = Arc::new(router);
        let server = RelayServer::bind(&self.socket_path, router, self.session_token.clone()).await?;
        tracing::info!("porpoise-server listening on {}", self.socket_path.display());
        self.server = Some(server);

        self.spawn_notification_subscriber();

        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        if let Some(ref server) = self.server {
            tokio::select! {
                result = server.run() => result,
                _ = Self::shutdown_signal() => {
                    tracing::info!("shutdown signal received");
                    self.shutdown().await;
                    Ok(())
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
                        tracing::info!(?event, "notification: received event");
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
            let body = if trimmed.len() > 200 {
                format!("{}...", &trimmed[..200])
            } else {
                trimmed.to_string()
            };
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
                let body = if trimmed.len() > 200 {
                    format!("{}...", &trimmed[..200])
                } else {
                    trimmed.to_string()
                };
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
