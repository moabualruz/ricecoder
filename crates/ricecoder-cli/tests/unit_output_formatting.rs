// Unit tests for output formatting

use ricecoder_cli::output::OutputStyle;

#[test]
fn test_success_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.success("Operation successful");
    assert!(result.contains("✓"));
    assert!(result.contains("Operation successful"));
}

#[test]
fn test_error_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.error("Operation failed");
    assert!(result.contains("✗"));
    assert!(result.contains("Operation failed"));
}

#[test]
fn test_warning_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.warning("Warning message");
    assert!(result.contains("⚠"));
    assert!(result.contains("Warning message"));
}

#[test]
fn test_info_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.info("Info message");
    assert!(result.contains("ℹ"));
    assert!(result.contains("Info message"));
}

#[test]
fn test_code_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.code("let x = 42;");
    assert_eq!(result, "let x = 42;");
}

#[test]
fn test_code_block_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.code_block("fn main() {}", "rust");
    assert_eq!(result, "fn main() {}");
}

#[test]
fn test_prompt_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.prompt("r[");
    assert!(result.contains("r["));
}

#[test]
fn test_header_formatting() {
    let style = OutputStyle { use_colors: false };
    let result = style.header("Configuration");
    assert_eq!(result, "Configuration");
}

#[test]
fn test_error_with_suggestion() {
    let style = OutputStyle { use_colors: false };
    let result = style.error_with_suggestion("File not found", "Check the path");
    assert!(result.contains("✗ File not found"));
    assert!(result.contains("Suggestion: Check the path"));
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
fn test_formatting_idempotence() {
    let style = OutputStyle { use_colors: false };
    let msg = "test message";
    let formatted1 = style.success(msg);
    let formatted2 = style.success(msg);
    assert_eq!(formatted1, formatted2);
}

#[test]
fn test_all_formatting_methods_idempotent() {
    let style = OutputStyle { use_colors: false };
    let msg = "test";

    assert_eq!(style.success(msg), style.success(msg));
    assert_eq!(style.error(msg), style.error(msg));
    assert_eq!(style.warning(msg), style.warning(msg));
    assert_eq!(style.info(msg), style.info(msg));
    assert_eq!(style.code(msg), style.code(msg));
    assert_eq!(style.prompt(msg), style.prompt(msg));
    assert_eq!(style.header(msg), style.header(msg));
}
