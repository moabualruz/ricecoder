use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_command_creation() {
        let cmd = TuiCommand::new(
            Some("dark".to_string()),
            true,
            None,
            Some("openai".to_string()),
            Some("gpt-4".to_string()),
        );

        let config = cmd.get_config();
        assert_eq!(config.theme, Some("dark".to_string()));
        assert!(config.vim_mode);
        assert_eq!(config.provider, Some("openai".to_string()));
        assert_eq!(config.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_tui_command_defaults() {
        let cmd = TuiCommand::new(None, false, None, None, None);
        let config = cmd.get_config();

        assert_eq!(config.theme, None);
        assert!(!config.vim_mode);
        assert_eq!(config.provider, None);
        assert_eq!(config.model, None);
    }

    #[test]
    fn test_tui_config_with_provider() {
        let cmd = TuiCommand::new(
            None,
            false,
            None,
            Some("anthropic".to_string()),
            Some("claude-3-opus".to_string()),
        );

        let config = cmd.get_config();
        assert_eq!(config.provider, Some("anthropic".to_string()));
        assert_eq!(config.model, Some("claude-3-opus".to_string()));
    }

    #[test]
    fn test_tui_config_with_theme() {
        let cmd = TuiCommand::new(Some("monokai".to_string()), false, None, None, None);

        let config = cmd.get_config();
        assert_eq!(config.theme, Some("monokai".to_string()));
    }

    #[test]
    fn test_tui_config_with_vim_mode() {
        let cmd = TuiCommand::new(None, true, None, None, None);

        let config = cmd.get_config();
        assert!(config.vim_mode);
    }
}