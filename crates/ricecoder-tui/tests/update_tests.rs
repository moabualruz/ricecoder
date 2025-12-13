use ricecoder_tui::*;
use crate::widgets::ChatWidget;
use ricecoder_help::HelpDialog;

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

            pending_operations: std::collections::HashMap::new(),
            subscriptions: vec![],
        }
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
    fn test_global_keybindings() {
        let model = create_test_model();

        // Test Ctrl+1 for Chat mode
        let key = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::CONTROL);
        let (_, commands) = model.clone().update(AppMessage::KeyPress(key));
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::SwitchMode(AppMode::Chat)));

        // Test Ctrl+C for exit
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let (_, commands) = model.update(AppMessage::KeyPress(key));
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::Exit));
    }

    #[test]
    fn test_command_input() {
        let model = create_test_model();

        // Type a character
        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty());
        let (new_model, commands) = model.update(AppMessage::KeyPress(key));
        assert_eq!(new_model.commands.current_command, "l");
        assert!(commands.is_empty());

        // Press Enter to execute
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let (final_model, commands) = new_model.update(AppMessage::KeyPress(key));
        assert_eq!(final_model.commands.current_command, "");
        assert_eq!(final_model.commands.command_history, vec!["l"]);
        assert_eq!(commands.len(), 1);
        assert!(matches!(commands[0], Command::ExecuteCommand(cmd) if cmd == "l"));
    }

    #[test]
    fn test_session_operations() {
        let model = create_test_model();

        // Create session
        let (model, _) = model.update(AppMessage::SessionCreated("new-session".to_string()));
        assert_eq!(model.sessions.session_count, 2);
        assert_eq!(model.sessions.active_session_id, Some("new-session".to_string()));

        // Close session
        let (model, _) = model.update(AppMessage::SessionClosed("new-session".to_string()));
        assert_eq!(model.sessions.session_count, 1);
        assert_eq!(model.sessions.active_session_id, None);
    }

    #[test]
    fn test_operation_lifecycle() {
        let model = create_test_model();
        let op = PendingOperation {
            id: "test-op".to_string(),
            description: "Test operation".to_string(),
            start_time: std::time::Instant::now(),
        };

        // Start operation
        let (model, _) = model.update(AppMessage::OperationStarted(op.clone()));
        assert!(model.pending_operations.contains_key("test-op"));

        // Complete operation
        let (model, _) = model.update(AppMessage::OperationCompleted("test-op".to_string()));
        assert!(!model.pending_operations.contains_key("test-op"));
    }
}