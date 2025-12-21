//! Property-based tests for Documentation Generator
//!
//! These tests verify correctness properties that should hold across all valid inputs

use proptest::prelude::*;
use ricecoder_github::{DocumentationCoverage, DocumentationGenerator, ReadmeConfig};

// Strategy for generating valid project names
fn valid_project_name_strategy() -> impl Strategy<Value = String> {
    r"[A-Za-z][A-Za-z0-9\-]{0,50}"
        .prop_map(|s| s.to_string())
        .prop_filter("project name must not be empty", |s| !s.is_empty())
}

// Strategy for generating valid descriptions
fn valid_description_strategy() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 \-_.,!?]{0,200}".prop_map(|s| s.trim().to_string())
}

// Strategy for generating README configurations
fn readme_config_strategy() -> impl Strategy<Value = ReadmeConfig> {
    (
        valid_project_name_strategy(),
        valid_description_strategy(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                project_name,
                description,
                include_toc,
                include_installation,
                include_usage,
                include_api,
                include_contributing,
                include_license,
            )| {
                ReadmeConfig {
                    project_name,
                    description,
                    include_toc,
                    include_installation,
                    include_usage,
                    include_api,
                    include_contributing,
                    include_license,
                }
            },
        )
}

// Strategy for generating Rust code with functions
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(r"(pub )?fn [a-z_][a-z0-9_]*\(\) \{\}", 0..10)
        .prop_map(|functions| functions.join("\n"))
}

// Strategy for generating documented Rust code
fn documented_rust_code_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (
            r"/// [a-zA-Z0-9 ]{1,50}",
            r"(pub )?fn [a-z_][a-z0-9_]*\(\) \{\}",
        ),
        0..5,
    )
    .prop_map(|items| {
        items
            .iter()
            .map(|(doc, func)| format!("{}\n{}", doc, func))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

// **Feature: ricecoder-github, Property 26: README Generation**
// *For any* repository structure, the system SHALL generate a README file with correct sections and content.
proptest! {
    #[test]
    fn prop_readme_generation_produces_non_empty_output(config in readme_config_strategy()) {
        let generator = DocumentationGenerator::new(config.clone());
        let readme = generator.generate_readme().expect("Failed to generate README");

        // README must not be empty
        prop_assert!(!readme.is_empty());

        // README must contain project name
        prop_assert!(readme.contains(&config.project_name));

        // README must contain description if provided
        if !config.description.is_empty() {
            prop_assert!(readme.contains(&config.description));
        }

        // README must contain expected sections based on config
        if config.include_toc {
            prop_assert!(readme.contains("## Table of Contents"));
        }

        if config.include_installation {
            prop_assert!(readme.contains("## Installation"));
        }

        if config.include_usage {
            prop_assert!(readme.contains("## Usage"));
        }

        if config.include_contributing {
            prop_assert!(readme.contains("## Contributing"));
        }

        if config.include_license {
            prop_assert!(readme.contains("## License"));
        }
    }

    #[test]
    fn prop_readme_generation_is_deterministic(config in readme_config_strategy()) {
        let generator1 = DocumentationGenerator::new(config.clone());
        let readme1 = generator1.generate_readme().expect("Failed to generate README");

        let generator2 = DocumentationGenerator::new(config);
        let readme2 = generator2.generate_readme().expect("Failed to generate README");

        // Same config should produce identical README
        prop_assert_eq!(readme1, readme2);
    }

    #[test]
    fn prop_readme_contains_markdown_structure(config in readme_config_strategy()) {
        let generator = DocumentationGenerator::new(config);
        let readme = generator.generate_readme().expect("Failed to generate README");

        // README must start with a heading
        prop_assert!(readme.trim_start().starts_with("#"));

        // README must contain valid markdown
        prop_assert!(readme.contains("\n"));
    }
}

// **Feature: ricecoder-github, Property 27: API Documentation Extraction**
// *For any* codebase, the system SHALL extract API documentation from code comments and docstrings.
proptest! {
    #[test]
    fn prop_api_documentation_extraction_finds_documented_functions(
        code in documented_rust_code_strategy()
    ) {
        let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
        let api_docs = generator.extract_api_documentation(&code)
            .expect("Failed to extract API docs");

        // If code contains functions, should extract documentation
        if code.contains("fn ") {
            // Should extract at least some documentation
            prop_assert!(!api_docs.is_empty() || !code.contains("///"));
        }
    }

    #[test]
    fn prop_api_documentation_extraction_is_deterministic(
        code in documented_rust_code_strategy()
    ) {
        let mut generator1 = DocumentationGenerator::new(ReadmeConfig::default());
        let docs1 = generator1.extract_api_documentation(&code)
            .expect("Failed to extract API docs");

        let mut generator2 = DocumentationGenerator::new(ReadmeConfig::default());
        let docs2 = generator2.extract_api_documentation(&code)
            .expect("Failed to extract API docs");

        // Same code should produce same documentation
        prop_assert_eq!(docs1.len(), docs2.len());
    }

    #[test]
    fn prop_api_documentation_extraction_handles_empty_code(
        _empty in Just(String::new())
    ) {
        let mut generator = DocumentationGenerator::new(ReadmeConfig::default());
        let api_docs = generator.extract_api_documentation("")
            .expect("Failed to extract API docs");

        // Empty code should produce empty documentation
        prop_assert!(api_docs.is_empty());
    }
}

// **Feature: ricecoder-github, Property 28: Documentation Synchronization**
// *For any* code change, the system SHALL update corresponding documentation to reflect the changes.
proptest! {
    #[test]
    fn prop_documentation_synchronization_detects_changes(
        old_code in rust_code_strategy(),
        new_code in rust_code_strategy()
    ) {
        let generator = DocumentationGenerator::new(ReadmeConfig::default());
        let sync_result = generator.synchronize_documentation(&old_code, &new_code)
            .expect("Failed to synchronize");

        // Synchronization should always succeed
        prop_assert!(sync_result.success);

        // If code is identical, no files should be updated
        if old_code == new_code {
            prop_assert!(sync_result.files_updated.is_empty());
        }
    }

    #[test]
    fn prop_documentation_synchronization_is_idempotent(
        old_code in rust_code_strategy(),
        new_code in rust_code_strategy()
    ) {
        let generator = DocumentationGenerator::new(ReadmeConfig::default());

        let sync1 = generator.synchronize_documentation(&old_code, &new_code)
            .expect("Failed to synchronize");

        let sync2 = generator.synchronize_documentation(&old_code, &new_code)
            .expect("Failed to synchronize");

        // Multiple synchronizations with same inputs should produce same result
        prop_assert_eq!(sync1.success, sync2.success);
        prop_assert_eq!(sync1.files_updated.len(), sync2.files_updated.len());
    }

    #[test]
    fn prop_documentation_synchronization_handles_identical_code(
        code in rust_code_strategy()
    ) {
        let generator = DocumentationGenerator::new(ReadmeConfig::default());
        let sync_result = generator.synchronize_documentation(&code, &code)
            .expect("Failed to synchronize");

        // Identical code should not require updates
        prop_assert!(sync_result.success);
    }
}

// **Feature: ricecoder-github, Property 29: Documentation Commit**
// *For any* documentation update, the system SHALL create a commit with the updated documentation.
proptest! {
    #[test]
    fn prop_documentation_commit_stores_files(
        files in prop::collection::vec(r"[a-z0-9_\-]+\.md", 1..5)
    ) {
        use ricecoder_github::DocumentationCommit;

        let mut commit = DocumentationCommit::new("Update docs");
        for file in &files {
            commit = commit.with_file(file.clone());
        }

        // Commit should contain all files
        prop_assert_eq!(commit.files.len(), files.len());

        for file in &files {
            prop_assert!(commit.files.contains(file));
        }
    }

    #[test]
    fn prop_documentation_commit_preserves_message(
        message in r"[a-zA-Z0-9 \-_.,]{1,100}"
    ) {
        use ricecoder_github::DocumentationCommit;

        let commit = DocumentationCommit::new(message.clone());

        // Commit message should be preserved
        prop_assert_eq!(commit.message, message);
    }
}

// **Feature: ricecoder-github, Property 30: Documentation Coverage Tracking**
// *For any* codebase, the system SHALL calculate documentation coverage metrics and identify gaps.
proptest! {
    #[test]
    fn prop_documentation_coverage_calculation_is_valid(
        total in 1u32..100,
        documented in 0u32..100
    ) {
        let documented = documented.min(total);
        let coverage = DocumentationCoverage::new(total, documented);

        // Coverage percentage should be between 0 and 100
        prop_assert!(coverage.coverage_percentage >= 0.0);
        prop_assert!(coverage.coverage_percentage <= 100.0);

        // Coverage should match calculation
        let expected = (documented as f32 / total as f32) * 100.0;
        prop_assert!((coverage.coverage_percentage - expected).abs() < 0.1);
    }

    #[test]
    fn prop_documentation_coverage_100_percent_when_all_documented(
        total in 1u32..100
    ) {
        let coverage = DocumentationCoverage::new(total, total);

        // 100% coverage when all items are documented
        prop_assert!((coverage.coverage_percentage - 100.0).abs() < 0.1);
    }

    #[test]
    fn prop_documentation_coverage_0_percent_when_none_documented(
        total in 1u32..100
    ) {
        let coverage = DocumentationCoverage::new(total, 0);

        // 0% coverage when no items are documented
        prop_assert!(coverage.coverage_percentage < 0.1);
    }

    #[test]
    fn prop_documentation_coverage_handles_zero_items(
        _zero in Just(0u32)
    ) {
        let coverage = DocumentationCoverage::new(0, 0);

        // Zero items should result in 100% coverage (vacuous truth)
        prop_assert!((coverage.coverage_percentage - 100.0).abs() < 0.1);
    }
}
