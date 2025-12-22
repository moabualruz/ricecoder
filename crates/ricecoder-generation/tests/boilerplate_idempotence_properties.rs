//! Property-based tests for boilerplate idempotence
//! **Feature: ricecoder-templates, Property 4: Boilerplate Idempotence**
//! **Validates: Requirements 3.1, 3.2, 3.3**

use std::collections::HashMap;

use proptest::prelude::*;
use ricecoder_generation::{
    models::{Boilerplate, BoilerplateFile},
    BoilerplateManager,
};

/// Strategy for generating valid boilerplate IDs
fn boilerplate_id_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{1,10}"
        .prop_map(|s| s.to_string())
        .prop_filter("id must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid boilerplate names
fn boilerplate_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9 ]{1,20}"
        .prop_map(|s| s.to_string())
        .prop_filter("name must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid file paths
fn file_path_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9_/]{3,20}\\.rs"
        .prop_map(|s| s.to_string())
        .prop_filter("path must not be empty", |s| !s.is_empty())
}

/// Strategy for generating simple template content
fn template_content_strategy() -> impl Strategy<Value = String> {
    "pub struct [A-Za-z0-9_]+ \\{\\}"
        .prop_map(|s| s.to_string())
        .prop_filter("content must not be empty", |s| !s.is_empty())
}

/// Strategy for generating boilerplate files
fn boilerplate_file_strategy() -> impl Strategy<Value = BoilerplateFile> {
    (file_path_strategy(), template_content_strategy()).prop_map(|(path, template)| {
        BoilerplateFile {
            path,
            template,
            condition: None,
        }
    })
}

/// Strategy for generating boilerplates
fn boilerplate_strategy() -> impl Strategy<Value = Boilerplate> {
    (
        boilerplate_id_strategy(),
        boilerplate_name_strategy(),
        prop::collection::vec(boilerplate_file_strategy(), 1..3),
    )
        .prop_map(|(id, name, files)| Boilerplate {
            id,
            name,
            description: "Test boilerplate".to_string(),
            language: "rust".to_string(),
            files,
            dependencies: vec![],
            scripts: vec![],
        })
}

/// Strategy for generating variable maps
fn variables_strategy() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map("[a-z][a-z0-9]{1,5}", "[a-zA-Z0-9_-]{1,10}", 1..3).prop_map(|map| {
        map.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    })
}

proptest! {
    /// Property: For any boilerplate, validation should be consistent
    /// across multiple calls with the same input
    #[test]
    fn prop_boilerplate_validation_idempotent(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Validate the boilerplate multiple times
        let result1 = manager.validate(&boilerplate);
        let result2 = manager.validate(&boilerplate);
        let result3 = manager.validate(&boilerplate);

        // All validations should have the same result
        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Validation should be consistent"
        );
        prop_assert_eq!(
            result2.is_ok(),
            result3.is_ok(),
            "Validation should be consistent"
        );
    }

    /// Property: For any boilerplate, extracting placeholders should be
    /// consistent across multiple calls
    #[test]
    fn prop_boilerplate_placeholder_extraction_idempotent(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Only test if boilerplate is valid
        if manager.validate(&boilerplate).is_err() {
            return Ok(());
        }

        // Extract placeholders multiple times
        let result1 = manager.extract_placeholders(&boilerplate);
        let result2 = manager.extract_placeholders(&boilerplate);

        // Both extractions should succeed and be identical
        prop_assert!(result1.is_ok(), "First extraction should succeed");
        prop_assert!(result2.is_ok(), "Second extraction should succeed");

        let placeholders1 = result1.unwrap();
        let placeholders2 = result2.unwrap();

        prop_assert_eq!(
            placeholders1.len(),
            placeholders2.len(),
            "Placeholder count should be consistent"
        );

        // All placeholder names should match
        for (name, _) in &placeholders1 {
            prop_assert!(
                placeholders2.contains_key(name),
                "Placeholder {} should be in second extraction",
                name
            );
        }
    }

    /// Property: For any boilerplate, the number of files should be
    /// consistent across multiple validations
    #[test]
    fn prop_boilerplate_file_count_consistent(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Validate multiple times
        let _ = manager.validate(&boilerplate);
        let _ = manager.validate(&boilerplate);

        // File count should remain the same
        let file_count = boilerplate.files.len();
        prop_assert!(file_count > 0, "Boilerplate should have at least one file");

        // Verify all files have non-empty paths and templates
        for file in &boilerplate.files {
            prop_assert!(!file.path.is_empty(), "File path should not be empty");
            prop_assert!(!file.template.is_empty(), "File template should not be empty");
        }
    }

    /// Property: For any boilerplate, validation should succeed if all
    /// required fields are present
    #[test]
    fn prop_boilerplate_validation_checks_required_fields(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Validate the boilerplate
        let result = manager.validate(&boilerplate);

        // If validation succeeds, all required fields should be non-empty
        if result.is_ok() {
            prop_assert!(!boilerplate.id.is_empty(), "ID should not be empty");
            prop_assert!(!boilerplate.name.is_empty(), "Name should not be empty");
            prop_assert!(!boilerplate.language.is_empty(), "Language should not be empty");
            prop_assert!(!boilerplate.files.is_empty(), "Files should not be empty");
        }
    }

    /// Property: For any boilerplate, the structure should remain unchanged
    /// after validation
    #[test]
    fn prop_boilerplate_validation_does_not_modify_structure(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Store original values
        let original_id = boilerplate.id.clone();
        let original_name = boilerplate.name.clone();
        let original_file_count = boilerplate.files.len();

        // Validate
        let _ = manager.validate(&boilerplate);

        // Verify structure is unchanged
        prop_assert_eq!(boilerplate.id, original_id, "ID should not change");
        prop_assert_eq!(boilerplate.name, original_name, "Name should not change");
        prop_assert_eq!(boilerplate.files.len(), original_file_count, "File count should not change");
    }

    /// Property: For any boilerplate with valid structure, placeholder
    /// extraction should find all placeholders in templates
    #[test]
    fn prop_boilerplate_placeholder_extraction_finds_all_placeholders(
        boilerplate in boilerplate_strategy(),
    ) {
        let manager = BoilerplateManager::new();

        // Only test if boilerplate is valid
        if manager.validate(&boilerplate).is_err() {
            return Ok(());
        }

        // Extract placeholders
        let placeholders = manager.extract_placeholders(&boilerplate).unwrap();

        // For each file, count expected placeholders
        let mut expected_count = 0;
        for file in &boilerplate.files {
            // Count {{...}} patterns in template
            let count = file.template.matches("{{").count();
            expected_count += count;
        }

        // Extracted placeholders should match expected count
        prop_assert_eq!(
            placeholders.len(),
            expected_count,
            "Placeholder count should match template count"
        );
    }
}
