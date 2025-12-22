//! Property-based tests for standards detection
//! **Feature: ricecoder-research, Property 6: Standards Detection Consistency**
//! **Validates: Requirements 1.5**

use std::io::Write;

use proptest::prelude::*;
use ricecoder_research::StandardsDetector;
use tempfile::NamedTempFile;

// ============================================================================
// Generators for property testing
// ============================================================================

/// Generate valid Rust code with consistent naming conventions
fn arb_rust_code_snake_case() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z_][a-z0-9_]{0,20}")
        .expect("valid regex")
        .prop_map(|name| format!("fn {}_function() {{\n    let {} = 42;\n}}\n", name, name))
}

/// Generate valid Rust code with consistent formatting
fn arb_rust_code_with_formatting() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {\n    println!(\"hello\");\n}".to_string()),
        Just("fn test() {\n    let x = 1;\n    let y = 2;\n}".to_string()),
        Just("pub struct MyStruct {\n    field: i32,\n}".to_string()),
    ]
}

/// Generate valid Rust code with documentation
fn arb_rust_code_with_docs() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("/// This is a doc comment\npub fn my_function() {}".to_string()),
        Just("/// Another doc\npub struct MyStruct {}".to_string()),
        Just("/// Documented enum\npub enum MyEnum {}".to_string()),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Standards detection consistency
    /// For any codebase, analyzing the same code multiple times SHALL produce identical standards profiles
    #[test]
    fn prop_standards_detection_consistency(code in arb_rust_code_with_formatting()) {
        let detector = StandardsDetector::new();

        // Create temporary files with the same code
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(code.as_bytes()).expect("failed to write file");
        file2.write_all(code.as_bytes()).expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze the same code twice
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Both analyses should produce identical results
        prop_assert_eq!(result1, result2, "standards detection should be consistent");
    }

    /// Property: Naming convention consistency
    /// For any code with consistent naming patterns, the detected naming conventions should be stable
    #[test]
    fn prop_naming_convention_consistency(code in arb_rust_code_snake_case()) {
        let detector = StandardsDetector::new();

        // Create temporary files
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(code.as_bytes()).expect("failed to write file");
        file2.write_all(code.as_bytes()).expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze the same code twice
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Naming conventions should be identical
        prop_assert_eq!(
            result1.naming_conventions,
            result2.naming_conventions,
            "naming conventions should be consistent"
        );
    }

    /// Property: Formatting style consistency
    /// For any code with consistent formatting, the detected formatting style should be stable
    #[test]
    fn prop_formatting_style_consistency(code in arb_rust_code_with_formatting()) {
        let detector = StandardsDetector::new();

        // Create temporary files
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(code.as_bytes()).expect("failed to write file");
        file2.write_all(code.as_bytes()).expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze the same code twice
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Formatting style should be identical
        prop_assert_eq!(
            result1.formatting_style,
            result2.formatting_style,
            "formatting style should be consistent"
        );
    }

    /// Property: Documentation style consistency
    /// For any code with consistent documentation patterns, the detected documentation style should be stable
    #[test]
    fn prop_documentation_style_consistency(code in arb_rust_code_with_docs()) {
        let detector = StandardsDetector::new();

        // Create temporary files
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(code.as_bytes()).expect("failed to write file");
        file2.write_all(code.as_bytes()).expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze the same code twice
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Documentation style should be identical
        prop_assert_eq!(
            result1.documentation_style,
            result2.documentation_style,
            "documentation style should be consistent"
        );
    }

    /// Property: Import organization consistency
    /// For any code with consistent import patterns, the detected import organization should be stable
    #[test]
    fn prop_import_organization_consistency(
        _dummy in Just(())
    ) {
        let detector = StandardsDetector::new();

        let code = "use std::io;\nuse external_crate;\nuse crate::module;";

        // Create temporary files
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(code.as_bytes()).expect("failed to write file");
        file2.write_all(code.as_bytes()).expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze the same code twice
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Import organization should be identical
        prop_assert_eq!(
            result1.import_organization,
            result2.import_organization,
            "import organization should be consistent"
        );
    }

    /// Property: Empty file handling consistency
    /// For empty files, standards detection should produce default standards consistently
    #[test]
    fn prop_empty_file_consistency(_dummy in Just(())) {
        let detector = StandardsDetector::new();

        // Create temporary files
        let mut file1 = NamedTempFile::new().expect("failed to create temp file");
        let mut file2 = NamedTempFile::new().expect("failed to create temp file");

        file1.write_all(b"").expect("failed to write file");
        file2.write_all(b"").expect("failed to write file");

        let path1 = file1.path();
        let path2 = file2.path();

        // Analyze empty files
        let result1 = detector.detect(&[path1]).expect("first analysis failed");
        let result2 = detector.detect(&[path2]).expect("second analysis failed");

        // Both should produce identical default standards
        prop_assert_eq!(result1, result2, "empty file analysis should be consistent");
    }
}
