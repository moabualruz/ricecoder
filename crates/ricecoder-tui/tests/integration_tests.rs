use ricecoder_tui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_container_creation() {
        let container = WidgetContainer::new();
        assert!(container.chat.messages.is_empty());
        assert!(container.diff.hunks.is_empty());
        assert!(container.dialog.is_none());
    }

    #[test]
    fn test_widget_container_reset() {
        let mut container = WidgetContainer::new();
        container
            .chat
            .add_message(crate::widgets::Message::user("test"));
        assert_eq!(container.chat.messages.len(), 1);

        container.reset_all();
        assert!(container.chat.messages.is_empty());
    }

    #[test]
    fn test_layout_coordinator_creation() {
        let coordinator = LayoutCoordinator::new(80, 24);
        assert_eq!(coordinator.width, 80);
        assert_eq!(coordinator.height, 24);
        assert!(coordinator.is_valid());
    }

    #[test]
    fn test_layout_coordinator_invalid_size() {
        let coordinator = LayoutCoordinator::new(40, 12);
        assert!(!coordinator.is_valid());
    }

    #[test]
    fn test_layout_coordinator_chat_layout() {
        let coordinator = LayoutCoordinator::new(80, 24);
        let layout = coordinator.layout_chat().unwrap();
        assert_eq!(layout.chat_area.width, 80);
        assert_eq!(layout.prompt_area.height, 3);
    }

    #[test]
    fn test_layout_coordinator_diff_layout() {
        let coordinator = LayoutCoordinator::new(80, 24);
        let layout = coordinator.layout_diff().unwrap();
        assert_eq!(layout.diff_area.width, 80);
        assert_eq!(layout.prompt_area.height, 3);
    }

    #[test]
    fn test_layout_coordinator_command_layout() {
        let coordinator = LayoutCoordinator::new(80, 24);
        let layout = coordinator.layout_command().unwrap();
        assert_eq!(layout.menu_area.width, 80);
        assert_eq!(layout.prompt_area.height, 3);
    }

    #[test]
    fn test_layout_coordinator_help_layout() {
        let coordinator = LayoutCoordinator::new(80, 24);
        let layout = coordinator.layout_help().unwrap();
        assert_eq!(layout.help_area.width, 80);
        assert_eq!(layout.prompt_area.height, 3);
    }

    #[test]
    fn test_layout_coordinator_update_size() {
        let mut coordinator = LayoutCoordinator::new(80, 24);
        coordinator.update_size(120, 40);
        assert_eq!(coordinator.width, 120);
        assert_eq!(coordinator.height, 40);
    }

    #[test]
    fn test_widget_integration_creation() {
        let integration = WidgetIntegration::new(80, 24);
        assert_eq!(integration.layout.width, 80);
        assert_eq!(integration.layout.height, 24);
    }

    #[test]
    fn test_widget_integration_on_resize() {
        let mut integration = WidgetIntegration::new(80, 24);
        let result = integration.on_resize(100, 30);
        assert!(result.is_ok());
        assert_eq!(integration.layout.width, 100);
        assert_eq!(integration.layout.height, 30);
    }

    #[test]
    fn test_widget_integration_on_resize_invalid() {
        let mut integration = WidgetIntegration::new(80, 24);
        let result = integration.on_resize(40, 12);
        assert!(result.is_err());
    }

    #[test]
    fn test_widget_integration_mode_switch() {
        let mut integration = WidgetIntegration::new(80, 24);
        let result = integration.on_mode_switch(AppMode::Chat, AppMode::Diff);
        assert!(result.is_ok());
        assert_eq!(integration.widgets.prompt.context.mode, AppMode::Diff);
    }

    #[test]
    fn test_widget_integration_get_layout_chat() {
        let integration = WidgetIntegration::new(80, 24);
        let layout = integration.get_layout(AppMode::Chat);
        assert!(layout.is_ok());
        match layout.unwrap() {
            LayoutInfo::Chat(_) => {}
            _ => panic!("Expected Chat layout"),
        }
    }

    #[test]
    fn test_widget_integration_get_layout_diff() {
        let integration = WidgetIntegration::new(80, 24);
        let layout = integration.get_layout(AppMode::Diff);
        assert!(layout.is_ok());
        match layout.unwrap() {
            LayoutInfo::Diff(_) => {}
            _ => panic!("Expected Diff layout"),
        }
    }

    #[test]
    fn test_widget_integration_get_layout_command() {
        let integration = WidgetIntegration::new(80, 24);
        let layout = integration.get_layout(AppMode::Command);
        assert!(layout.is_ok());
        match layout.unwrap() {
            LayoutInfo::Command(_) => {}
            _ => panic!("Expected Command layout"),
        }
    }

    #[test]
    fn test_widget_integration_get_layout_help() {
        let integration = WidgetIntegration::new(80, 24);
        let layout = integration.get_layout(AppMode::Help);
        assert!(layout.is_ok());
        match layout.unwrap() {
            LayoutInfo::Help(_) => {}
            _ => panic!("Expected Help layout"),
        }
    }

    #[test]
    fn test_state_synchronizer_sync_chat_to_prompt() {
        let chat = crate::widgets::ChatWidget::new();
        let mut prompt = crate::prompt::PromptWidget::new();
        StateSynchronizer::sync_chat_to_prompt(&chat, &mut prompt);
        // Should not panic
    }

    #[test]
    fn test_state_synchronizer_sync_prompt_to_chat() {
        let prompt = crate::prompt::PromptWidget::new();
        let mut chat = crate::widgets::ChatWidget::new();
        StateSynchronizer::sync_prompt_to_chat(&prompt, &mut chat);
        // Should not panic
    }

    #[test]
    fn test_state_synchronizer_sync_diff_to_prompt() {
        let diff = crate::diff::DiffWidget::new();
        let mut prompt = crate::prompt::PromptWidget::new();
        StateSynchronizer::sync_diff_to_prompt(&diff, &mut prompt);
        // Should not panic
    }

    #[test]
    fn test_layout_info_variants() {
        let chat_layout = ChatLayout {
            chat_area: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 21,
            },
            prompt_area: Rect {
                x: 0,
                y: 21,
                width: 80,
                height: 3,
            },
        };
        let _info = LayoutInfo::Chat(chat_layout);

        let diff_layout = DiffLayout {
            diff_area: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 21,
            },
            prompt_area: Rect {
                x: 0,
                y: 21,
                width: 80,
                height: 3,
            },
        };
        let _info = LayoutInfo::Diff(diff_layout);

        let command_layout = CommandLayout {
            menu_area: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 21,
            },
            prompt_area: Rect {
                x: 0,
                y: 21,
                width: 80,
                height: 3,
            },
        };
        let _info = LayoutInfo::Command(command_layout);

        let help_layout = HelpLayout {
            help_area: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 21,
            },
            prompt_area: Rect {
                x: 0,
                y: 21,
                width: 80,
                height: 3,
            },
        };
        let _info = LayoutInfo::Help(help_layout);
    }
}
