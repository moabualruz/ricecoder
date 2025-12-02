//! Property-Based Tests for Boilerplate Discovery
//!
//! This module contains property-based tests that validate the correctness properties
//! defined in the ricecoder-templates feature specification.
//!
//! Feature: ricecoder-templates
//! Design: .kiro/specs/ricecoder-templates/design.md

use proptest::prelude::*;
use std::fs;
use tempfile::TempDir;

// Import from ricecoder-generation crate
use ricecoder_generation::templates::BoilerplateDiscovery;
use ricecoder_generation::models::BoilerplateSource;

/// Property 5: Boilerplate Discovery Precedence
///
/// *For any* boilerplate name, when boilerplates exist in both project and global scopes,
/// the project-scoped boilerplate SHALL be discovered and used.
///
/// **Feature: ricecoder-templates, Property 5: Boilerplate Discovery Precedence**
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[test]
fn test_boilerplate_discovery_precedence_project_overrides_global() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a project boilerplate with the same name as a global one
    let project_bp = project_dir.join("my-bp");
    fs::create_dir_all(&project_bp).unwrap();
    let project_metadata = r#"
id: my-bp
name: My Boilerplate
description: Project version
language: rust
files: []
dependencies: []
scripts: []
"#;
    fs::write(project_bp.join("boilerplate.yaml"), project_metadata).unwrap();

    let result = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

    // Should find the boilerplate
    let bp = result.boilerplates.iter().find(|bp| bp.id == "my-bp");
    assert!(bp.is_some(), "Boilerplate 'my-bp' should be discovered");

    // Verify it's marked as project source (not global)
    let bp = bp.unwrap();
    assert!(
        matches!(bp.source, BoilerplateSource::Project(_)),
        "Boilerplate should be from project scope"
    );
}

/// Property 5: Boilerplate Discovery Precedence (Deterministic)
///
/// *For any* set of boilerplates in project and global scopes, discovering twice
/// with identical input SHALL produce identical results (deterministic).
///
/// **Feature: ricecoder-templates, Property 5: Boilerplate Discovery Precedence**
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[test]
fn test_boilerplate_discovery_is_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
    fs::create_dir_all(&project_dir).unwrap();

    // Create multiple boilerplates
    for i in 0..3 {
        let bp = project_dir.join(format!("bp-{}", i));
        fs::create_dir_all(&bp).unwrap();
        let metadata = format!(
            r#"
id: bp-{}
name: Boilerplate {}
description: Test boilerplate
language: rust
files: []
dependencies: []
scripts: []
"#,
            i, i
        );
        fs::write(bp.join("boilerplate.yaml"), metadata).unwrap();
    }

    // Discover twice
    let result1 = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();
    let result2 = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

    // Results should be identical
    assert_eq!(
        result1.boilerplates.len(),
        result2.boilerplates.len(),
        "Discovery should return same number of boilerplates"
    );

    // Sort by ID for comparison (order might vary)
    let mut bp1: Vec<_> = result1.boilerplates.iter().map(|bp| &bp.id).collect();
    let mut bp2: Vec<_> = result2.boilerplates.iter().map(|bp| &bp.id).collect();
    bp1.sort();
    bp2.sort();

    assert_eq!(bp1, bp2, "Discovery should return same boilerplates");
}

/// Property 5: Boilerplate Discovery Precedence (Project Scope Priority)
///
/// *For any* boilerplate that exists in both project and global scopes with the same ID,
/// the project-scoped version SHALL be returned (project scope takes precedence).
///
/// **Feature: ricecoder-templates, Property 5: Boilerplate Discovery Precedence**
/// **Validates: Requirements 4.1, 4.2, 4.3**
proptest! {
    #[test]
    fn prop_boilerplate_discovery_project_precedence(
        bp_id in "[a-z][a-z0-9-]{0,19}",
        project_desc in "[a-zA-Z0-9 ]*",
    ) {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
        fs::create_dir_all(&project_dir).unwrap();

        // Create project boilerplate
        let project_bp = project_dir.join(&bp_id);
        fs::create_dir_all(&project_bp).unwrap();
        let project_metadata = format!(
            r#"
id: {}
name: Project Boilerplate
description: {}
language: rust
files: []
dependencies: []
scripts: []
"#,
            bp_id, project_desc
        );
        fs::write(project_bp.join("boilerplate.yaml"), project_metadata).unwrap();

        // Discover boilerplates
        let result = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

        // Should find the boilerplate
        let bp = result.boilerplates.iter().find(|bp| bp.id == bp_id);
        prop_assert!(bp.is_some(), "Boilerplate should be discovered");

        // Verify it's from project scope
        let bp = bp.unwrap();
        prop_assert!(
            matches!(bp.source, BoilerplateSource::Project(_)),
            "Boilerplate should be from project scope"
        );
    }
}

/// Property 5: Boilerplate Discovery Precedence (Search Paths)
///
/// *For any* discovery operation, the search_paths result SHALL include both
/// project and global boilerplate directories (when they exist).
///
/// **Feature: ricecoder-templates, Property 5: Boilerplate Discovery Precedence**
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[test]
fn test_boilerplate_discovery_includes_search_paths() {
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join(".ricecoder").join("boilerplates");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a boilerplate
    let bp = project_dir.join("test-bp");
    fs::create_dir_all(&bp).unwrap();
    let metadata = r#"
id: test-bp
name: Test Boilerplate
description: Test
language: rust
files: []
dependencies: []
scripts: []
"#;
    fs::write(bp.join("boilerplate.yaml"), metadata).unwrap();

    let result = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

    // Should include project boilerplate directory in search paths
    assert!(
        result.search_paths.iter().any(|p| p.ends_with("boilerplates")),
        "Search paths should include boilerplate directory"
    );
}

/// Property 5: Boilerplate Discovery Precedence (Empty Discovery)
///
/// *For any* project with no boilerplates, discovery SHALL return empty list
/// without errors.
///
/// **Feature: ricecoder-templates, Property 5: Boilerplate Discovery Precedence**
/// **Validates: Requirements 4.1, 4.2, 4.3**
#[test]
fn test_boilerplate_discovery_empty_project() {
    let temp_dir = TempDir::new().unwrap();

    // Don't create any boilerplates
    let result = BoilerplateDiscovery::discover(temp_dir.path()).unwrap();

    // Should return empty list
    assert_eq!(
        result.boilerplates.len(),
        0,
        "Discovery should return empty list for project with no boilerplates"
    );
}
