use ricecoder_keybinds::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_registry_creation() {
        let registry = ParserRegistry::new();
        assert!(registry.get_parser("json").is_some());
        assert!(registry.get_parser("markdown").is_some());
        assert!(registry.get_parser("md").is_some());
    }

    #[test]
    fn test_parser_registry_parse_json() {
        let registry = ParserRegistry::new();
        let json = r#"{
            "version": "1.0",
            "keybinds": [
                {
                    "action_id": "editor.save",
                    "key": "Ctrl+S",
                    "category": "editing",
                    "description": "Save file",
                    "is_default": true
                }
            ]
        }"#;

        let keybinds = registry.parse(json, "json").unwrap();
        assert_eq!(keybinds.len(), 1);
        assert_eq!(keybinds[0].action_id, "editor.save");
    }

    #[test]
    fn test_parser_registry_parse_markdown() {
        let registry = ParserRegistry::new();
        let markdown = r#"# Keybinds

## Editing

- `editor.save`: Ctrl+S - Save file
"#;

        let keybinds = registry.parse(markdown, "markdown").unwrap();
        assert_eq!(keybinds.len(), 1);
        assert_eq!(keybinds[0].action_id, "editor.save");
    }

    #[test]
    fn test_parser_registry_auto_detect_json() {
        let registry = ParserRegistry::new();
        let json = r#"{
            "version": "1.0",
            "keybinds": [
                {
                    "action_id": "editor.save",
                    "key": "Ctrl+S",
                    "category": "editing",
                    "description": "Save file",
                    "is_default": true
                }
            ]
        }"#;

        let keybinds = registry.parse_auto(json).unwrap();
        assert_eq!(keybinds.len(), 1);
    }

    #[test]
    fn test_parser_registry_auto_detect_markdown() {
        let registry = ParserRegistry::new();
        let markdown = r#"# Keybinds

## Editing

- `editor.save`: Ctrl+S - Save file
"#;

        let keybinds = registry.parse_auto(markdown).unwrap();
        assert_eq!(keybinds.len(), 1);
    }

    #[test]
    fn test_json_parser_valid() {
        let json = r#"{
            "version": "1.0",
            "keybinds": [
                {
                    "action_id": "editor.save",
                    "key": "Ctrl+S",
                    "category": "editing",
                    "description": "Save file",
                    "is_default": true
                }
            ]
        }"#;

        let parser = JsonKeybindParser;
        let keybinds = parser.parse(json).unwrap();
        assert_eq!(keybinds.len(), 1);
        assert_eq!(keybinds[0].action_id, "editor.save");
        assert_eq!(keybinds[0].key, "Ctrl+S");
        assert!(keybinds[0].is_default);
    }

    #[test]
    fn test_json_parser_invalid() {
        let json = "invalid json";
        let parser = JsonKeybindParser;
        assert!(parser.parse(json).is_err());
    }

    #[test]
    fn test_json_parser_missing_keybinds() {
        let json = r#"{"version": "1.0"}"#;
        let parser = JsonKeybindParser;
        assert!(parser.parse(json).is_err());
    }

    #[test]
    fn test_markdown_parser_valid() {
        let markdown = r#"# Keybinds

## Editing

- `editor.save`: Ctrl+S - Save file
- `editor.undo`: Ctrl+Z - Undo

## Navigation

- `nav.next`: Tab - Next item
"#;

        let parser = MarkdownKeybindParser;
        let keybinds = parser.parse(markdown).unwrap();
        assert_eq!(keybinds.len(), 3);
        assert_eq!(keybinds[0].action_id, "editor.save");
        assert_eq!(keybinds[0].category, "Editing");
        assert_eq!(keybinds[2].category, "Navigation");
    }

    #[test]
    fn test_markdown_parser_empty() {
        let markdown = "# Keybinds\n\n## Editing\n";
        let parser = MarkdownKeybindParser;
        let keybinds = parser.parse(markdown).unwrap();
        assert_eq!(keybinds.len(), 0);
    }
}
