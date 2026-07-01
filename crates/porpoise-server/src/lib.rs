pub mod services;

use std::path::PathBuf;
#[cfg(unix)]
use std::sync::Arc;
use porpoise_core::bus::EventBus;
use porpoise_core::config::AppConfig;
use porpoise_core::state::AppState;
use porpoise_core::error::Result;
use porpoise_db::DbPool;

#[cfg(unix)]
use porpoise_relay::{RelayServer, Router};

pub struct Daemon {
    pub state: AppState,
    pub db: DbPool,
    #[cfg(unix)]
    pub server: Option<RelayServer>,
    pub socket_path: PathBuf,
}

impl Daemon {
    pub async fn new(config: AppConfig, db_path: PathBuf) -> Result<Self> {
        let event_bus = EventBus::new(config.core.event_bus_capacity);
        let state = AppState::new(config.clone(), event_bus.clone());
        let db = DbPool::open(&db_path)?;
        porpoise_db::migration::run_migrations(&db)?;

        let socket_path = config.core.data_dir
            .unwrap_or_else(|| std::env::temp_dir())
            .join("porpoise.sock");

        Ok(Self {
            state,
            db,
            socket_path,
        })
    }

    #[cfg(unix)]
    pub async fn start(&mut self) -> Result<()> {
        let router = Arc::new(Router::new(self.state.clone()));
        let server = RelayServer::bind(&self.socket_path, router).await?;
        tracing::info!("porpoise-server listening on {}", self.socket_path.display());
        self.server = Some(server);
        Ok(())
    }

    #[cfg(not(unix))]
    pub async fn start(&mut self) -> Result<()> {
        tracing::warn!("IPC not available on this platform");
        Ok(())
    }

    #[cfg(unix)]
    pub async fn run(&self) -> Result<()> {
        if let Some(ref server) = self.server {
            server.run().await?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub async fn run(&self) -> Result<()> {
        tracing::info!("porpoise-server running (no IPC)");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
