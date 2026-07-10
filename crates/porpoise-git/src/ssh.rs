use std::path::{Path, PathBuf};

use git2::{Cred, RemoteCallbacks, Repository};
use porpoise_core::error::{PorpoiseError, Result};

/// SSH authentication credentials for git operations.
#[derive(Debug, Clone)]
pub enum SshCredentials {
    /// Use a specific private key file with optional passphrase.
    KeyFile {
        private_key_path: PathBuf,
        passphrase: Option<String>,
    },
    /// Use the SSH agent (ssh-agent on Unix, Pageant on Windows).
    Agent,
    /// Use a default key from `~/.ssh/`.
    DefaultKey,
}

/// Clones a repository using SSH key authentication.
///
/// Uses `git2`'s `RemoteCallbacks` to provide credentials during the
/// SSH handshake. Supports explicit key files, SSH agent, and default
/// `~/.ssh/` keys.
pub fn clone_ssh(url: &str, path: &Path, credentials: &SshCredentials) -> Result<Repository> {
    let mut callbacks = RemoteCallbacks::new();
    let creds = credentials.clone();

    callbacks.credentials(move |_url, username_from_url, _allowed_types| match &creds {
        SshCredentials::KeyFile {
            private_key_path,
            passphrase,
        } => Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            private_key_path,
            passphrase.as_deref(),
        ),
        SshCredentials::Agent => Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")),
        SshCredentials::DefaultKey => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| git2::Error::from_str("cannot determine home directory for SSH key"))?;
            let key = PathBuf::from(home).join(".ssh/id_rsa");
            Cred::ssh_key(username_from_url.unwrap_or("git"), None, &key, None)
        }
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);
    builder
        .clone(url, path)
        .map_err(|e| PorpoiseError::Git(format!("SSH clone '{url}': {e}")))
}

/// Pushes a branch to a remote using SSH key authentication.
pub fn push_ssh(repo: &Repository, remote: &str, branch: &str, credentials: &SshCredentials) -> Result<()> {
    let mut callbacks = RemoteCallbacks::new();
    let creds = credentials.clone();

    callbacks.credentials(move |_url, username_from_url, _allowed_types| match &creds {
        SshCredentials::KeyFile {
            private_key_path,
            passphrase,
        } => Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            private_key_path,
            passphrase.as_deref(),
        ),
        SshCredentials::Agent => Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")),
        SshCredentials::DefaultKey => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| git2::Error::from_str("cannot determine home directory for SSH key"))?;
            let key = PathBuf::from(home).join(".ssh/id_rsa");
            Cred::ssh_key(username_from_url.unwrap_or("git"), None, &key, None)
        }
    });

    let mut remote = repo
        .find_remote(remote)
        .map_err(|e| PorpoiseError::Git(format!("find remote: {e}")))?;

    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| PorpoiseError::Git(format!("SSH push: {e}")))?;

    Ok(())
}

/// Fetches from a remote using SSH key authentication.
pub fn fetch_ssh(repo: &Repository, remote: &str, credentials: &SshCredentials) -> Result<()> {
    let mut callbacks = RemoteCallbacks::new();
    let creds = credentials.clone();

    callbacks.credentials(move |_url, username_from_url, _allowed_types| match &creds {
        SshCredentials::KeyFile {
            private_key_path,
            passphrase,
        } => Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            private_key_path,
            passphrase.as_deref(),
        ),
        SshCredentials::Agent => Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")),
        SshCredentials::DefaultKey => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| git2::Error::from_str("cannot determine home directory for SSH key"))?;
            let key = PathBuf::from(home).join(".ssh/id_rsa");
            Cred::ssh_key(username_from_url.unwrap_or("git"), None, &key, None)
        }
    });

    let mut remote = repo
        .find_remote(remote)
        .map_err(|e| PorpoiseError::Git(format!("find remote: {e}")))?;

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    remote
        .fetch(&[] as &[&str], Some(&mut fetch_options), None)
        .map_err(|e| PorpoiseError::Git(format!("SSH fetch: {e}")))?;

    Ok(())
}

impl GitEngine {
    /// Clones a repository over SSH with key-based authentication.
    pub fn clone_ssh(url: &str, path: &Path, credentials: &SshCredentials) -> Result<Self> {
        let repo = crate::ssh::clone_ssh(url, path, credentials)?;
        Ok(Self {
            repo_path: path.to_path_buf(),
            repo,
        })
    }

    /// Pushes the current branch to origin over SSH.
    pub fn push_ssh(&self, branch: &str, credentials: &SshCredentials) -> Result<()> {
        crate::ssh::push_ssh(&self.repo, "origin", branch, credentials)
    }

    /// Fetches from origin over SSH.
    pub fn fetch_ssh(&self, credentials: &SshCredentials) -> Result<()> {
        crate::ssh::fetch_ssh(&self.repo, "origin", credentials)
    }
}

use crate::engine::GitEngine;
