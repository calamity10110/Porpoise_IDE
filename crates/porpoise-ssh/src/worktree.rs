use std::path::PathBuf;

use porpoise_core::error::Result;

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
        let _output = self
            .session
            .exec(&format!(
                "cd {} && git worktree add -B {} {}",
                repo_path, branch, worktree_path
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
        let output = self
            .session
            .exec(&format!("cd {} && git worktree list --porcelain", repo_path))
            .await?;

        Ok(output
            .lines()
            .filter(|l| l.starts_with("worktree "))
            .map(|l| l.trim_start_matches("worktree ").to_string())
            .collect())
    }

    /// Removes a worktree on the remote host.
    pub async fn remove_worktree(&self, repo_path: &str, worktree_path: &str) -> Result<()> {
        self.session
            .exec(&format!("cd {} && git worktree remove {}", repo_path, worktree_path))
            .await?;
        Ok(())
    }

    /// Runs an arbitrary command on the remote host in the repo directory.
    pub async fn exec(&self, command: &str) -> Result<String> {
        self.session.exec(command).await
    }
}
