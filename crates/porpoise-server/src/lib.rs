pub mod services;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use porpoise_agent::AgentPool;
use porpoise_core::{bus::EventBus, config::AppConfig, error::Result, state::AppState};
use porpoise_db::DbPool;
use porpoise_relay::{RelayServer, Router};
use porpoise_runtime::PtyManager;

pub struct Daemon {
    pub state: AppState,
    pub db: DbPool,
    pub server: Option<RelayServer>,
    pub socket_path: std::path::PathBuf,
    pub agent_pool: AgentPool,
    pub start_time: DateTime<Utc>,
    pidfile_path: Option<std::path::PathBuf>,
}

impl Daemon {
    pub async fn new(config: AppConfig, db_path: std::path::PathBuf) -> Result<Self> {
        let event_bus = EventBus::new(config.core.event_bus_capacity);
        let state = AppState::new(config.clone(), event_bus.clone());
        let db = DbPool::open(&db_path)?;
        porpoise_db::migration::run_migrations(&db)?;

        let socket_path = config
            .core
            .data_dir
            .unwrap_or_else(std::env::temp_dir)
            .join("porpoise.sock");

        let agent_pool = AgentPool::new(config.agent.max_concurrent_agents as usize);

        Ok(Self {
            state,
            db,
            socket_path,
            agent_pool,
            start_time: Utc::now(),
            server: None,
            pidfile_path: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.write_pidfile()?;
        let mut router = Router::new(self.state.clone());
        let pty_manager = Arc::new(PtyManager::new(self.state.event_bus().clone()));
        services::register_all(&mut router, self.start_time, pty_manager);
        let router = Arc::new(router);
        let server = RelayServer::bind(&self.socket_path, router).await?;
        tracing::info!("porpoise-server listening on {}", self.socket_path.display());
        self.server = Some(server);
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

    #[allow(dead_code)]
    async fn shutdown(&self) {
        tracing::info!("shutting down");
        self.agent_pool.shutdown_all().await.ok();
        #[cfg(unix)]
        self.cleanup_pidfile();
        tracing::info!("shutdown complete");
        std::process::exit(0);
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
                        return Err(porpoise_core::error::PorpoiseError::Runtime(format!(
                            "daemon already running (PID {old})"
                        )));
                    }
                }
            }
        }
        std::fs::write(&path, pid.to_string())
            .map_err(|e| porpoise_core::error::PorpoiseError::Runtime(format!("write pidfile: {e}")))?;
        self.pidfile_path = Some(path);
        Ok(())
    }

    #[allow(dead_code)]
    fn cleanup_pidfile(&self) {
        if let Some(ref path) = self.pidfile_path {
            std::fs::remove_file(path).ok();
        }
    }
}
