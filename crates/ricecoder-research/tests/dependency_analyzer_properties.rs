//! Property-based tests for DependencyAnalyzer
//!
//! **Feature: ricecoder-research, Property 7: Multi-Language Dependency Parsing**
//! **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 3.12**

use proptest::prelude::*;
use ricecoder_research::DependencyAnalyzer;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Property 7: Multi-Language Dependency Parsing
// ============================================================================
// For any supported project type (Rust, Node.js, Python, Go, Java, Kotlin, .NET, PHP, Ruby, Swift, Dart/Flutter),
// dependency parsing SHALL extract all declared dependencies with version information and constraints.

proptest! {
    /// Property: Rust dependencies are correctly parsed from Cargo.toml
    #[test]
    fn prop_rust_dependencies_parsed(
        names in prop::collection::hash_set(r"[a-z][a-z0-9_]*", 1..5),
        versions in prop::collection::vec(r"[0-9]+\.[0-9]+\.[0-9]+", 1..5)
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build Cargo.toml content with unique names
        let mut cargo_toml = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        let names_vec: Vec<_> = names.into_iter().collect();
        for (i, name) in names_vec.iter().enumerate() {
            let version = &versions[i % versions.len()];
            cargo_toml.push_str(&format!("{} = \"{}\"\n", name, version));
        }

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // All declared dependencies should be parsed
        prop_assert_eq!(parsed.len(), names_vec.len());

        // Each parsed dependency should be found
        for name in &names_vec {
            let found = parsed.iter().find(|d| &d.name == name);
            prop_assert!(found.is_some(), "Dependency {} not found", name);
        }
    }

    /// Property: Node.js dependencies are correctly parsed from package.json
    #[test]
    fn prop_nodejs_dependencies_parsed(
        names in prop::collection::hash_set(r"[a-z][a-z0-9\-]*", 1..5),
        versions in prop::collection::vec(r"[0-9]+\.[0-9]+\.[0-9]+", 1..5)
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build package.json content with unique names
        let mut json_deps = String::from("{\n  \"name\": \"test\",\n  \"dependencies\": {\n");
        let names_vec: Vec<_> = names.into_iter().collect();
        for (i, name) in names_vec.iter().enumerate() {
            let version = &versions[i % versions.len()];
            json_deps.push_str(&format!("    \"{}\": \"{}\",\n", name, version));
        }
        json_deps.push_str("    \"dummy\": \"0.0.0\"\n  }\n}");

        fs::write(temp_dir.path().join("package.json"), json_deps).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // All declared dependencies should be parsed (plus dummy)
        prop_assert!(parsed.len() >= names_vec.len());

        // Each declared dependency should be found
        for name in &names_vec {
            let found = parsed.iter().find(|d| &d.name == name);
            prop_assert!(found.is_some(), "Dependency {} not found", name);
        }
    }

    /// Property: Python dependencies are correctly parsed from requirements.txt
    #[test]
    fn prop_python_dependencies_parsed(
        names in prop::collection::hash_set(r"[a-z][a-z0-9\-]*", 1..5),
        versions in prop::collection::vec(r"[0-9]+\.[0-9]+\.[0-9]+", 1..5)
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build requirements.txt content with unique names
        let mut requirements = String::new();
        let names_vec: Vec<_> = names.into_iter().collect();
        for (i, name) in names_vec.iter().enumerate() {
            let version = &versions[i % versions.len()];
            requirements.push_str(&format!("{}=={}\n", name, version));
        }

        fs::write(temp_dir.path().join("requirements.txt"), requirements).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // All declared dependencies should be parsed
        prop_assert_eq!(parsed.len(), names_vec.len());

        // Each parsed dependency should be found
        for name in &names_vec {
            let found = parsed.iter().find(|d| &d.name == name);
            prop_assert!(found.is_some(), "Dependency {} not found", name);
        }
    }

    /// Property: Go dependencies are correctly parsed from go.mod
    #[test]
    fn prop_go_dependencies_parsed(
        deps in prop::collection::vec(
            (r"github\.com/[a-z][a-z0-9\-]*/[a-z][a-z0-9\-]*", r"v[0-9]+\.[0-9]+\.[0-9]+"),
            1..5
        )
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build go.mod content
        let mut go_mod = String::from("module test\n\ngo 1.21\n\nrequire (\n");
        for (name, version) in &deps {
            go_mod.push_str(&format!("    {} {}\n", name, version));
        }
        go_mod.push_str(")\n");

        fs::write(temp_dir.path().join("go.mod"), go_mod).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // All declared dependencies should be parsed
        prop_assert_eq!(parsed.len(), deps.len());

        // Each parsed dependency should have matching name and version
        for (expected_name, expected_version) in &deps {
            let found = parsed.iter().find(|d| &d.name == expected_name);
            prop_assert!(found.is_some(), "Dependency {} not found", expected_name);
            prop_assert_eq!(&found.unwrap().version, expected_version);
        }
    }

    /// Property: All parsed dependencies have non-empty names and versions
    #[test]
    fn prop_dependencies_have_valid_fields(
        deps in prop::collection::vec(
            (r"[a-z][a-z0-9_]*", r"[0-9]+\.[0-9]+\.[0-9]+"),
            1..5
        )
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build Cargo.toml
        let mut cargo_toml = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for (name, version) in &deps {
            cargo_toml.push_str(&format!("{} = \"{}\"\n", name, version));
        }

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // All dependencies should have non-empty names and versions
        for dep in &parsed {
            prop_assert!(!dep.name.is_empty(), "Dependency name is empty");
            prop_assert!(!dep.version.is_empty(), "Dependency version is empty");
        }
    }

    /// Property: Dependency parsing is deterministic
    #[test]
    fn prop_dependency_parsing_deterministic(
        deps in prop::collection::vec(
            (r"[a-z][a-z0-9_]*", r"[0-9]+\.[0-9]+\.[0-9]+"),
            1..5
        )
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build Cargo.toml
        let mut cargo_toml = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        for (name, version) in &deps {
            cargo_toml.push_str(&format!("{} = \"{}\"\n", name, version));
        }

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        // Parse twice
        let parsed1 = analyzer.analyze(temp_dir.path()).unwrap();
        let parsed2 = analyzer.analyze(temp_dir.path()).unwrap();

        // Results should be identical
        prop_assert_eq!(parsed1.len(), parsed2.len());
        for (dep1, dep2) in parsed1.iter().zip(parsed2.iter()) {
            prop_assert_eq!(&dep1.name, &dep2.name);
            prop_assert_eq!(&dep1.version, &dep2.version);
            prop_assert_eq!(dep1.is_dev, dep2.is_dev);
        }
    }

    /// Property: Dev dependencies are correctly marked
    #[test]
    fn prop_dev_dependencies_marked(
        dev_names in prop::collection::hash_set(r"[a-z][a-z0-9_]*", 1..3),
        regular_names in prop::collection::hash_set(r"[a-z][a-z0-9_]*", 1..3),
        versions in prop::collection::vec(r"[0-9]+\.[0-9]+\.[0-9]+", 1..3)
    ) {
        let analyzer = DependencyAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Build Cargo.toml with both regular and dev dependencies
        let mut cargo_toml = String::from("[package]\nname = \"test\"\nversion = \"0.1.0\"\n\n[dependencies]\n");
        let regular_vec: Vec<_> = regular_names.into_iter().collect();
        for (i, name) in regular_vec.iter().enumerate() {
            let version = &versions[i % versions.len()];
            cargo_toml.push_str(&format!("{} = \"{}\"\n", name, version));
        }
        cargo_toml.push_str("\n[dev-dependencies]\n");
        let dev_vec: Vec<_> = dev_names.into_iter().collect();
        for (i, name) in dev_vec.iter().enumerate() {
            let version = &versions[i % versions.len()];
            cargo_toml.push_str(&format!("{} = \"{}\"\n", name, version));
        }

        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        let parsed = analyzer.analyze(temp_dir.path()).unwrap();

        // Check that dev dependencies are marked correctly
        for name in &dev_vec {
            let found = parsed.iter().find(|d| &d.name == name);
            prop_assert!(found.is_some(), "Dev dependency {} not found", name);
            prop_assert!(found.unwrap().is_dev, "Dev dependency {} not marked as dev", name);
        }

        // Check that regular dependencies are not marked as dev
        for name in &regular_vec {
            let found = parsed.iter().find(|d| &d.name == name);
            prop_assert!(found.is_some(), "Regular dependency {} not found", name);
            prop_assert!(!found.unwrap().is_dev, "Regular dependency {} incorrectly marked as dev", name);
        }
    }
}
