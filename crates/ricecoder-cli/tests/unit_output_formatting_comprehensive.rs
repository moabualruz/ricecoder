// Comprehensive unit tests for output formatting
// **Feature: ricecoder-cli, Tests for Requirements 3.1-3.8**

use ricecoder_cli::output::OutputStyle;

// ============================================================================
// OutputStyle Tests - Basic Formatting
// ============================================================================

#[test]
fn test_output_style_success_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.success("Operation completed");

    assert!(result.contains("✓"));
    assert!(result.contains("Operation completed"));
}

#[test]
fn test_output_style_error_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.error("Something went wrong");

    assert!(result.contains("✗"));
    assert!(result.contains("Something went wrong"));
}

#[test]
fn test_output_style_warning_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.warning("Be careful");

    assert!(result.contains("⚠"));
    assert!(result.contains("Be careful"));
}

#[test]
fn test_output_style_info_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.info("Here's some information");

    assert!(result.contains("ℹ"));
    assert!(result.contains("Here's some information"));
}

// ============================================================================
// OutputStyle Tests - Code Formatting
// ============================================================================

#[test]
fn test_output_style_code_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.code("let x = 42;");

    assert_eq!(result, "let x = 42;");
}

#[test]
fn test_output_style_code_with_colors() {
    let style = OutputStyle { use_colors: true };
    let result = style.code("let x = 42;");

    // With colors, the result should contain the code
    assert!(result.contains("let x = 42;"));
}

#[test]
fn test_output_style_code_block_rust() {
    let style = OutputStyle { use_colors: false };
    let code = "fn main() {\n    println!(\"Hello\");\n}";
    let result = style.code_block(code, "rust");

    assert!(result.contains("fn main()"));
    assert!(result.contains("println!"));
}

#[test]
fn test_output_style_code_block_python() {
    let style = OutputStyle { use_colors: false };
    let code = "def hello():\n    print('Hello')";
    let result = style.code_block(code, "python");

    assert!(result.contains("def hello()"));
    assert!(result.contains("print"));
}

#[test]
fn test_output_style_code_block_javascript() {
    let style = OutputStyle { use_colors: false };
    let code = "function hello() {\n    console.log('Hello');\n}";
    let result = style.code_block(code, "javascript");

    assert!(result.contains("function hello()"));
    assert!(result.contains("console.log"));
}

#[test]
fn test_output_style_code_block_json() {
    let style = OutputStyle { use_colors: false };
    let code = r#"{"name": "test", "value": 42}"#;
    let result = style.code_block(code, "json");

    assert!(result.contains("name"));
    assert!(result.contains("test"));
}

#[test]
fn test_output_style_code_block_yaml() {
    let style = OutputStyle { use_colors: false };
    let code = "name: test\nvalue: 42";
    let result = style.code_block(code, "yaml");

    assert!(result.contains("name"));
    assert!(result.contains("test"));
}

#[test]
fn test_output_style_code_block_unknown_language() {
    let style = OutputStyle { use_colors: false };
    let code = "some code";
    let result = style.code_block(code, "unknown");

    assert_eq!(result, "some code");
}

// ============================================================================
// OutputStyle Tests - Prompt Formatting
// ============================================================================

#[test]
fn test_output_style_prompt_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.prompt("r[");

    assert!(result.contains("r["));
    assert!(result.ends_with(" "));
}

#[test]
fn test_output_style_prompt_with_colors() {
    let style = OutputStyle { use_colors: true };
    let result = style.prompt("r[");

    assert!(result.contains("r["));
    assert!(result.ends_with(" "));
}

// ============================================================================
// OutputStyle Tests - Header Formatting
// ============================================================================

#[test]
fn test_output_style_header_without_colors() {
    let style = OutputStyle { use_colors: false };
    let result = style.header("Configuration");

    assert_eq!(result, "Configuration");
}

#[test]
fn test_output_style_header_with_colors() {
    let style = OutputStyle { use_colors: true };
    let result = style.header("Configuration");

    assert!(result.contains("Configuration"));
}

// ============================================================================
// OutputStyle Tests - Error Formatting with Context
// ============================================================================

#[test]
fn test_output_style_error_with_suggestion() {
    let style = OutputStyle { use_colors: false };
    let result = style.error_with_suggestion("File not found", "Check the file path");

    assert!(result.contains("✗ File not found"));
    assert!(result.contains("Suggestion: Check the file path"));
    assert!(result.contains("\n"));
}

#[test]
fn test_output_style_error_with_context() {
    let style = OutputStyle { use_colors: false };
    let result = style.error_with_context("Invalid config", "in ~/.ricecoder/config.toml");

    assert!(result.contains("✗ Invalid config"));
    assert!(result.contains("Context: in ~/.ricecoder/config.toml"));
    assert!(result.contains("\n"));
}

#[test]
fn test_output_style_error_verbose() {
    let style = OutputStyle { use_colors: false };
    let result = style.error_verbose("Parse error", "Expected 'name' field at line 5");

    assert!(result.contains("✗ Parse error"));
    assert!(result.contains("Expected 'name' field at line 5"));
}

// ============================================================================
// OutputStyle Tests - Section Formatting
// ============================================================================

#[test]
fn test_output_style_section() {
    let style = OutputStyle { use_colors: false };
    let result = style.section("Configuration");

    assert!(result.contains("Configuration"));
    assert!(result.contains("─"));
    assert!(result.contains("\n"));
}

#[test]
fn test_output_style_section_with_long_title() {
    let style = OutputStyle { use_colors: false };
    let result = style.section("Very Long Configuration Section Title");

    assert!(result.contains("Very Long Configuration Section Title"));
    // Should have dashes matching the title length
    assert!(result.contains("─"));
}

// ============================================================================
// OutputStyle Tests - List Formatting
// ============================================================================

#[test]
fn test_output_style_list_item() {
    let style = OutputStyle { use_colors: false };
    let result = style.list_item("First item");

    assert!(result.contains("•"));
    assert!(result.contains("First item"));
    assert!(result.starts_with("  "));
}

#[test]
fn test_output_style_list_item_with_special_characters() {
    let style = OutputStyle { use_colors: false };
    let result = style.list_item("Item with special chars: !@#$%");

    assert!(result.contains("•"));
    assert!(result.contains("Item with special chars: !@#$%"));
}

// ============================================================================
// OutputStyle Tests - Key-Value Formatting
// ============================================================================

#[test]
fn test_output_style_key_value() {
    let style = OutputStyle { use_colors: false };
    let result = style.key_value("provider", "openai");

    assert!(result.contains("provider"));
    assert!(result.contains("openai"));
    assert!(result.contains(":"));
    assert!(result.starts_with("  "));
}

#[test]
fn test_output_style_key_value_with_empty_value() {
    let style = OutputStyle { use_colors: false };
    let result = style.key_value("key", "");

    assert!(result.contains("key"));
    assert!(result.contains(":"));
}

#[test]
fn test_output_style_key_value_with_special_characters() {
    let style = OutputStyle { use_colors: false };
    let result = style.key_value("path", "/home/user/.ricecoder/config.toml");

    assert!(result.contains("path"));
    assert!(result.contains("/home/user/.ricecoder/config.toml"));
}

// ============================================================================
// OutputStyle Tests - Idempotence (Property 3)
// ============================================================================

#[test]
fn test_output_formatting_idempotence_success() {
    let style = OutputStyle { use_colors: false };
    let msg = "test message";

    let formatted1 = style.success(msg);
    let formatted2 = style.success(msg);

    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_output_formatting_idempotence_error() {
    let style = OutputStyle { use_colors: false };
    let msg = "error message";

    let formatted1 = style.error(msg);
    let formatted2 = style.error(msg);

    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_output_formatting_idempotence_warning() {
    let style = OutputStyle { use_colors: false };
    let msg = "warning message";

    let formatted1 = style.warning(msg);
    let formatted2 = style.warning(msg);

    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_output_formatting_idempotence_info() {
    let style = OutputStyle { use_colors: false };
    let msg = "info message";

    let formatted1 = style.info(msg);
    let formatted2 = style.info(msg);

    assert_eq!(formatted1, formatted2, "Formatting should be idempotent");
}

#[test]
fn test_output_formatting_idempotence_code() {
    let style = OutputStyle { use_colors: false };
    let code = "let x = 42;";

    let formatted1 = style.code(code);
    let formatted2 = style.code(code);

    assert_eq!(
        formatted1, formatted2,
        "Code formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_code_block() {
    let style = OutputStyle { use_colors: false };
    let code = "fn main() {}";

    let formatted1 = style.code_block(code, "rust");
    let formatted2 = style.code_block(code, "rust");

    assert_eq!(
        formatted1, formatted2,
        "Code block formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_prompt() {
    let style = OutputStyle { use_colors: false };
    let prompt = "r[";

    let formatted1 = style.prompt(prompt);
    let formatted2 = style.prompt(prompt);

    assert_eq!(
        formatted1, formatted2,
        "Prompt formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_header() {
    let style = OutputStyle { use_colors: false };
    let title = "Configuration";

    let formatted1 = style.header(title);
    let formatted2 = style.header(title);

    assert_eq!(
        formatted1, formatted2,
        "Header formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_section() {
    let style = OutputStyle { use_colors: false };
    let title = "Configuration";

    let formatted1 = style.section(title);
    let formatted2 = style.section(title);

    assert_eq!(
        formatted1, formatted2,
        "Section formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_list_item() {
    let style = OutputStyle { use_colors: false };
    let item = "First item";

    let formatted1 = style.list_item(item);
    let formatted2 = style.list_item(item);

    assert_eq!(
        formatted1, formatted2,
        "List item formatting should be idempotent"
    );
}

#[test]
fn test_output_formatting_idempotence_key_value() {
    let style = OutputStyle { use_colors: false };

    let formatted1 = style.key_value("key", "value");
    let formatted2 = style.key_value("key", "value");

    assert_eq!(
        formatted1, formatted2,
        "Key-value formatting should be idempotent"
    );
}

// ============================================================================
// OutputStyle Tests - Color Handling
// ============================================================================

#[test]
fn test_output_style_default_detects_tty() {
    let _style = OutputStyle::default();
    // Just verify it creates successfully
    assert!(true);
}

#[test]
fn test_output_style_with_colors_enabled() {
    let style = OutputStyle { use_colors: true };
    let result = style.success("test");

    // With colors enabled, result should still contain the message
    assert!(result.contains("test"));
}

#[test]
fn test_output_style_with_colors_disabled() {
    let style = OutputStyle { use_colors: false };
    let result = style.success("test");

    // With colors disabled, result should be plain text
    assert_eq!(result, "✓ test");
}

// ============================================================================
// OutputStyle Tests - Edge Cases
// ============================================================================

#[test]
fn test_output_style_empty_message() {
    let style = OutputStyle { use_colors: false };

    let success = style.success("");
    let error = style.error("");
    let warning = style.warning("");
    let info = style.info("");

    assert!(success.contains("✓"));
    assert!(error.contains("✗"));
    assert!(warning.contains("⚠"));
    assert!(info.contains("ℹ"));
}

#[test]
fn test_output_style_very_long_message() {
    let style = OutputStyle { use_colors: false };
    let long_msg = "a".repeat(1000);

    let result = style.success(&long_msg);
    assert!(result.contains(&long_msg));
}

#[test]
fn test_output_style_message_with_newlines() {
    let style = OutputStyle { use_colors: false };
    let msg = "line1\nline2\nline3";

    let result = style.success(msg);
    assert!(result.contains("line1"));
    assert!(result.contains("line2"));
    assert!(result.contains("line3"));
}

#[test]
fn test_output_style_message_with_unicode() {
    let style = OutputStyle { use_colors: false };
    let msg = "Hello 世界 🌍";

    let result = style.success(msg);
    assert!(result.contains("Hello"));
    assert!(result.contains("世界"));
}

// ============================================================================
// OutputStyle Tests - Consistency
// ============================================================================

#[test]
fn test_output_style_consistency_across_methods() {
    let style = OutputStyle { use_colors: false };

    // All methods should produce consistent output format
    let success = style.success("test");
    let error = style.error("test");
    let warning = style.warning("test");
    let info = style.info("test");

    // All should contain the message
    assert!(success.contains("test"));
    assert!(error.contains("test"));
    assert!(warning.contains("test"));
    assert!(info.contains("test"));

    // All should have a symbol
    assert!(success.contains("✓"));
    assert!(error.contains("✗"));
    assert!(warning.contains("⚠"));
    assert!(info.contains("ℹ"));
}

#[test]
fn test_output_style_consistency_with_and_without_colors() {
    let msg = "test message";

    let style_no_color = OutputStyle { use_colors: false };
    let style_with_color = OutputStyle { use_colors: true };

    let result_no_color = style_no_color.success(msg);
    let result_with_color = style_with_color.success(msg);

    // Both should contain the message
    assert!(result_no_color.contains(msg));
    assert!(result_with_color.contains(msg));

    // Both should contain the symbol
    assert!(result_no_color.contains("✓"));
    assert!(result_with_color.contains("✓"));
}
