// Shell completion generation
// Adapted from automation/src/cli/completion.rs

use crate::router::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completions
pub fn generate_completions(shell: &str) -> Result<(), String> {
    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => return Err(format!("Unknown shell: {}", shell)),
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "rice", &mut io::stdout());
    Ok(())
}
