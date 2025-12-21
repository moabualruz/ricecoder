use ricecoder_tui::*;

mod tests {
    use super::*;

    fn create_test_model() -> AppModel {
        AppModel {
            mode: AppMode::Chat,
            previous_mode: AppMode::Chat,
            theme: Theme::default(),
            terminal_caps: TerminalCapabilities::default(),

            sessions: SessionState {
                active_session_id: Some("test-session".to_string()),
                session_count: 1,
                total_tokens: TokenUsage::default(),
            },

            commands: CommandState {
                command_history: vec![],
                current_command: "".to_string(),
                command_palette_visible: false,
            },

            ui: UiState {
                focus_manager: FocusManager::new(),
                keyboard_nav: KeyboardNavigationManager::new(),
                screen_reader: ScreenReaderAnnouncer::new(),
                chat_widget: ChatWidget::new(),
                help_dialog: HelpDialog::default_ricecoder(),
                file_picker_visible: false,
                config: TuiConfig::default(),
            },

            pending_operations: HashMap::new(),
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
