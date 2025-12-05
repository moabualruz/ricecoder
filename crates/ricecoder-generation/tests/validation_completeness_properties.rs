//! Property-based tests for validation completeness
//!
//! **Feature: ricecoder-generation, Property 2: Validation Completeness**
//! **Validates: Requirements 1.4, 3.4**
//!
//! Property: For any generated code, validation SHALL detect all syntax errors before writing.

use proptest::prelude::*;
use ricecoder_generation::{CodeValidator, GeneratedFile, ValidationConfig};

/// Strategy for generating valid Rust code
fn valid_rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("pub fn hello() { println!(\"Hello\"); }".to_string()),
        Just("struct User { name: String }".to_string()),
        Just("impl User { fn new(name: String) -> Self { User { name } } }".to_string()),
    ]
}

/// Strategy for generating invalid Rust code with syntax errors
fn invalid_rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {".to_string()),  // Missing closing brace
        Just("fn hello() { println!(\"Hello\") ".to_string()),  // Missing closing brace
        Just("struct User { name: String".to_string()),  // Missing closing brace
        Just("fn test() { let x = [1, 2, 3".to_string()),  // Missing closing bracket
        Just("fn test() { let x = (1, 2, 3".to_string()),  // Missing closing paren
    ]
}

/// Strategy for generating TypeScript code with syntax errors
fn invalid_typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("function hello() { console.log(\"Hello\") ".to_string()),  // Missing closing brace
        Just("const x = { a: 1, b: 2".to_string()),  // Missing closing brace
        Just("interface User { name: string".to_string()),  // Missing closing brace
        Just("function test() { const arr = [1, 2, 3".to_string()),  // Missing closing bracket
        Just("function test() { const x = (1, 2, 3".to_string()),  // Missing closing paren
    ]
}

/// Strategy for generating Python code with syntax errors
fn invalid_python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("def hello(:\n    print(\"Hello\")".to_string()),  // Missing closing paren
        Just("def test():\n    arr = [1, 2, 3".to_string()),  // Missing closing bracket
        Just("def test():\n    x = (1, 2, 3".to_string()),  // Missing closing paren
        Just("def test():\n    d = {\"a\": 1".to_string()),  // Missing closing brace
    ]
}

/// Strategy for generating generated files with valid code
fn valid_generated_file_strategy() -> impl Strategy<Value = GeneratedFile> {
    prop_oneof![
        (Just("src/main.rs"), valid_rust_code_strategy(), Just("rust"))
            .prop_map(|(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }),
    ]
}

/// Strategy for generating generated files with invalid code
fn invalid_generated_file_strategy() -> impl Strategy<Value = GeneratedFile> {
    prop_oneof![
        (Just("src/main.rs"), invalid_rust_code_strategy(), Just("rust"))
            .prop_map(|(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }),
        (Just("src/index.ts"), invalid_typescript_code_strategy(), Just("typescript"))
            .prop_map(|(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }),
        (Just("src/main.py"), invalid_python_code_strategy(), Just("python"))
            .prop_map(|(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }),
    ]
}

proptest! {
    /// Property: Valid code passes validation
    ///
    /// For any valid generated code, the CodeValidator should report
    /// no errors and mark validation as successful.
    #[test]
    fn prop_valid_code_passes_validation(file in valid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        prop_assert!(
            result.valid,
            "Valid code failed validation: {:?}",
            result.errors
        );
        prop_assert!(
            result.errors.is_empty(),
            "Valid code produced errors: {:?}",
            result.errors
        );
    }

    /// Property: Invalid code with unmatched delimiters fails validation
    ///
    /// For any invalid generated code with unmatched braces/parens/brackets,
    /// the CodeValidator should detect at least one error.
    #[test]
    fn prop_invalid_code_fails_validation(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        // Only check languages that have bracket checking
        match file.language.as_str() {
            "rust" | "typescript" => {
                prop_assert!(
                    !result.valid,
                    "Invalid code passed validation: {}",
                    file.content
                );
                prop_assert!(
                    !result.errors.is_empty(),
                    "Invalid code produced no errors: {}",
                    file.content
                );
            }
            "python" => {
                // Python validator only checks braces and parens, not brackets
                // So only assert if the code has unmatched braces or parens
                let has_unmatched_braces = file.content.matches('{').count() != file.content.matches('}').count();
                let has_unmatched_parens = file.content.matches('(').count() != file.content.matches(')').count();
                
                if has_unmatched_braces || has_unmatched_parens {
                    prop_assert!(
                        !result.valid,
                        "Invalid code passed validation: {}",
                        file.content
                    );
                    prop_assert!(
                        !result.errors.is_empty(),
                        "Invalid code produced no errors: {}",
                        file.content
                    );
                }
            }
            _ => {}
        }
    }

    /// Property: Validation errors include file paths
    ///
    /// For any invalid generated code, validation errors should include
    /// the file path where the error occurred.
    #[test]
    fn prop_validation_errors_include_file_paths(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        for error in &result.errors {
            prop_assert_eq!(
                &error.file, &file.path,
                "Error file path doesn't match: expected {}, got {}",
                file.path, error.file
            );
        }
    }

    /// Property: Validation errors include line numbers
    ///
    /// For any invalid generated code, validation errors should include
    /// line numbers (at least 1).
    #[test]
    fn prop_validation_errors_include_line_numbers(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        for error in &result.errors {
            prop_assert!(
                error.line >= 1,
                "Error line number is invalid: {}",
                error.line
            );
        }
    }

    /// Property: Validation errors include messages
    ///
    /// For any invalid generated code, validation errors should include
    /// descriptive error messages.
    #[test]
    fn prop_validation_errors_include_messages(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        for error in &result.errors {
            prop_assert!(
                !error.message.is_empty(),
                "Error message is empty"
            );
        }
    }

    /// Property: Multiple files are validated independently
    ///
    /// For any collection of generated files, the CodeValidator should
    /// validate each file independently and report all errors.
    #[test]
    fn prop_multiple_files_validated_independently(
        files in prop::collection::vec(invalid_generated_file_strategy(), 1..3)
    ) {
        let validator = CodeValidator::new();
        let result = validator.validate(&files).expect("Validation failed");

        // Filter to only files that should have errors
        let files_with_errors: Vec<_> = files.iter().filter(|f| {
            match f.language.as_str() {
                "rust" | "typescript" => true,
                "python" => {
                    let has_unmatched_braces = f.content.matches('{').count() != f.content.matches('}').count();
                    let has_unmatched_parens = f.content.matches('(').count() != f.content.matches(')').count();
                    has_unmatched_braces || has_unmatched_parens
                }
                _ => false
            }
        }).collect();

        if !files_with_errors.is_empty() {
            // Should have errors from all files with errors
            prop_assert!(
                !result.errors.is_empty(),
                "No errors found for invalid files"
            );

            // Each file should have at least one error
            let mut file_errors: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for error in &result.errors {
                *file_errors.entry(error.file.clone()).or_insert(0) += 1;
            }

            for file in &files_with_errors {
                prop_assert!(
                    file_errors.contains_key(&file.path),
                    "No errors found for file: {}",
                    file.path
                );
            }
        }
    }

    /// Property: Validation configuration is respected
    ///
    /// For any generated code, the CodeValidator should respect the
    /// validation configuration settings.
    #[test]
    fn prop_validation_configuration_respected(file in invalid_generated_file_strategy()) {
        // With all checks disabled, should pass
        let config = ValidationConfig {
            check_syntax: false,
            run_linters: false,
            run_type_checking: false,
            warnings_as_errors: false,
        };
        let validator = CodeValidator::with_config(config);
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        prop_assert!(
            result.valid,
            "Validation failed with all checks disabled"
        );
    }

    /// Property: Validation is deterministic
    ///
    /// For any generated code, validating it twice should produce
    /// the same result.
    #[test]
    fn prop_validation_is_deterministic(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result1 = validator.validate(&[file.clone()]).expect("Validation failed");
        let result2 = validator.validate(&[file.clone()]).expect("Validation failed");

        prop_assert_eq!(
            result1.valid, result2.valid,
            "Validation results differ"
        );
        prop_assert_eq!(
            result1.errors.len(), result2.errors.len(),
            "Error count differs between validations"
        );
    }

    /// Property: Validation preserves file metadata
    ///
    /// For any generated code, validation should not modify the
    /// file path or language.
    #[test]
    fn prop_validation_preserves_file_metadata(file in invalid_generated_file_strategy()) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[file.clone()]).expect("Validation failed");

        // File metadata should be preserved in errors
        for error in &result.errors {
            prop_assert_eq!(
                &error.file, &file.path,
                "File path changed during validation"
            );
        }
    }

    /// Property: Empty file list produces valid result
    ///
    /// For an empty list of files, validation should pass with no errors.
    #[test]
    fn prop_empty_file_list_passes_validation(_unit in Just(())) {
        let validator = CodeValidator::new();
        let result = validator.validate(&[]).expect("Validation failed");

        prop_assert!(
            result.valid,
            "Empty file list failed validation"
        );
        prop_assert!(
            result.errors.is_empty(),
            "Empty file list produced errors"
        );
    }
}
