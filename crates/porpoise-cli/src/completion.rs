use clap::Command;
use clap_complete::{generate_to, Shell};
use std::io::Error;

#[allow(dead_code)]
pub fn generate_completions(cmd: &mut Command, out_dir: &std::path::Path) -> Result<(), Error> {
    let shells = vec![Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell];
    for shell in shells {
        generate_to(shell, cmd, "porpoise", out_dir)?;
    }
    Ok(())
}
