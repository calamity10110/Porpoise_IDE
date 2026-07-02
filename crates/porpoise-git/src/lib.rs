pub mod engine;
pub mod watcher;
pub mod worktree;
pub mod remote;
pub mod types;

pub use engine::GitEngine;
pub use worktree::WorktreeManager;
pub use remote::{GitHubProvider, RemoteProvider};
pub use watcher::FileWatcher;
pub use types::*;

