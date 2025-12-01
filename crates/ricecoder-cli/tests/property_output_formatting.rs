// Property-based test for output formatting idempotence
// **Feature: ricecoder-cli, Property 3: Output Formatting Idempotence**
// **Validates: Requirements 3.1**

use proptest::prelude::*;
use ricecoder_cli::output::OutputStyle;

/// Property 3: Output Formatting Idempotence
/// For any output content, formatting the same content twice SHALL produce identical results.
#[test]
fn prop_output_formatting_idempotence() {
    proptest!(|(msg in ".*")| {
        let style = OutputStyle { use_colors: false };
        
        // Format the message twice
        let formatted1 = style.success(&msg);
        let formatted2 = style.success(&msg);
        
        // They should be identical
        prop_assert_eq!(formatted1, formatted2);
    });
}

/// Test that formatting without colors is deterministic
#[test]
fn test_formatting_without_colors_deterministic() {
    let style = OutputStyle { use_colors: false };
    let msg = "test message";
    
    let formatted1 = style.success(msg);
    let formatted2 = style.success(msg);
    let formatted3 = style.success(msg);
    
    assert_eq!(formatted1, formatted2);
    assert_eq!(formatted2, formatted3);
}

/// Test that all formatting methods are idempotent
#[test]
fn test_all_formatting_methods_idempotent() {
    let style = OutputStyle { use_colors: false };
    let msg = "test";
    
    // Test success
    assert_eq!(style.success(msg), style.success(msg));
    
    // Test error
    assert_eq!(style.error(msg), style.error(msg));
    
    // Test warning
    assert_eq!(style.warning(msg), style.warning(msg));
    
    // Test info
    assert_eq!(style.info(msg), style.info(msg));
    
    // Test code
    assert_eq!(style.code(msg), style.code(msg));
    
    // Test prompt
    assert_eq!(style.prompt(msg), style.prompt(msg));
    
    // Test header
    assert_eq!(style.header(msg), style.header(msg));
}

/// Test that formatting produces consistent output structure
#[test]
fn test_formatting_output_structure() {
    let style = OutputStyle { use_colors: false };
    
    // Success should start with ✓
    assert!(style.success("test").starts_with("✓"));
    
    // Error should start with ✗
    assert!(style.error("test").starts_with("✗"));
    
    // Warning should start with ⚠
    assert!(style.warning("test").starts_with("⚠"));
    
    // Info should start with ℹ
    assert!(style.info("test").starts_with("ℹ"));
}
