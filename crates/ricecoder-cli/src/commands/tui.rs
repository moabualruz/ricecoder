//! TUI command - Launch the terminal user interface

use crate::commands::Command;
use crate::error::{CliError, CliResult};
use std::path::PathBuf;
use ricecoder_providers::provider::manager::ProviderManager;
use chrono;

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

/// Load provider data for TUI initialization
async fn load_provider_data_for_tui() -> CliResult<(Vec<ricecoder_tui::model::ProviderInfo>, Option<String>)> {
    // Get provider manager from DI container
    let provider_manager = crate::di::get_service::<ProviderManager>()
        .ok_or_else(|| CliError::Internal("ProviderManager not available in DI container".to_string()))?;

    // Get available providers
    let available_providers = provider_manager.get_all_provider_statuses()
        .into_iter()
        .map(|status| ricecoder_tui::model::ProviderInfo {
            id: status.id.clone(),
            name: status.name.clone(),
            state: match status.state {
                ricecoder_providers::provider::manager::ConnectionState::Connected => ricecoder_tui::model::ProviderConnectionState::Connected,
                ricecoder_providers::provider::manager::ConnectionState::Disconnected => ricecoder_tui::model::ProviderConnectionState::Disconnected,
                ricecoder_providers::provider::manager::ConnectionState::Error => ricecoder_tui::model::ProviderConnectionState::Error,
                ricecoder_providers::provider::manager::ConnectionState::Disabled => ricecoder_tui::model::ProviderConnectionState::Disabled,
            },
            models: status.models.iter().map(|m| m.id.clone()).collect(),
            error_message: status.error_message.clone(),
            last_checked: status.last_checked.map(|st| {
                let duration = st.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos()).unwrap_or_else(|| chrono::Utc::now())
            }),
        })
        .collect();

    // Get current provider (for now, we'll set this to None and let the TUI handle provider switching)
    let current_provider = None;

    Ok((available_providers, current_provider))
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

        // Provider and model are handled separately in load_provider_data_for_tui

        // Load provider data for the TUI
        let (available_providers, current_provider) = load_provider_data_for_tui().await?;

        // Create and run the application
        let mut app = App::with_config_and_providers(tui_config, available_providers, current_provider).await
            .map_err(|e| CliError::Internal(format!("Failed to initialize TUI: {}", e)))?;

        Ok(())
    })
}




