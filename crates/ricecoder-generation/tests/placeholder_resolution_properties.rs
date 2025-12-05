//! Property-based tests for placeholder resolution
//! **Feature: ricecoder-templates, Property 2: Placeholder Resolution Completeness**
//! **Validates: Requirements 1.1, 2.1, 2.2**

use proptest::prelude::*;
use ricecoder_generation::templates::{CaseTransform, PlaceholderResolver};

/// Strategy for generating valid placeholder names
fn placeholder_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]*"
        .prop_map(|s| s.to_string())
        .prop_filter("name must not be empty", |s| !s.is_empty())
}

/// Strategy for generating placeholder values
fn placeholder_value_strategy() -> impl Strategy<Value = String> {
    "[a-z_][a-z0-9_]*"
        .prop_map(|s| s.to_string())
        .prop_filter("value must not be empty", |s| !s.is_empty())
}

proptest! {
    /// Property: For any placeholder name and value, if the value is provided,
    /// resolution should succeed and return the transformed value
    #[test]
    fn prop_placeholder_resolution_succeeds_with_provided_value(
        name in placeholder_name_strategy(),
        value in placeholder_value_strategy(),
    ) {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_value(&name, &value);

        // Should succeed for all case transforms
        let result_pascal = resolver.resolve(&name, CaseTransform::PascalCase);
        let result_camel = resolver.resolve(&name, CaseTransform::CamelCase);
        let result_snake = resolver.resolve(&name, CaseTransform::SnakeCase);
        let result_kebab = resolver.resolve(&name, CaseTransform::KebabCase);
        let result_upper = resolver.resolve(&name, CaseTransform::UpperCase);
        let result_lower = resolver.resolve(&name, CaseTransform::LowerCase);

        prop_assert!(result_pascal.is_ok());
        prop_assert!(result_camel.is_ok());
        prop_assert!(result_snake.is_ok());
        prop_assert!(result_kebab.is_ok());
        prop_assert!(result_upper.is_ok());
        prop_assert!(result_lower.is_ok());
    }

    /// Property: For any placeholder name, if the value is NOT provided,
    /// resolution should fail with MissingPlaceholder error
    #[test]
    fn prop_placeholder_resolution_fails_without_value(
        name in placeholder_name_strategy(),
    ) {
        let resolver = PlaceholderResolver::new();

        // Should fail for all case transforms
        let result_pascal = resolver.resolve(&name, CaseTransform::PascalCase);
        let result_camel = resolver.resolve(&name, CaseTransform::CamelCase);
        let result_snake = resolver.resolve(&name, CaseTransform::SnakeCase);
        let result_kebab = resolver.resolve(&name, CaseTransform::KebabCase);
        let result_upper = resolver.resolve(&name, CaseTransform::UpperCase);
        let result_lower = resolver.resolve(&name, CaseTransform::LowerCase);

        prop_assert!(result_pascal.is_err());
        prop_assert!(result_camel.is_err());
        prop_assert!(result_snake.is_err());
        prop_assert!(result_kebab.is_err());
        prop_assert!(result_upper.is_err());
        prop_assert!(result_lower.is_err());
    }

    /// Property: For any set of required placeholders, validation should fail
    /// if any required placeholder is missing
    #[test]
    fn prop_validation_fails_with_missing_required(
        required_names in prop::collection::vec(placeholder_name_strategy(), 1..5),
        provided_names in prop::collection::vec(placeholder_name_strategy(), 0..3),
    ) {
        let mut resolver = PlaceholderResolver::new();

        // Add provided values
        for name in &provided_names {
            resolver.add_value(name, "value");
        }

        // Mark all required names
        for name in &required_names {
            resolver.require(name);
        }

        // Check if any required name is missing
        let any_missing = required_names.iter().any(|name| !provided_names.contains(name));

        let validation_result = resolver.validate();
        if any_missing {
            prop_assert!(validation_result.is_err());
        } else {
            prop_assert!(validation_result.is_ok());
        }
    }

    /// Property: For any placeholder with a default value, resolution should succeed
    /// even if the value is not explicitly provided
    #[test]
    fn prop_placeholder_with_default_resolves(
        name in placeholder_name_strategy(),
        default_value in placeholder_value_strategy(),
    ) {
        let resolver = PlaceholderResolver::new();

        // Resolve with default value
        let result = resolver.resolve_with_default(&name, CaseTransform::PascalCase, Some(&default_value));
        prop_assert!(result.is_ok());
        // The result may be empty if the default value is just underscores or similar
        // but the resolution should still succeed
    }

    /// Property: For any set of placeholder values, the resolver should report
    /// all provided names correctly
    #[test]
    fn prop_provided_names_matches_added_values(
        values in prop::collection::hash_map(placeholder_name_strategy(), placeholder_value_strategy(), 1..10),
    ) {
        let mut resolver = PlaceholderResolver::new();
        resolver.add_values(values.clone());

        let provided = resolver.provided_names();
        let provided_set: std::collections::HashSet<_> = provided.into_iter().collect();
        let expected_set: std::collections::HashSet<_> = values.keys().cloned().collect();

        prop_assert_eq!(provided_set, expected_set);
    }

    /// Property: For any placeholder name, has_value should return true
    /// if and only if the value was added
    #[test]
    fn prop_has_value_consistency(
        name in placeholder_name_strategy(),
        value in placeholder_value_strategy(),
    ) {
        let mut resolver = PlaceholderResolver::new();

        // Before adding
        prop_assert!(!resolver.has_value(&name));

        // After adding
        resolver.add_value(&name, &value);
        prop_assert!(resolver.has_value(&name));
    }
}
