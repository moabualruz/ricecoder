//! Mode indicator component

use crate::model::AppMode;

/// Mode indicator component
#[derive(Debug, Clone)]
pub struct ModeIndicator {
    /// Current mode
    pub mode: AppMode,
    /// Show keyboard shortcut
    pub show_shortcut: bool,
    /// Show mode capabilities
    pub show_capabilities: bool,
}

impl ModeIndicator {
    /// Create a new mode indicator
    pub fn new(mode: AppMode) -> Self {
        Self {
            mode,
            show_shortcut: true,
            show_capabilities: false,
        }
    }

    /// Get the display text for the mode
    pub fn display_text(&self) -> String {
        if self.show_shortcut {
            format!("[{}] {}", self.mode.shortcut(), self.mode.display_name())
        } else {
            format!("[{}]", self.mode.display_name())
        }
    }

    /// Get the short display text
    pub fn short_text(&self) -> &'static str {
        self.mode.display_name()
    }

    /// Get the capabilities for the current mode
    pub fn get_capabilities(&self) -> Vec<&'static str> {
        match self.mode {
            AppMode::Chat => vec!["QuestionAnswering", "FreeformChat"],
            AppMode::Command => vec!["CodeGeneration", "FileOperations", "CommandExecution"],
            AppMode::Diff => vec!["CodeModification", "FileOperations"],
            AppMode::Mcp => vec!["ToolExecution", "ServerManagement"],
            AppMode::Provider => vec!["ProviderManagement", "StatusMonitoring"],
            AppMode::Session => vec!["SessionManagement", "Sharing"],
            AppMode::Help => vec!["QuestionAnswering"],
        }
    }

    /// Get capabilities display text
    pub fn capabilities_text(&self) -> String {
        let caps = self.get_capabilities();
        format!("Capabilities: {}", caps.join(", "))
    }

    /// Update the mode
    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    /// Toggle shortcut display
    pub fn toggle_shortcut_display(&mut self) {
        self.show_shortcut = !self.show_shortcut;
    }

    /// Toggle capabilities display
    pub fn toggle_capabilities_display(&mut self) {
        self.show_capabilities = !self.show_capabilities;
    }

    /// Enable capabilities display
    pub fn show_capabilities_enabled(&mut self) {
        self.show_capabilities = true;
    }

    /// Disable capabilities display
    pub fn hide_capabilities_enabled(&mut self) {
        self.show_capabilities = false;
    }
}

impl Default for ModeIndicator {
    fn default() -> Self {
        Self::new(AppMode::Chat)
    }
}
