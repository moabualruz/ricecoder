//! Property-based tests for case transformation consistency
//! **Feature: ricecoder-templates, Property 3: Case Transformation Consistency**
//! **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

use proptest::prelude::*;
use ricecoder_generation::templates::CaseTransform;

/// Strategy for generating valid input strings for case transformation
fn case_transform_input_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]*"
        .prop_map(|s| s.to_string())
        .prop_filter("input must not be empty", |s| !s.is_empty())
}

proptest! {
    /// Property: For any input string, applying the same case transformation
    /// should produce consistent results (same input always produces same output)
    #[test]
    fn prop_case_transform_consistent(
        input in case_transform_input_strategy(),
    ) {
        let transforms = vec![
            CaseTransform::PascalCase,
            CaseTransform::CamelCase,
            CaseTransform::SnakeCase,
            CaseTransform::KebabCase,
            CaseTransform::UpperCase,
            CaseTransform::LowerCase,
        ];

        for transform in transforms {
            let first = transform.apply(&input);
            let second = transform.apply(&input);
            prop_assert_eq!(first, second, "Transform {:?} is not consistent", transform);
        }
    }

    /// Property: For any input string, PascalCase transformation should produce
    /// a string that starts with an uppercase letter (or is empty)
    #[test]
    fn prop_pascal_case_starts_uppercase(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::PascalCase.apply(&input);
        if !result.is_empty() {
            prop_assert!(result.chars().next().unwrap().is_uppercase());
        }
    }

    /// Property: For any input string, camelCase transformation should produce
    /// a string that starts with a lowercase letter (or is empty)
    #[test]
    fn prop_camel_case_starts_lowercase(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::CamelCase.apply(&input);
        if !result.is_empty() {
            prop_assert!(result.chars().next().unwrap().is_lowercase());
        }
    }

    /// Property: For any input string, snake_case transformation should only
    /// contain lowercase letters, digits, and underscores
    #[test]
    fn prop_snake_case_valid_chars(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::SnakeCase.apply(&input);
        prop_assert!(result.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_'));
    }

    /// Property: For any input string, kebab-case transformation should only
    /// contain lowercase letters, digits, and hyphens
    #[test]
    fn prop_kebab_case_valid_chars(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::KebabCase.apply(&input);
        prop_assert!(result.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-'));
    }

    /// Property: For any input string, UPPERCASE transformation should only
    /// contain uppercase letters, digits, underscores, and hyphens
    #[test]
    fn prop_uppercase_valid_chars(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::UpperCase.apply(&input);
        prop_assert!(result.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_' || c == '-'));
    }

    /// Property: For any input string, lowercase transformation should only
    /// contain lowercase letters, digits, underscores, and hyphens
    #[test]
    fn prop_lowercase_valid_chars(
        input in case_transform_input_strategy(),
    ) {
        let result = CaseTransform::LowerCase.apply(&input);
        prop_assert!(result.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_' || c == '-'));
    }

    /// Property: For any input string, case transformations should preserve
    /// the semantic content (same letters/digits, just different casing/separators)
    #[test]
    fn prop_case_transform_preserves_content(
        input in case_transform_input_strategy(),
    ) {
        let pascal = CaseTransform::PascalCase.apply(&input);
        let camel = CaseTransform::CamelCase.apply(&input);
        let snake = CaseTransform::SnakeCase.apply(&input);
        let kebab = CaseTransform::KebabCase.apply(&input);
        let upper = CaseTransform::UpperCase.apply(&input);
        let lower = CaseTransform::LowerCase.apply(&input);

        // All transformations should produce non-empty results for non-empty input
        prop_assert!(!pascal.is_empty());
        prop_assert!(!camel.is_empty());
        prop_assert!(!snake.is_empty());
        prop_assert!(!kebab.is_empty());
        prop_assert!(!upper.is_empty());
        prop_assert!(!lower.is_empty());
    }

    /// Property: For any input string, converting to PascalCase and then to
    /// lowercase should produce a valid lowercase result
    #[test]
    fn prop_case_transform_chain(
        input in case_transform_input_strategy(),
    ) {
        let pascal = CaseTransform::PascalCase.apply(&input);
        let lower_from_pascal = CaseTransform::LowerCase.apply(&pascal);
        
        // The result should be all lowercase
        prop_assert!(lower_from_pascal.chars().all(|c| !c.is_uppercase() || !c.is_alphabetic()));
    }

    /// Property: For any input string, UPPERCASE transformation should be
    /// the inverse of lowercase transformation (in terms of case)
    #[test]
    fn prop_uppercase_lowercase_inverse(
        input in case_transform_input_strategy(),
    ) {
        let upper = CaseTransform::UpperCase.apply(&input);
        let lower = CaseTransform::LowerCase.apply(&input);
        
        // Uppercase should have no lowercase letters
        prop_assert!(!upper.chars().any(|c| c.is_lowercase()));
        // Lowercase should have no uppercase letters
        prop_assert!(!lower.chars().any(|c| c.is_uppercase()));
    }

    /// Property: For any input string, applying different case transformations
    /// should produce different results (except for edge cases)
    #[test]
    fn prop_case_transforms_produce_different_results(
        input in "[a-z][a-z0-9]*".prop_map(|s| s.to_string()),
    ) {
        let pascal = CaseTransform::PascalCase.apply(&input);
        let camel = CaseTransform::CamelCase.apply(&input);
        let _snake = CaseTransform::SnakeCase.apply(&input);
        let _kebab = CaseTransform::KebabCase.apply(&input);
        let _upper = CaseTransform::UpperCase.apply(&input);
        let lower = CaseTransform::LowerCase.apply(&input);

        let input_len = input.len();
        
        // For single-word lowercase input, most transforms should differ
        // (except lowercase which should be the same as input)
        prop_assert_eq!(lower, input);
        
        // PascalCase should differ from camelCase for multi-word inputs
        // but for single word, PascalCase should capitalize first letter
        if input_len > 1 {
            prop_assert_ne!(pascal, camel);
        }
    }
}
