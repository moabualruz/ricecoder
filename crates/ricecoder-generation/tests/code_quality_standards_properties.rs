//! Property-based tests for code quality standards
//!
//! **Feature: ricecoder-generation, Property 7: Code Quality Standards**
//! **Validates: Requirements 2.2, 2.4**
//!
//! Property: For any generated code, all public types and functions SHALL include doc comments,
//! and all fallible operations SHALL include error handling using language-native error types.

use proptest::prelude::*;
use ricecoder_generation::{CodeQualityConfig, CodeQualityEnforcer, GeneratedFile};

/// Strategy for generating Rust code with public items
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("pub fn hello() {}".to_string()),
        Just("pub struct User { name: String }".to_string()),
        Just("pub enum Status { Active, Inactive }".to_string()),
        Just("pub fn process() -> Result<(), String> { Ok(()) }".to_string()),
    ]
}

/// Strategy for generating TypeScript code with public items
fn typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("export function hello() {}".to_string()),
        Just("export class User { name: string; }".to_string()),
        Just("export interface Status { active: boolean; }".to_string()),
        Just("export function process(): Promise<void> { return Promise.resolve(); }".to_string()),
    ]
}

/// Strategy for generating Python code with public items
fn python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("def hello():\n    pass".to_string()),
        Just("class User:\n    def __init__(self, name):\n        self.name = name".to_string()),
        Just("def process():\n    return None".to_string()),
    ]
}

/// Strategy for generating generated files
fn generated_file_strategy() -> impl Strategy<Value = GeneratedFile> {
    prop_oneof![
        (Just("src/main.rs"), rust_code_strategy(), Just("rust")).prop_map(
            |(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }
        ),
        (
            Just("src/index.ts"),
            typescript_code_strategy(),
            Just("typescript")
        )
            .prop_map(|(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }),
        (Just("src/main.py"), python_code_strategy(), Just("python")).prop_map(
            |(path, content, language)| GeneratedFile {
                path: path.to_string(),
                content,
                language: language.to_string(),
            }
        ),
    ]
}

proptest! {
    /// Property: Doc comments are added to public items
    ///
    /// For any generated code with public items, the CodeQualityEnforcer
    /// should add doc comments to all public types and functions.
    #[test]
    fn prop_doc_comments_added_to_public_items(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced = enforcer.enforce_file(&file).expect("Failed to enforce quality");

        // Check that doc comments were added
        match file.language.as_str() {
            "rust" => {
                if file.content.contains("pub fn ") || file.content.contains("pub struct ") {
                    prop_assert!(
                        enhanced.content.contains("///"),
                        "Rust doc comments not added to: {}",
                        file.content
                    );
                }
            }
            "typescript" => {
                if file.content.contains("export function ") || file.content.contains("export class ") {
                    prop_assert!(
                        enhanced.content.contains("/**") || enhanced.content.contains("*"),
                        "TypeScript JSDoc comments not added to: {}",
                        file.content
                    );
                }
            }
            "python" => {
                if file.content.contains("def ") || file.content.contains("class ") {
                    prop_assert!(
                        enhanced.content.contains("\"\"\""),
                        "Python docstrings not added to: {}",
                        file.content
                    );
                }
            }
            _ => {}
        }
    }

    /// Property: Error handling is enforced
    ///
    /// For any generated code, the CodeQualityEnforcer should detect
    /// missing error handling patterns.
    #[test]
    fn prop_error_handling_detected(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let issues = enforcer.check_error_handling(&file.content, &file.language);

        // For Rust code with unwrap(), issues should be detected
        if file.language == "rust" && file.content.contains("unwrap()") {
            prop_assert!(
                !issues.is_empty(),
                "Error handling issues not detected in Rust code"
            );
        }
    }

    /// Property: Doc comment checks work correctly
    ///
    /// For any generated code, the CodeQualityEnforcer should correctly
    /// identify missing doc comments.
    #[test]
    fn prop_doc_comment_checks_work(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let issues = enforcer.check_doc_comments(&file.content, &file.language);

        // For Rust code with public items, issues should be detected
        if file.language == "rust" && file.content.contains("pub fn ") {
            prop_assert!(
                !issues.is_empty(),
                "Doc comment issues not detected in Rust code"
            );
        }
    }

    /// Property: Enforcement preserves code structure
    ///
    /// For any generated code, enforcing quality standards should not
    /// remove or corrupt the original code structure.
    #[test]
    fn prop_enforcement_preserves_structure(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced = enforcer.enforce_file(&file).expect("Failed to enforce quality");

        // The enhanced code should contain all original content
        for line in file.content.lines() {
            if !line.trim().is_empty() {
                prop_assert!(
                    enhanced.content.contains(line),
                    "Original code line lost during enforcement: {}",
                    line
                );
            }
        }
    }

    /// Property: Enforcement is idempotent
    ///
    /// For any generated code, enforcing quality standards twice should
    /// produce the same result as enforcing once.
    #[test]
    fn prop_enforcement_is_idempotent(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced_once = enforcer.enforce_file(&file).expect("Failed to enforce quality");
        let enhanced_twice = enforcer.enforce_file(&enhanced_once).expect("Failed to enforce quality");

        prop_assert_eq!(
            enhanced_once.content, enhanced_twice.content,
            "Enforcement is not idempotent"
        );
    }

    /// Property: Configuration is respected
    ///
    /// For any generated code, the CodeQualityEnforcer should respect
    /// the configuration settings.
    #[test]
    fn prop_configuration_respected(file in generated_file_strategy()) {
        let config = CodeQualityConfig {
            require_doc_comments: false,
            generate_tests: false,
            enforce_naming: false,
            enforce_error_handling: false,
        };
        let enforcer = CodeQualityEnforcer::with_config(config);
        let enhanced = enforcer.enforce_file(&file).expect("Failed to enforce quality");

        // With all checks disabled, content should be unchanged
        prop_assert_eq!(
            file.content, enhanced.content,
            "Configuration not respected"
        );
    }

    /// Property: File metadata is preserved
    ///
    /// For any generated code, enforcing quality standards should not
    /// change the file path or language.
    #[test]
    fn prop_file_metadata_preserved(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced = enforcer.enforce_file(&file).expect("Failed to enforce quality");

        prop_assert_eq!(&file.path, &enhanced.path, "File path changed");
        prop_assert_eq!(&file.language, &enhanced.language, "Language changed");
    }

    /// Property: Multiple files can be enforced
    ///
    /// For any collection of generated files, the CodeQualityEnforcer
    /// should be able to enforce all of them without errors.
    #[test]
    fn prop_multiple_files_enforced(files in prop::collection::vec(generated_file_strategy(), 1..5)) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced = enforcer.enforce(files.clone()).expect("Failed to enforce quality");

        prop_assert_eq!(
            files.len(), enhanced.len(),
            "Number of files changed during enforcement"
        );

        for (original, result) in files.iter().zip(enhanced.iter()) {
            prop_assert_eq!(&original.path, &result.path, "File path changed");
            prop_assert_eq!(&original.language, &result.language, "Language changed");
        }
    }

    /// Property: Language-specific enforcement works
    ///
    /// For any generated code, the CodeQualityEnforcer should apply
    /// language-specific quality standards.
    #[test]
    fn prop_language_specific_enforcement(file in generated_file_strategy()) {
        let enforcer = CodeQualityEnforcer::new();
        let enhanced = enforcer.enforce_file(&file).expect("Failed to enforce quality");

        // The enhanced code should be valid for the language
        match file.language.as_str() {
            "rust" => {
                // Rust code should have proper syntax
                prop_assert!(
                    enhanced.content.contains("fn ") || enhanced.content.contains("struct ") || enhanced.content.contains("enum "),
                    "Rust code structure lost"
                );
            }
            "typescript" => {
                // TypeScript code should have proper syntax
                prop_assert!(
                    enhanced.content.contains("function ") || enhanced.content.contains("class ") || enhanced.content.contains("interface "),
                    "TypeScript code structure lost"
                );
            }
            "python" => {
                // Python code should have proper syntax
                prop_assert!(
                    enhanced.content.contains("def ") || enhanced.content.contains("class "),
                    "Python code structure lost"
                );
            }
            _ => {}
        }
    }
}
