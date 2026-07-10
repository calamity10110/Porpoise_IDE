use std::{sync::Arc, time::Duration};

use porpoise_core::error::{PorpoiseError, Result};
use tokio::{sync::RwLock, time::sleep};

use crate::{auth::AuthMethod, session::SshSession};

/// Configuration for SSH auto-reconnect behavior.
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts (0 = infinite).
    pub max_attempts: u32,
    /// Initial delay before the first retry (doubles each attempt).
    pub initial_delay_ms: u64,
    /// Maximum delay between retries.
    pub max_delay_ms: u64,
    /// TCP keepalive interval in seconds (0 = disabled).
    pub keepalive_interval: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 500,
            max_delay_ms: 30_000,
            keepalive_interval: 30,
        }
    }
}

/// A managed SSH connection that automatically reconnects on failure.
///
/// Wraps an `SshSession` and monitors connectivity. When the connection drops,
/// it attempts to reconnect with exponential backoff. Ongoing commands are
/// failed with a `ConnectionRefused` error when the connection is lost.
pub struct AutoReconnectSession {
    inner: Arc<RwLock<Option<SshSession>>>,
    config: ReconnectConfig,
    host: String,
    port: u16,
    username: String,
    auth: AuthMethod,
    connected: Arc<std::sync::atomic::AtomicBool>,
}

impl AutoReconnectSession {
    /// Creates a new auto-reconnecting SSH session and connects.
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        auth: AuthMethod,
        config: ReconnectConfig,
    ) -> Result<Self> {
        let session = SshSession::connect(host, port, username, &auth).await?;

        if config.keepalive_interval > 0 {
            let _ = session.set_keepalive(true, config.keepalive_interval);
        }

        let connected = Arc::new(std::sync::atomic::AtomicBool::new(true));

        Ok(Self {
            inner: Arc::new(RwLock::new(Some(session))),
            config,
            host: host.into(),
            port,
            username: username.into(),
            auth,
            connected,
        })
    }

    /// Executes a command on the remote host, attempting reconnect on failure.
    pub async fn exec(&self, command: &str) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(ref session) = *guard {
            match session.exec(command) {
                Ok(output) => return Ok(output),
                Err(_e) => {
                    drop(guard);
                    self.connected.store(false, std::sync::atomic::Ordering::Relaxed);
                    self.reconnect().await?;
                    return self.exec_after_reconnect(command).await;
                }
            }
        }
        self.reconnect().await?;
        self.exec_after_reconnect(command).await
    }

    async fn exec_after_reconnect(&self, command: &str) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(ref session) = *guard {
            session.exec(command)
        } else {
            Err(PorpoiseError::SshConnect {
                host: self.host.clone(),
                reason: "not connected".into(),
            })
        }
    }

    async fn reconnect(&self) -> Result<()> {
        let mut delay = self.config.initial_delay_ms;
        for _ in 0..self.config.max_attempts {
            sleep(Duration::from_millis(delay)).await;
            match SshSession::connect(&self.host, self.port, &self.username, &self.auth).await {
                Ok(session) => {
                    if self.config.keepalive_interval > 0 {
                        let _ = session.set_keepalive(true, self.config.keepalive_interval);
                    }
                    let mut guard = self.inner.write().await;
                    *guard = Some(session);
                    self.connected.store(true, std::sync::atomic::Ordering::Relaxed);
                    return Ok(());
                }
                Err(_) => {
                    delay = (delay * 2).min(self.config.max_delay_ms);
                }
            }
        }
        Err(PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("reconnect failed after {} attempts", self.config.max_attempts),
        })
    }

    /// Returns the host:port string for this connection.
    pub fn connected_str(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns whether the session is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the reconnect configuration.
    pub fn config(&self) -> &ReconnectConfig {
        &self.config
    }
}
