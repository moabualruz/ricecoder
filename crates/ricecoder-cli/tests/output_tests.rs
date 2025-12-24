use ricecoder_cli::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_style_without_colors() {
        let style = OutputStyle { use_colors: false };
        assert_eq!(style.success("test"), "✓ test");
        assert_eq!(style.error("test"), "✗ test");
        assert_eq!(style.warning("test"), "⚠	test");
        assert_eq!(style.info("test"), "ℹ test");
    }

    #[test]
    fn test_output_formatting_idempotence() {
        let style = OutputStyle { use_colors: false };
        let msg = "test message";
        let formatted1 = style.success(msg);
        let formatted2 = style.success(msg);
        assert_eq!(formatted1, formatted2);
    }

    #[test]
    fn test_error_with_suggestion() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_suggestion("File not found", "Check the file path");
        assert!(result.contains("✗ File not found"));
        assert!(result.contains("Suggestion: Check the file path"));
    }

    #[test]
    fn test_error_with_context() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_context("Invalid config", "in ~/.ricecoder/config.toml");
        assert!(result.contains("✗ Invalid config"));
        assert!(result.contains("Context: in ~/.ricecoder/config.toml"));
    }

    #[test]
    fn test_section_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.section("Configuration");
        assert!(result.contains("Configuration"));
        assert!(result.contains("─"));
    }

    #[test]
    fn test_list_item_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.list_item("First item");
        assert!(result.contains("•"));
        assert!(result.contains("First item"));
    }

    #[test]
    fn test_key_value_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.key_value("key", "value");
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_error_with_suggestions() {
        let style = OutputStyle { use_colors: false };
        let suggestions = vec!["Try this", "Or that"];
        let result = style.error_with_suggestions("Something failed", &suggestions);
        assert!(result.contains("✗ Something failed"));
        assert!(result.contains("Suggestions:"));
        assert!(result.contains("1. Try this"));
        assert!(result.contains("2. Or that"));
    }

    #[test]
    fn test_error_with_docs() {
        let style = OutputStyle { use_colors: false };
        let result = style.error_with_docs("File not found", "https://docs.example.com");
        assert!(result.contains("✗ File not found"));
        assert!(result.contains("https://docs.example.com"));
    }

    #[test]
    fn test_numbered_item_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.numbered_item(1, "First item");
        assert!(result.contains("1. First item"));
    }

    #[test]
    fn test_tip_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.tip("This is a helpful tip");
        assert!(result.contains("💡"));
        assert!(result.contains("This is a helpful tip"));
    }

    #[test]
    fn test_link_formatting() {
        let style = OutputStyle { use_colors: false };
        let result = style.link("Documentation", "https://docs.example.com");
        assert!(result.contains("Documentation"));
        assert!(result.contains("https://docs.example.com"));
    }
}
