use ricecoder_tui::*;

mod tests {
    use super::*;

    #[test]
    fn test_command_palette_creation() {
        let palette = CommandPaletteWidget::new();
        assert!(!palette.is_visible());
        assert_eq!(palette.query(), "");
        assert_eq!(palette.filtered_count(), 0);
    }

    #[test]
    fn test_add_commands() {
        let mut palette = CommandPaletteWidget::new();

        let cmd1 = PaletteCommand {
            name: "help".to_string(),
            display_name: "Help".to_string(),
            description: "Show help information".to_string(),
            shortcut: Some("F1".to_string()),
            category: "General".to_string(),
        };

        let cmd2 = PaletteCommand {
            name: "quit".to_string(),
            display_name: "Quit".to_string(),
            description: "Exit the application".to_string(),
            shortcut: Some("Ctrl+Q".to_string()),
            category: "General".to_string(),
        };

        palette.add_commands(vec![cmd1, cmd2]);
        assert_eq!(palette.filtered_count(), 2);
    }

    #[test]
    fn test_fuzzy_matching() {
        assert_eq!(fuzzy_match("he", "help"), Some(vec![(0, 2)]));
        assert_eq!(fuzzy_match("hp", "help"), Some(vec![(0, 1), (3, 4)]));
        assert_eq!(fuzzy_match("xyz", "help"), None);
        assert_eq!(fuzzy_match("", "help"), Some(vec![]));
    }

    #[test]
    fn test_selection_navigation() {
        let mut palette = CommandPaletteWidget::new();

        let cmd = PaletteCommand {
            name: "test".to_string(),
            display_name: "Test".to_string(),
            description: "Test command".to_string(),
            shortcut: None,
            category: "Test".to_string(),
        };

        palette.add_command(cmd);
        palette.show();

        assert_eq!(palette.selected_index, 0);
        palette.select_down();
        assert_eq!(palette.selected_index, 0); // Can't go down with only one item

        palette.select_up();
        assert_eq!(palette.selected_index, 0); // Can't go up from first item
    }
}