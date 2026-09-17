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
    #[expect(dead_code)]
    token_path: PathBuf,
}

impl SessionTokenStore {
    pub fn create(data_dir: &Path) -> Result<Self> {
        let token_path = data_dir.join("ipc-token");
        let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();

        #[cfg(unix)]
        {
            use std::{io::Write, os::unix::fs::OpenOptionsExt};
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
            std::fs::write(&token_path, &token).map_err(|e| PorpoiseError::Ipc(format!("write token: {e}")))?;
            restrict_token_acl_windows(&token_path)
                .map_err(|e| PorpoiseError::Ipc(format!("restrict token acl: {e}")))?;
        }
        Ok(Self { token, token_path })
    }

    pub fn validate_handshake(&self, client_hs: &Handshake) -> bool {
        // Constant-time comparison (CVE-P1-3): a plain `==` short-circuits on
        // the first differing byte, leaking token contents over a timing side
        // channel. Compare every byte unconditionally so no such channel exists.
        match client_hs.session_token.as_deref() {
            Some(provided) => constant_time_eq(provided.as_bytes(), self.token.as_bytes()),
            None => false,
        }
    }
}

pub fn load_token(path: &Path) -> Result<String> {
    let token = std::fs::read_to_string(path).map_err(|e| PorpoiseError::Ipc(format!("read token: {e}")))?;
    Ok(token.trim().to_string())
}

#[cfg(windows)]
fn restrict_token_acl_windows(path: &Path) -> std::io::Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "token path must be valid UTF-8"))?;
    let username = std::env::var("USERNAME")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "USERNAME env var not set"))?;

    // /inheritance:r removes inherited ACEs; /grant:r replaces ACEs with current-user-only Full Control.
    let output = std::process::Command::new("icacls")
        .arg(path_str)
        .args(["/inheritance:r"])
        .args(["/grant:r", &format!("{username}:F")])
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "icacls failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn restrict_token_acl_windows(_path: &Path) -> std::io::Result<()> {
    Ok(())
}


/// Compares two byte slices in constant time.
///
/// Every byte is XOR-reduced into an accumulator before the final equality test,
/// so the running time does not depend on *where* (or whether) the slices first
/// differ. A length mismatch returns `false` immediately; for the fixed-length
/// IPC token this is acceptable since token length is not secret.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_matches_and_mismatches() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn validate_handshake_accepts_only_exact_token() {
        let store = SessionTokenStore {
            token: "deadbeefcafebabe".to_string(),
            token_path: std::path::PathBuf::from("/tmp/x"),
        };
        // helper closure building a Handshake by mutating only the token field
        let make = |t: Option<String>| Handshake {
            version: 1,
            min_version: 1,
            server_name: "test".to_string(),
            session_token: t,
            peer_pid: 0,
        };
        assert!(store.validate_handshake(&make(Some("deadbeefcafebabe".to_string()))));
        assert!(!store.validate_handshake(&make(Some("deadbeefcafef00d".to_string()))));
        assert!(!store.validate_handshake(&make(None)));
        // a token that shares a long prefix must still be rejected
        assert!(!store.validate_handshake(&make(Some("deadbeefcafebab0".to_string()))));
        // silence unused-mut if the field is read-only in this context
        let _ = &store;
    }
}
