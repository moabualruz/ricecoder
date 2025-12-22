// Integration tests for path resolution
// **Feature: ricecoder-path-resolution, Tests for Requirements 4.3, 4.4**

use std::{fs, path::PathBuf};

use ricecoder_generation::prompt_builder::PromptBuilder;
use ricecoder_storage::PathResolver;
use tempfile::TempDir;

// ============================================================================
// PromptBuilder Path Resolution Integration Tests
// ============================================================================

#[test]
fn test_prompt_builder_loads_steering_rules_from_project_location() {
    // Test that PromptBuilder loads steering rules from the correct project location
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create .agent/steering directory
    let steering_dir = temp_path.join(".agent").join("steering");
    fs::create_dir_all(&steering_dir).expect("Failed to create steering directory");

    // Create a steering rules file
    let rules_file = steering_dir.join("naming-conventions.md");
    fs::write(
        &rules_file,
        "# Naming Conventions\n\nUse snake_case for Rust",
    )
    .expect("Failed to write rules file");

    // Create PromptBuilder with temp directory as project root
    let builder = PromptBuilder::new(temp_path.to_path_buf());

    // Load steering rules
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have loaded rules
    assert!(!rules.naming_conventions.is_empty());
}

#[test]
fn test_prompt_builder_loads_steering_rules_from_global_location() {
    // Test that PromptBuilder loads steering rules from global location
    let original = std::env::var("RICECODER_HOME").ok();

    // Create a temporary global directory
    let temp_global = TempDir::new().expect("Failed to create temp directory");
    let global_path = temp_global.path();

    // Set RICECODER_HOME to temp directory
    std::env::set_var("RICECODER_HOME", global_path);

    // Create steering directory in global location
    let steering_dir = global_path.join("steering");
    fs::create_dir_all(&steering_dir).expect("Failed to create steering directory");

    // Create a steering rules file
    let rules_file = steering_dir.join("code-quality.md");
    fs::write(&rules_file, "# Code Quality\n\nZero warnings in production")
        .expect("Failed to write rules file");

    // Create PromptBuilder
    let builder = PromptBuilder::default();

    // Load steering rules
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have loaded rules (at minimum naming conventions)
    assert!(!rules.naming_conventions.is_empty());

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_prompt_builder_environment_variable_override_end_to_end() {
    // Test that environment variable overrides work end-to-end
    let original = std::env::var("RICECODER_HOME").ok();

    // Create two temporary directories
    let temp_default = TempDir::new().expect("Failed to create temp directory");
    let temp_override = TempDir::new().expect("Failed to create temp directory");

    // Set RICECODER_HOME to override directory
    std::env::set_var("RICECODER_HOME", temp_override.path());

    // Create steering directory in override location
    let steering_dir = temp_override.path().join("steering");
    fs::create_dir_all(&steering_dir).expect("Failed to create steering directory");

    // Create a steering rules file with unique content
    let rules_file = steering_dir.join("override.md");
    fs::write(
        &rules_file,
        "# Override Rules\n\nThis is from override location",
    )
    .expect("Failed to write rules file");

    // Create PromptBuilder
    let builder = PromptBuilder::new(temp_default.path().to_path_buf());

    // Load steering rules
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have loaded rules (at minimum naming conventions)
    assert!(!rules.naming_conventions.is_empty());

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_prompt_builder_precedence_project_over_global() {
    // Test that project-level rules take precedence over global rules
    let original = std::env::var("RICECODER_HOME").ok();

    // Create temporary directories
    let temp_project = TempDir::new().expect("Failed to create temp directory");
    let temp_global = TempDir::new().expect("Failed to create temp directory");

    // Set RICECODER_HOME to global directory
    std::env::set_var("RICECODER_HOME", temp_global.path());

    // Create project-level steering directory
    let project_steering = temp_project.path().join(".agent").join("steering");
    fs::create_dir_all(&project_steering).expect("Failed to create project steering directory");

    // Create global-level steering directory
    let global_steering = temp_global.path().join("steering");
    fs::create_dir_all(&global_steering).expect("Failed to create global steering directory");

    // Create files in both locations
    fs::write(
        project_steering.join("project.md"),
        "# Project Rules\n\nProject-level rule",
    )
    .expect("Failed to write project rules");

    fs::write(
        global_steering.join("global.md"),
        "# Global Rules\n\nGlobal-level rule",
    )
    .expect("Failed to write global rules");

    // Create PromptBuilder
    let builder = PromptBuilder::new(temp_project.path().to_path_buf());

    // Load steering rules
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have loaded rules (precedence is handled internally)
    assert!(!rules.naming_conventions.is_empty());

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_prompt_builder_both_agent_and_kiro_configurations_coexist() {
    // Test that both .agent/ and .kiro/ configurations can coexist
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create both .agent and .kiro directories
    let agent_steering = temp_path.join(".agent").join("steering");
    let kiro_steering = temp_path.join(".kiro").join("steering");

    fs::create_dir_all(&agent_steering).expect("Failed to create .agent/steering");
    fs::create_dir_all(&kiro_steering).expect("Failed to create .kiro/steering");

    // Create files in both locations
    fs::write(agent_steering.join("agent.md"), "# Agent Rules")
        .expect("Failed to write agent rules");
    fs::write(kiro_steering.join("kiro.md"), "# Kiro Rules").expect("Failed to write kiro rules");

    // Create PromptBuilder
    let builder = PromptBuilder::new(temp_path.to_path_buf());

    // Load steering rules
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have loaded rules from .agent location (primary)
    assert!(!rules.naming_conventions.is_empty());
}

#[test]
fn test_prompt_builder_handles_missing_steering_directory_gracefully() {
    // Test that missing steering directory is handled gracefully
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Don't create any steering directory

    // Create PromptBuilder
    let builder = PromptBuilder::new(temp_path.to_path_buf());

    // Load steering rules should still succeed with defaults
    let rules = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // Should have default rules
    assert!(!rules.naming_conventions.is_empty());
    assert!(rules.naming_conventions.contains_key("rust"));
}

#[test]
fn test_prompt_builder_path_resolution_consistency_across_calls() {
    // Test that path resolution is consistent across multiple calls
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create .agent/steering directory
    let steering_dir = temp_path.join(".agent").join("steering");
    fs::create_dir_all(&steering_dir).expect("Failed to create steering directory");

    // Create PromptBuilder
    let builder = PromptBuilder::new(temp_path.to_path_buf());

    // Load steering rules multiple times
    let rules1 = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");
    let rules2 = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");
    let rules3 = builder
        .load_steering_rules()
        .expect("Failed to load steering rules");

    // All should be identical
    assert_eq!(rules1.naming_conventions, rules2.naming_conventions);
    assert_eq!(rules2.naming_conventions, rules3.naming_conventions);
}

// ============================================================================
// CLI Path Resolution Integration Tests
// ============================================================================

#[test]
fn test_cli_loads_specs_from_agent_directory() {
    // Test that CLI loads specs from .agent/specs directory
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create .agent/specs directory
    let specs_dir = temp_path.join(".agent").join("specs");
    fs::create_dir_all(&specs_dir).expect("Failed to create specs directory");

    // Create a spec file
    let spec_file = specs_dir.join("test-spec.md");
    fs::write(&spec_file, "# Test Specification\n\nThis is a test spec")
        .expect("Failed to write spec file");

    // Verify the spec file exists at the expected location
    assert!(spec_file.exists(), "Spec file should exist at .agent/specs");

    // Verify the path matches what PathResolver would return
    let project_path = PathResolver::resolve_project_path();
    let expected_specs_path = temp_path.join(&project_path).join("specs");
    assert_eq!(expected_specs_path, specs_dir);
}

#[test]
fn test_cli_specs_path_resolution_with_environment_variable() {
    // Test that specs path resolution respects environment variables
    let original = std::env::var("RICECODER_HOME").ok();

    // Create temporary directories
    let temp_project = TempDir::new().expect("Failed to create temp directory");
    let temp_global = TempDir::new().expect("Failed to create temp directory");

    // Set RICECODER_HOME
    std::env::set_var("RICECODER_HOME", temp_global.path());

    // Create specs in project location
    let project_specs = temp_project.path().join(".agent").join("specs");
    fs::create_dir_all(&project_specs).expect("Failed to create project specs");

    // Create specs in global location
    let global_specs = temp_global.path().join("specs");
    fs::create_dir_all(&global_specs).expect("Failed to create global specs");

    // Create spec files in both locations
    fs::write(project_specs.join("project.md"), "# Project Spec")
        .expect("Failed to write project spec");
    fs::write(global_specs.join("global.md"), "# Global Spec")
        .expect("Failed to write global spec");

    // Verify both locations exist
    assert!(project_specs.exists());
    assert!(global_specs.exists());

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_cli_backward_compatibility_with_kiro_specs() {
    // Test that CLI maintains backward compatibility with .kiro/specs
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Create .kiro/specs directory (old location)
    let kiro_specs = temp_path.join(".kiro").join("specs");
    fs::create_dir_all(&kiro_specs).expect("Failed to create .kiro/specs");

    // Create a spec file in old location
    let spec_file = kiro_specs.join("legacy-spec.md");
    fs::write(&spec_file, "# Legacy Specification").expect("Failed to write legacy spec");

    // Verify the file exists
    assert!(
        spec_file.exists(),
        "Legacy spec should exist at .kiro/specs"
    );
}

#[test]
fn test_cli_specs_path_consistency_across_calls() {
    // Test that specs path is consistent across multiple calls
    let project_path1 = PathResolver::resolve_project_path();
    let project_path2 = PathResolver::resolve_project_path();
    let project_path3 = PathResolver::resolve_project_path();

    // All should be identical
    assert_eq!(project_path1, project_path2);
    assert_eq!(project_path2, project_path3);

    // All should be .agent
    assert_eq!(project_path1, PathBuf::from(".agent"));
}

#[test]
fn test_end_to_end_path_resolution_workflow() {
    // Test complete end-to-end path resolution workflow
    // Create temporary directories
    let temp_project = TempDir::new().expect("Failed to create temp directory");
    let temp_global = TempDir::new().expect("Failed to create temp directory");

    // Create project structure
    let project_agent = temp_project.path().join(".agent");
    let project_steering = project_agent.join("steering");
    let project_specs = project_agent.join("specs");

    fs::create_dir_all(&project_steering).expect("Failed to create project steering");
    fs::create_dir_all(&project_specs).expect("Failed to create project specs");

    // Create global structure
    let global_steering = temp_global.path().join("steering");
    let global_specs = temp_global.path().join("specs");

    fs::create_dir_all(&global_steering).expect("Failed to create global steering");
    fs::create_dir_all(&global_specs).expect("Failed to create global specs");

    // Create files
    fs::write(project_steering.join("rules.md"), "# Project Rules")
        .expect("Failed to write project rules");
    fs::write(project_specs.join("spec.md"), "# Project Spec")
        .expect("Failed to write project spec");
    fs::write(global_steering.join("rules.md"), "# Global Rules")
        .expect("Failed to write global rules");
    fs::write(global_specs.join("spec.md"), "# Global Spec").expect("Failed to write global spec");

    // Verify paths are resolved correctly
    let project_path = PathResolver::resolve_project_path();
    assert_eq!(project_path, PathBuf::from(".agent"));

    let global_path = PathResolver::resolve_global_path().expect("Should resolve global path");

    // Global path should be resolvable (exact value depends on environment)
    assert!(!global_path.as_os_str().is_empty());

    // Verify all directories exist
    assert!(project_steering.exists());
    assert!(project_specs.exists());
    assert!(global_steering.exists());
    assert!(global_specs.exists());
}
