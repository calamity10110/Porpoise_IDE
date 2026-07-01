pub mod agent;
pub mod browser;
pub mod config;
pub mod daemon;
pub mod git;
pub mod skill;
pub mod ssh;
pub mod terminal;
pub mod worktree;

use crate::app::Commands;
use crate::output::OutputFormat;
use porpoise_core::error::Result;

pub async fn handle_command(cmd: Commands, format: &OutputFormat) -> Result<String> {
    match cmd {
        Commands::Daemon(args) => daemon::handle(args, format).await,
        Commands::Worktree(args) => worktree::handle(args, format).await,
        Commands::Terminal(args) => terminal::handle(args, format).await,
        Commands::Agent(args) => agent::handle(args, format).await,
        Commands::Git(args) => git::handle(args, format).await,
        Commands::Browser(args) => browser::handle(args, format).await,
        Commands::Ssh(args) => ssh::handle(args, format).await,
        Commands::Config(args) => config::handle(args, format).await,
        Commands::Skill(args) => skill::handle(args, format).await,
        Commands::Status => Ok(format.format(&serde_json::json!({"status": "running"}))),
        Commands::Version => Ok(format!("porpoise {}", env!("CARGO_PKG_VERSION"))),
    }
}
