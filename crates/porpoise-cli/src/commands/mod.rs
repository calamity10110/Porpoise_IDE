pub mod agent;
pub mod browser;
pub mod config;
pub mod daemon;
pub mod git;
pub mod mobile;
pub mod skill;
pub mod ssh;
pub mod terminal;
pub mod worktree;

use porpoise_core::error::Result;

use crate::{app::Commands, output::OutputFormat};

/// Pipe long text output through the system pager (`less`).
/// No-op if output is short or pager unavailable.
fn page_output(output: &str) -> String {
    let line_count = output.lines().count();
    if line_count <= 24 {
        return output.to_string();
    }
    if let Ok(mut child) = std::process::Command::new("less")
        .args(["-F", "-R", "-X"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(output.as_bytes());
            let _ = stdin.flush();
        }
        let _ = child.wait();
    }
    String::new()
}

fn should_page(cmd: &Commands) -> bool {
    matches!(cmd, Commands::Worktree(_) | Commands::Agent(_) | Commands::Terminal(_))
}

pub async fn handle_command(cmd: Commands, format: &OutputFormat) -> Result<String> {
    let is_list = should_page(&cmd);
    let result = match cmd {
        Commands::Daemon(args) => daemon::handle(args, format).await,
        Commands::Worktree(args) => worktree::handle(args, format).await,
        Commands::Terminal(args) => terminal::handle(args, format).await,
        Commands::Agent(args) => agent::handle(args, format).await,
        Commands::Git(args) => git::handle(args, format).await,
        Commands::Browser(args) => browser::handle(args, format).await,
        Commands::Ssh(args) => ssh::handle(args, format).await,
        Commands::Config(args) => config::handle(args, format).await,
        Commands::Mobile(args) => mobile::handle(args, format).await,
        Commands::Skill(args) => skill::handle(args, format).await,
        Commands::Status => Ok(format.format(&serde_json::json!({"status": "running"}))),
        Commands::Version => Ok(format!("porpoise {}", env!("CARGO_PKG_VERSION"))),
    }?;
    if is_list { Ok(page_output(&result)) } else { Ok(result) }
}
