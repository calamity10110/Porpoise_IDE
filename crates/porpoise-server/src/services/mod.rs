pub mod agent;
pub mod browser_service;
pub mod config;
pub mod git;
pub mod health;
pub mod notifications;
pub mod skills_service;
pub mod ssh_service;
pub mod terminal;
pub mod worktree;

#[cfg(unix)]
mod relay_handlers;

pub use notifications::NotificationService;
#[cfg(unix)]
pub use relay_handlers::register_all;
