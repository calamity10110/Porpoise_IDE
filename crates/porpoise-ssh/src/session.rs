use porpoise_core::error::{PorpoiseError, Result};
use crate::auth::AuthMethod;

pub struct SshSession {
    _host: String,
    _port: u16,
    _username: String,
}

impl SshSession {
    pub async fn connect(host: &str, port: u16, username: &str, _auth: &AuthMethod) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let tcp = tokio::net::TcpStream::connect(&addr).await
            .map_err(|e| PorpoiseError::SshConnect { host: host.into(), reason: e.to_string() })?;
        let tcp_std = tcp.into_std()
            .map_err(|e| PorpoiseError::SshConnect { host: host.into(), reason: e.to_string() })?;

        let mut sess = ssh2::Session::new()
            .map_err(|e| PorpoiseError::SshConnect { host: host.into(), reason: e.to_string() })?;
        sess.set_tcp_stream(tcp_std);
        sess.handshake()
            .map_err(|e| PorpoiseError::SshConnect { host: host.into(), reason: e.to_string() })?;

        Ok(Self { _host: host.into(), _port: port, _username: username.into() })
    }

    pub fn exec(&self, _command: &str) -> Result<String> {
        Err(PorpoiseError::Unimplemented("SSH exec via ssh2 channel"))
    }
}
