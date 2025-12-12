//! Elm Architecture (TEA) implementation for RiceCoder TUI
//!
//! This module implements the Model-Update-View pattern for predictable,
//! immutable state management with structural sharing and reactive updates.
//!
//! The core TEA components are now split into separate modules:
//! - `model.rs`: Contains AppModel, AppMessage, and related state types
//! - `update.rs`: Contains the pure update function and Command enum
//! - `tea.rs`: Contains ReactiveState manager and TEA orchestration

use crate::model::*;
use crate::update::Command;
use ricecoder_storage::TuiConfig;
use crate::style::Theme;
use crate::terminal_state::TerminalCapabilities;

impl AppModel {
    /// Create initial application state
    pub fn init(
        config: TuiConfig,
        theme: Theme,
        terminal_caps: TerminalCapabilities,
    ) -> Self {
        crate::model::AppModel::init(config, theme, terminal_caps)
    }

    /// Pure update function - delegates to update module
    pub fn update(self, message: AppMessage) -> (Self, Vec<Command>) {
        crate::update::AppModel::update(self, message)
    }

// StateDiff and StateChange are now in model.rs
}

/// Reactive state manager with change tracking
pub struct ReactiveState {
    current: AppModel,
    history: Vec<AppModel>,
    max_history: usize,
}

impl ReactiveState {
    pub fn new(initial_state: AppModel) -> Self {
        Self {
            current: initial_state,
            history: Vec::new(),
            max_history: 50, // Keep last 50 states for undo
        }
    }

    /// Apply a message and return the state diff
    pub fn update(&mut self, message: AppMessage) -> Result<StateDiff, String> {
        if !self.current.can_transition(&message) {
            return Err(format!("Invalid state transition: {:?}", message));
        }

        let previous = self.current.clone();
        let (new_state, _commands) = self.current.update(message);

        // Validate the new state
        new_state.validate()?;

        // Store previous state in history
        self.history.push(previous);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Calculate diff
        let diff = new_state.diff(&self.current);

        // Update current state
        self.current = new_state;

        Ok(diff)
    }

    /// Get current state (immutable reference)
    pub fn current(&self) -> &AppModel {
        &self.current
    }

    /// Undo last change
    pub fn undo(&mut self) -> Result<StateDiff, String> {
        if let Some(previous) = self.history.pop() {
            let diff = self.current.diff(&previous);
            self.current = previous;
            Ok(diff)
        } else {
            Err("No more states to undo".to_string())
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Get state at specific history index
    pub fn state_at(&self, index: usize) -> Option<&AppModel> {
        if index == 0 {
            Some(&self.current)
        } else {
            self.history.get(self.history.len().saturating_sub(index))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::ChatWidget;
    use ricecoder_help::HelpDialog;

    fn create_test_model() -> AppModel {
        AppModel {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme: crate::style::Theme::default(),
            terminal_caps: crate::terminal_state::TerminalCapabilities::default(),

            sessions: SessionState {
                active_session_id: Some("test-session".to_string()),
                session_count: 1,
                total_tokens: ricecoder_sessions::TokenUsage::default(),
            },

            commands: TeaCommandState {
                command_history: vec![],
                current_command: "".to_string(),
                command_palette_visible: false,
            },

            ui: UiState {
                focus_manager: crate::accessibility::FocusManager::new(),
                keyboard_nav: crate::accessibility::KeyboardNavigationManager::new(),
                screen_reader: crate::accessibility::ScreenReaderAnnouncer::new(),
                chat_widget: ChatWidget::new(),
                help_dialog: HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config: TuiConfig::default(),
            },

            pending_operations: std::collections::HashMap::new(),
            subscriptions: vec![],
        }
    }

    #[test]
    fn test_app_model_initialization() {
        let model = create_test_model();
        assert_eq!(model.mode, AppMode::Chat);
        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_mode_change_update() {
        let model = create_test_model();
        let (new_model, commands) = model.update(AppMessage::ModeChanged(AppMode::Command));

        assert_eq!(new_model.mode, AppMode::Command);
        assert_eq!(new_model.previous_mode, AppMode::Chat);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_command_palette_toggle() {
        let model = create_test_model();
        let (new_model, commands) = model.update(AppMessage::CommandPaletteToggled);

        assert!(new_model.commands.command_palette_visible);
        assert!(commands.is_empty());

        let (final_model, _) = new_model.update(AppMessage::CommandPaletteToggled);
        assert!(!final_model.commands.command_palette_visible);
    }

    #[test]
    fn test_state_validation() {
        let mut model = create_test_model();
        // Create an invalid state
        model.sessions.session_count = 0;
        model.sessions.active_session_id = Some("invalid".to_string());

        assert!(model.validate().is_err());
    }

    #[test]
    fn test_structural_sharing() {
        let model = create_test_model();
        let new_model = model.with_mode(AppMode::Command);

        assert_eq!(new_model.mode, AppMode::Command);
        assert_eq!(new_model.previous_mode, AppMode::Chat);
        // Other fields should be shared
        assert_eq!(new_model.theme, model.theme);
    }

    #[test]
    fn test_reactive_state_management() {
        let model = create_test_model();
        let mut reactive = ReactiveState::new(model);

        let diff = reactive.update(AppMessage::ModeChanged(AppMode::Command)).unwrap();
        assert!(diff.has_change(&StateChange::Mode(AppMode::Command)));

        assert!(reactive.can_undo());
        let undo_diff = reactive.undo().unwrap();
        assert_eq!(reactive.current().mode, AppMode::Chat);
    }

    #[test]
    fn test_state_diff_calculation() {
        let model1 = create_test_model();
        let model2 = model1.clone().with_mode(AppMode::Command);

        let diff = model2.diff(&model1);
        assert!(diff.has_change(&StateChange::Mode(AppMode::Command)));
        assert!(diff.mode_changed() == Some(AppMode::Command));
    }

    #[test]
    fn test_atomic_transitions() {
        let model = create_test_model();

        let result = model.transition(|m| m.with_mode(AppMode::Command));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().mode, AppMode::Command);
    }
}

