use ricecoder_cli::*;

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