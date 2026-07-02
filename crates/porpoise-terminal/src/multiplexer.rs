use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use porpoise_core::bus::EventBus;
use porpoise_core::error::Result;
use porpoise_core::types::id::{TerminalId, SessionId};
use porpoise_runtime::PtyManager;

pub struct PtyMultiplexer {
    pty_manager: Arc<PtyManager>,
    sessions: Arc<RwLock<HashMap<SessionId, SessionInfo>>>,
    #[allow(dead_code)]
    event_bus: EventBus,
}

struct SessionInfo {
    terminals: Vec<TerminalId>,
}

impl PtyMultiplexer {
    pub fn new(pty_manager: Arc<PtyManager>, event_bus: EventBus) -> Self {
        Self { pty_manager, sessions: Arc::new(RwLock::new(HashMap::new())), event_bus }
    }

    pub async fn alloc_terminal(&self, session_id: SessionId, rows: u16, cols: u16, shell: &str) -> Result<TerminalId> {
        let id = self.pty_manager.alloc(rows, cols, shell).await?;
        let mut sessions = self.sessions.write().await;
        let entry = sessions.entry(session_id).or_insert(SessionInfo {
            terminals: Vec::new(),
        });
        entry.terminals.push(id);
        Ok(id)
    }

    pub async fn write(&self, id: TerminalId, data: &[u8]) -> Result<()> {
        self.pty_manager.write(id, data).await
    }

    pub async fn read(&self, id: TerminalId, buf: &mut [u8]) -> Result<usize> {
        self.pty_manager.read(id, buf).await
    }

    pub async fn resize(&self, id: TerminalId, rows: u16, cols: u16) -> Result<()> {
        self.pty_manager.resize(id, rows, cols).await
    }

    pub async fn close(&self, id: TerminalId) -> Result<()> {
        self.pty_manager.close(id).await
    }

    pub async fn session_terminals(&self, session_id: SessionId) -> Vec<TerminalId> {
        self.sessions.read().await.get(&session_id)
            .map(|s| s.terminals.clone())
            .unwrap_or_default()
    }
}
