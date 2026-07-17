use std::path::{Path, PathBuf};

use porpoise_core::error::{PorpoiseError, Result};
use serde::{Deserialize, Serialize};

use crate::message::Handshake;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting { next_attempt_ms: u64 },
}

pub struct SessionTokenStore {
    pub token: String,
    token_path: PathBuf,
}

impl SessionTokenStore {
    pub fn create(data_dir: &Path) -> Result<Self> {
        let token_path = data_dir.join("ipc-token");
        // 32 random bytes → 64-char hex string
        let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();

        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&token_path)
                .map_err(|e| PorpoiseError::Ipc(format!("create token file: {e}")))?;
            file.write_all(token.as_bytes())
                .map_err(|e| PorpoiseError::Ipc(format!("write token: {e}")))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&token_path, &token)
                .map_err(|e| PorpoiseError::Ipc(format!("write token: {e}")))?;
        }
        Ok(Self { token, token_path })
    }

    pub fn validate_handshake(&self, client_hs: &Handshake) -> bool {
        client_hs.session_token.as_deref() == Some(self.token.as_str())
    }
}

pub fn load_token(path: &Path) -> Result<String> {
    let token = std::fs::read_to_string(path)
        .map_err(|e| PorpoiseError::Ipc(format!("read token: {e}")))?;
    Ok(token.trim().to_string())
}
