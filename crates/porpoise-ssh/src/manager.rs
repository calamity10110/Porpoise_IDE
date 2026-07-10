use std::{collections::HashMap, sync::Arc};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::sync::RwLock;

use crate::{auth::AuthMethod, session::SshSession};

pub struct SshManager {
    sessions: Arc<RwLock<HashMap<String, SshSession>>>,
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(&self, host: &str, port: u16, username: &str, auth: &AuthMethod) -> Result<String> {
        let session = SshSession::connect(host, port, username, auth).await?;
        let id = format!("{}@{}:{}", username, host, port);
        self.sessions.write().await.insert(id.clone(), session);
        Ok(id)
    }

    pub async fn disconnect(&self, id: &str) -> Result<()> {
        self.sessions.write().await.remove(id);
        Ok(())
    }

    pub async fn list(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }

    pub async fn exec(&self, id: &str, command: &str) -> Result<String> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(id).ok_or_else(|| PorpoiseError::SshConnect {
            host: id.into(),
            reason: "session not found".into(),
        })?;
        session.exec(command)
    }
}
