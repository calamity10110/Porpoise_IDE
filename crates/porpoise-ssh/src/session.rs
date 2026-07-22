use std::{io::Read, sync::Mutex};

use porpoise_core::error::{PorpoiseError, Result};

use crate::auth::AuthMethod;

pub struct SshSession {
    session: Mutex<ssh2::Session>,
    host: String,
    #[allow(dead_code)]
    port: u16,
    #[allow(dead_code)]
    username: String,
}

impl SshSession {
    pub async fn connect(host: &str, port: u16, username: &str, auth: &AuthMethod) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let tcp = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| PorpoiseError::SshConnect {
                host: host.into(),
                reason: e.to_string(),
            })?;
        let tcp_std = tcp.into_std().map_err(|e| PorpoiseError::SshConnect {
            host: host.into(),
            reason: e.to_string(),
        })?;

        let mut sess = ssh2::Session::new().map_err(|e| PorpoiseError::SshConnect {
            host: host.into(),
            reason: e.to_string(),
        })?;
        sess.set_tcp_stream(tcp_std);
        sess.handshake().map_err(|e| PorpoiseError::SshConnect {
            host: host.into(),
            reason: e.to_string(),
        })?;

        match auth {
            AuthMethod::Password(password) => {
                sess.userauth_password(username, password.as_str())
                    .map_err(|e| PorpoiseError::SshAuth(e.to_string()))?;
            }
            AuthMethod::KeyFile(path, passphrase) => {
                sess.userauth_pubkey_file(
                    username,
                    None,
                    std::path::Path::new(path),
                    passphrase.as_deref().map(|p| p.as_str()),
                )
                .map_err(|e| PorpoiseError::SshAuth(e.to_string()))?;
            }
            AuthMethod::Agent => {
                sess.userauth_agent(username)
                    .map_err(|e| PorpoiseError::SshAuth(e.to_string()))?;
            }
        }

        Ok(Self {
            session: Mutex::new(sess),
            host: host.into(),
            port,
            username: username.into(),
        })
    }

    /// Executes a command on the remote host via SSH and returns stdout.
    ///
    /// Stderr is merged into the returned string. Non-zero exit codes are
    /// returned as an `SshConnect` error with the exit code in the message.
    pub fn exec(&self, command: &str) -> Result<String> {
        let session = self.session.lock().map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("lock: {e}"),
        })?;

        let mut channel = session.channel_session().map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("channel: {e}"),
        })?;

        channel.exec(command).map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("exec: {e}"),
        })?;

        let mut output = String::new();
        loop {
            let mut buf = [0u8; 8192];
            match channel.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => output.push_str(&String::from_utf8_lossy(&buf[..n])),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if !channel.eof() {
                        std::hint::spin_loop();
                        continue;
                    }
                    break;
                }
                Err(e) => {
                    return Err(PorpoiseError::SshConnect {
                        host: self.host.clone(),
                        reason: format!("read: {e}"),
                    });
                }
            }
        }

        let _ = channel.wait_close();
        let exit_code = channel.exit_status().unwrap_or(-1);

        if exit_code != 0 {
            return Err(PorpoiseError::SshConnect {
                host: self.host.clone(),
                reason: format!("exit code {exit_code}: {output}"),
            });
        }

        Ok(output)
    }

    /// Returns a reference to the underlying ssh2 session for advanced operations.
    pub fn raw_session(&self) -> &Mutex<ssh2::Session> {
        &self.session
    }

    /// Enables TCP keepalive with the given interval in seconds.
    ///
    /// When `want_reply` is true, the keepalive messages request an acknowledgment
    /// from the server, allowing detection of dead connections within two intervals.
    pub fn set_keepalive(&self, want_reply: bool, interval_secs: u32) -> Result<()> {
        let session = self.session.lock().map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("lock: {e}"),
        })?;
        session.set_keepalive(want_reply, interval_secs);
        Ok(())
    }

    /// Opens a direct TCP/IP tunnel through the SSH server to a third-party host.
    ///
    /// Communication from client to SSH server is encrypted; from server to target
    /// host travels in cleartext. Returns the raw `ssh2::Channel` for read/write.
    pub fn port_forward(&self, target_host: &str, target_port: u16) -> Result<ssh2::Channel> {
        let session = self.session.lock().map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("lock: {e}"),
        })?;
        session
            .channel_direct_tcpip(target_host, target_port, None)
            .map_err(|e| PorpoiseError::SshConnect {
                host: format!("{target_host}:{target_port}"),
                reason: format!("port forward: {e}"),
            })
    }

    /// Starts listening for inbound TCP/IP connections on the remote host.
    ///
    /// Returns a `ssh2::Listener` and the actual port assigned. New connections
    /// are accepted via `listener.accept()`.
    pub fn forward_listen(&self, remote_port: u16, host: Option<&str>) -> Result<(ssh2::Listener, u16)> {
        let session = self.session.lock().map_err(|e| PorpoiseError::SshConnect {
            host: self.host.clone(),
            reason: format!("lock: {e}"),
        })?;
        let (listener, port) =
            session
                .channel_forward_listen(remote_port, host, None)
                .map_err(|e| PorpoiseError::SshConnect {
                    host: self.host.clone(),
                    reason: format!("forward listen: {e}"),
                })?;
        Ok((listener, port))
    }
}
