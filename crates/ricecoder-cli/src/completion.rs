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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_shells() {
        let shells = vec!["bash", "zsh", "fish", "powershell"];
        for shell in shells {
            // Just verify the function doesn't panic
            let _ = generate_completions(shell);
        }
    }

    #[test]
    fn test_invalid_shell() {
        let result = generate_completions("invalid");
        assert!(result.is_err());
    }
}
