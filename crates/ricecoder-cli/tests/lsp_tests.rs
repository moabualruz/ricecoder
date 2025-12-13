use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_command_creation() {
        let cmd = LspCommand::new(Some("debug".to_string()), Some(8080), true);
        let config = cmd.get_config();

        assert_eq!(config.log_level, "debug");
        assert_eq!(config.port, Some(8080));
        assert!(config.debug);
    }

    #[test]
    fn test_lsp_command_defaults() {
        let cmd = LspCommand::new(None, None, false);
        let config = cmd.get_config();

        assert_eq!(config.log_level, "info");
        assert_eq!(config.port, None);
        assert!(!config.debug);
    }

    #[test]
    fn test_lsp_command_debug_mode() {
        let cmd = LspCommand::new(None, None, true);
        let config = cmd.get_config();

        assert_eq!(config.log_level, "debug");
        assert!(config.debug);
    }

    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.port, None);
        assert!(!config.debug);
    }

    #[test]
    fn test_lsp_command_with_port() {
        let cmd = LspCommand::new(None, Some(9000), false);
        let config = cmd.get_config();

        assert_eq!(config.port, Some(9000));
    }

    #[test]
    fn test_lsp_command_with_log_level() {
        let cmd = LspCommand::new(Some("trace".to_string()), None, false);
        let config = cmd.get_config();

        assert_eq!(config.log_level, "trace");
    }
}