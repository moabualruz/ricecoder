use ricecoder_keybinds::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_all() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let output = KeybindHelp::display_all(&keybind_refs);

        assert!(output.contains("# All Keybinds"));
        assert!(output.contains("## editing"));
        assert!(output.contains("## navigation"));
        assert!(output.contains("editor.save"));
    }

    #[test]
    fn test_display_by_category() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let output = KeybindHelp::display_by_category(&keybind_refs, "editing");

        assert!(output.contains("# editing Keybinds"));
        assert!(output.contains("editor.save"));
        assert!(output.contains("editor.undo"));
        assert!(!output.contains("nav.next"));
    }

    #[test]
    fn test_search() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save file"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo change"),
            Keybind::new("nav.next", "Tab", "navigation", "Next item"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let results = KeybindHelp::search(&keybind_refs, "save");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action_id, "editor.save");
    }

    #[test]
    fn test_search_by_key() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save file"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo change"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let results = KeybindHelp::search(&keybind_refs, "Ctrl+S");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action_id, "editor.save");
    }

    #[test]
    fn test_paginate() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
            Keybind::new("nav.next", "Tab", "navigation", "Next"),
            Keybind::new("nav.prev", "Shift+Tab", "navigation", "Previous"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let page = KeybindHelp::paginate(&keybind_refs, 1, 2);

        assert_eq!(page.current_page, 1);
        assert_eq!(page.total_pages, 2);
        assert_eq!(page.items.len(), 2);
    }

    #[test]
    fn test_paginate_invalid_page() {
        let keybinds = vec![
            Keybind::new("editor.save", "Ctrl+S", "editing", "Save"),
            Keybind::new("editor.undo", "Ctrl+Z", "editing", "Undo"),
        ];

        let keybind_refs: Vec<&Keybind> = keybinds.iter().collect();
        let page = KeybindHelp::paginate(&keybind_refs, 10, 2);

        assert_eq!(page.items.len(), 0);
    }
}