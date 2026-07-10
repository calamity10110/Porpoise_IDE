pub mod engine;
pub mod remote;
pub mod ssh;
pub mod types;
pub mod watcher;
pub mod worktree;

pub use engine::GitEngine;
pub use remote::{GitHubProvider, RemoteProvider};
pub use ssh::{SshCredentials, clone_ssh, fetch_ssh, push_ssh};
pub use types::*;
pub use watcher::FileWatcher;
pub use worktree::WorktreeManager;
