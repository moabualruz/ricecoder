use ricecoder_tui::*;
use crate::widgets::ChatWidget;
use ricecoder_help::HelpDialog;

mod tests {
    use super::*;

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