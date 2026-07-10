use std::io::Error;

use clap::Command;
use clap_complete::{Shell, generate_to};

#[allow(dead_code)]
pub fn generate_completions(cmd: &mut Command, out_dir: &std::path::Path) -> Result<(), Error> {
    let shells = vec![Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell];
    for shell in shells {
        generate_to(shell, cmd, "porpoise", out_dir)?;
    }
    Ok(())
}
