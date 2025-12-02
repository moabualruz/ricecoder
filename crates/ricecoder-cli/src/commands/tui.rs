//! TUI command - Launch the terminal user interface

use crate::error::{CliError, CliResult};
use crate::commands::Command;
use std::path::PathBuf;

/// TUI command configuration
#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// Theme to use (dark, light, monokai, dracula, nord)
    pub theme: Option<String>,
    /// Enable vim keybindings
    pub vim_mode: bool,
    /// Custom config file path
    pub config_file: Option<PathBuf>,
    /// AI provider to use
    pub provider: Option<String>,
    /// Model to use
    pub model: Option<String>,
}

/// TUI command handler
pub struct TuiCommand {
    theme: Option<String>,
    vim_mode: bool,
    config_file: Option<PathBuf>,
    provider: Option<String>,
    model: Option<String>,
}

impl TuiCommand {
    /// Create a new TUI command
    pub fn new(
        theme: Option<String>,
        vim_mode: bool,
        config_file: Option<PathBuf>,
        provider: Option<String>,
        model: Option<String>,
    ) -> Self {
        Self {
            theme,
            vim_mode,
            config_file,
            provider,
            model,
        }
    }

    /// Get the TUI configuration
    pub fn get_config(&self) -> TuiConfig {
        TuiConfig {
            theme: self.theme.clone(),
            vim_mode: self.vim_mode,
            config_file: self.config_file.clone(),
            provider: self.provider.clone(),
            model: self.model.clone(),
        }
    }
}

impl Command for TuiCommand {
    fn execute(&self) -> CliResult<()> {
        // Build TUI configuration
        let config = self.get_config();

        // Launch the TUI application
        launch_tui(config)
    }
}

/// Launch the TUI application
fn launch_tui(config: TuiConfig) -> CliResult<()> {
    // Create a runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Internal(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        // Import the TUI app
        use ricecoder_tui::App;

        // Create TUI configuration from CLI config
        let mut tui_config = ricecoder_tui::TuiConfig::default();

        // Apply theme if specified
        if let Some(theme) = config.theme {
            tui_config.theme = theme;
        }

        // Apply vim mode if enabled
        if config.vim_mode {
            tui_config.vim_mode = true;
        }

        // Apply provider and model if specified
        if let Some(provider) = config.provider {
            tui_config.provider = Some(provider);
        }
        if let Some(model) = config.model {
            tui_config.model = Some(model);
        }

        // Validate provider configuration if specified
        if tui_config.provider.is_some() {
            validate_provider_config(&tui_config)?;
        }

        // Create and run the application
        let mut app = App::with_config(tui_config)
            .map_err(|e| CliError::Internal(format!("Failed to initialize TUI: {}", e)))?;

        app.run()
            .await
            .map_err(|e| CliError::Internal(format!("TUI error: {}", e)))
    })
}

/// Validate provider configuration
fn validate_provider_config(config: &ricecoder_tui::TuiConfig) -> CliResult<()> {
    let supported_providers = vec!["openai", "anthropic", "ollama", "google", "zen"];
    
    if let Some(ref provider) = config.provider {
        if !supported_providers.contains(&provider.as_str()) {
            return Err(CliError::Internal(format!(
                "Unsupported provider: {}. Supported providers: {}",
                provider,
                supported_providers.join(", ")
            )));
        }
    }

    Ok(())
}

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
        let cmd = TuiCommand::new(
            Some("monokai".to_string()),
            false,
            None,
            None,
            None,
        );

        let config = cmd.get_config();
        assert_eq!(config.theme, Some("monokai".to_string()));
    }

    #[test]
    fn test_tui_config_with_vim_mode() {
        let cmd = TuiCommand::new(
            None,
            true,
            None,
            None,
            None,
        );

        let config = cmd.get_config();
        assert!(config.vim_mode);
    }
}
