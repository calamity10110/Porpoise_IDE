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

        // Verify the remote host key before authentication. Without this the
        // connection trusts whatever key the peer presents during the handshake,
        // allowing a man-in-the-middle to silently capture credentials (CVE-C1).
        verify_host_key(&sess, host, crate::hostkey::HostKeyPolicy::TrustOnFirstUse)?;

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

/// Resolves the default OpenSSH `known_hosts` file (`~/.ssh/known_hosts`).
///
/// Returns `None` when no home directory can be determined, in which case the
/// caller treats every host as unseen (first-use) under TOFU.
fn default_known_hosts_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| {
        if cfg!(windows) {
            std::env::var_os("USERPROFILE")
        } else {
            None
        }
    })?;
    Some(std::path::PathBuf::from(home).join(".ssh").join("known_hosts"))
}

/// Verifies the remote host key against the local `known_hosts` collection
/// *before* any credentials are sent.
///
/// Without this check the client authenticates against whatever key the peer
/// presented during the handshake, so a man-in-the-middle attacker positioned
/// between us and the real server can silently capture the password/key
/// (CVE-C1).
///
/// Decisions (fail-closed):
/// - `Match`    -> proceed.
/// - `NotFound` -> under [`TrustOnFirstUse`][crate::hostkey::HostKeyPolicy] the
///   key is recorded to `known_hosts`, then the connection proceeds; under
///   `Strict` the connection is refused.
/// - `Mismatch` / `Failure` -> always refused; authentication never happens.
fn verify_host_key(sess: &ssh2::Session, host: &str, policy: crate::hostkey::HostKeyPolicy) -> Result<()> {
    let (key, key_type) = match sess.host_key() {
        Some(v) => v,
        None => {
            return Err(PorpoiseError::SshConnect {
                host: host.into(),
                reason: "peer did not present a host key".into(),
            });
        }
    };

    let mut kh = sess.known_hosts().map_err(|e| PorpoiseError::SshConnect {
        host: host.into(),
        reason: format!("known_hosts init: {e}"),
    })?;

    // Load any persisted known_hosts so a previously-seen host is recognized.
    if let Some(path) = default_known_hosts_path()
        && path.exists()
    {
        let _ = kh.read_file(&path, ssh2::KnownHostFileKind::OpenSSH);
    }

    let verdict = crate::hostkey::classify(kh.check(host, key));

    if !crate::hostkey::policy_accepts(verdict, policy) {
        return Err(PorpoiseError::SshConnect {
            host: host.into(),
            reason: match verdict {
                Some(crate::hostkey::HostKeyVerdict::KeyMismatch) => {
                    "host key mismatch (possible MITM); refusing to authenticate".into()
                }
                _ => "host key verification failed; refusing to authenticate".into(),
            },
        });
    }

    // Trust-on-first-use: persist the newly seen key so a *future* mismatch is
    // detected instead of silently accepted.
    if matches!(verdict, Some(crate::hostkey::HostKeyVerdict::UnknownHost)) {
        let _ = kh.add(host, key, "porpoise-ssh", ssh2::KnownHostKeyFormat::from(key_type));
        if let Some(path) = default_known_hosts_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = kh.write_file(&path, ssh2::KnownHostFileKind::OpenSSH);
        }
    }

    Ok(())
}
