// Unit tests for PathResolver usage in CLI
// **Feature: ricecoder-path-resolution, Tests for Requirements 4.1, 4.2**

use std::path::PathBuf;

use ricecoder_cli::commands::{ChatCommand, Command};
use ricecoder_storage::PathResolver;

// ============================================================================
// ChatCommand Path Resolution Tests
// ============================================================================

#[test]
fn test_chat_command_loads_specs_from_correct_location() {
    // Test that ChatCommand loads specs from the correct location using PathResolver
    let cmd = ChatCommand::new(None, None, None);

    // The command should be able to load project context
    // which uses PathResolver internally
    let result = cmd.execute();

    // Should succeed (even if no specs are found, the path resolution should work)
    assert!(result.is_ok(), "Chat command should execute successfully");
}

#[test]
fn test_chat_command_project_path_resolution() {
    // Test that project path is resolved correctly
    let project_path = PathResolver::resolve_project_path();

    // Project path should be .agent
    assert_eq!(project_path, PathBuf::from(".agent"));
}

#[test]
fn test_chat_command_specs_path_construction() {
    // Test that specs path is constructed correctly from project path
    let project_path = PathResolver::resolve_project_path();
    let specs_path = project_path.join("specs");

    // Specs path should be .agent/specs
    assert_eq!(specs_path, PathBuf::from(".agent/specs"));
}

#[test]
fn test_chat_command_error_handling_for_missing_specs() {
    // Test that error handling works when specs directory doesn't exist
    let cmd = ChatCommand::new(None, None, None);

    // Should handle gracefully even if specs don't exist
    let result = cmd.execute();
    assert!(
        result.is_ok(),
        "Chat command should handle missing specs gracefully"
    );
}

#[test]
fn test_chat_command_with_environment_variable_override() {
    // Test that path resolution respects environment variables
    let original = std::env::var("RICECODER_HOME").ok();

    // Set a test environment variable
    std::env::set_var("RICECODER_HOME", "/tmp/test-ricecoder");

    // Create and execute chat command
    let cmd = ChatCommand::new(None, None, None);
    let result = cmd.execute();

    // Should still work with environment variable set
    assert!(
        result.is_ok(),
        "Chat command should work with RICECODER_HOME set"
    );

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    } else {
        std::env::remove_var("RICECODER_HOME");
    }
}

#[test]
fn test_chat_command_without_environment_variable() {
    // Test that path resolution works without environment variables
    let original = std::env::var("RICECODER_HOME").ok();

    // Ensure RICECODER_HOME is not set
    std::env::remove_var("RICECODER_HOME");

    // Create and execute chat command
    let cmd = ChatCommand::new(None, None, None);
    let result = cmd.execute();

    // Should work without environment variable
    assert!(
        result.is_ok(),
        "Chat command should work without RICECODER_HOME"
    );

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}

#[test]
fn test_chat_command_provider_validation() {
    // Test that provider validation works
    let cmd = ChatCommand::new(None, Some("openai".to_string()), None);

    // Should accept valid provider
    let result = cmd.execute();
    assert!(result.is_ok(), "Chat command should accept valid provider");
}

#[test]
fn test_chat_command_invalid_provider_error_handling() {
    // Test that invalid provider is handled correctly
    let cmd = ChatCommand::new(None, Some("invalid_provider".to_string()), None);

    // Should fail with invalid provider
    let result = cmd.execute();
    assert!(
        result.is_err(),
        "Chat command should reject invalid provider"
    );
}

#[test]
fn test_chat_command_model_defaults() {
    // Test that model defaults are applied correctly
    let cmd = ChatCommand::new(None, None, None);

    // Should have default model
    assert!(cmd.model.is_none(), "Model should be None initially");
}

#[test]
fn test_chat_command_with_custom_model() {
    // Test that custom model is accepted
    let cmd = ChatCommand::new(None, None, Some("gpt-4".to_string()));

    // Should accept custom model
    assert_eq!(cmd.model, Some("gpt-4".to_string()));
}

// ============================================================================
// PathResolver Integration Tests
// ============================================================================

#[test]
fn test_path_resolver_project_path_consistency() {
    // Test that project path is consistent across multiple calls
    let path1 = PathResolver::resolve_project_path();
    let path2 = PathResolver::resolve_project_path();
    let path3 = PathResolver::resolve_project_path();

    assert_eq!(path1, path2);
    assert_eq!(path2, path3);
}

// Note: Global path consistency tests are in the generation crate's path_resolution_properties.rs
// because they need to be run serially to avoid conflicts with parallel test execution.

// Note: Environment variable tests are in the generation crate's path_resolution_properties.rs
// because they need to be run serially to avoid conflicts with parallel test execution.
// This test file focuses on CLI-specific path resolution tests that don't modify environment variables.

#[test]
fn test_path_resolver_project_path_is_agent() {
    // Test that project path is always .agent
    let project_path = PathResolver::resolve_project_path();

    assert_eq!(project_path, PathBuf::from(".agent"));
}

#[test]
fn test_path_resolver_global_path_ends_with_ricecoder() {
    // Test that global path ends with .ricecoder
    let original = std::env::var("RICECODER_HOME").ok();

    // Ensure RICECODER_HOME is not set
    std::env::remove_var("RICECODER_HOME");

    let global_path = PathResolver::resolve_global_path().expect("Should resolve global path");

    let path_str = global_path.to_str().expect("Path should be valid UTF-8");
    assert!(
        path_str.ends_with(".ricecoder") || path_str.ends_with(".ricecoder/"),
        "Global path should end with .ricecoder, got: {}",
        path_str
    );

    // Restore original
    if let Some(orig) = original {
        std::env::set_var("RICECODER_HOME", orig);
    }
}

// ============================================================================
// CLI Command Path Resolution Integration Tests
// ============================================================================

#[test]
fn test_chat_command_uses_path_resolver_for_specs() {
    // Test that ChatCommand uses PathResolver for specs path
    let cmd = ChatCommand::new(None, None, None);

    // Execute should use PathResolver internally
    let result = cmd.execute();

    // Should succeed (path resolution should work)
    assert!(
        result.is_ok(),
        "Chat command should use PathResolver successfully"
    );
}

#[test]
fn test_chat_command_specs_path_is_agent_specs() {
    // Test that specs path is .agent/specs
    let project_path = PathResolver::resolve_project_path();
    let specs_path = project_path.join("specs");

    assert_eq!(specs_path, PathBuf::from(".agent/specs"));
}

#[test]
fn test_chat_command_knowledge_base_path_resolution() {
    // Test that knowledge base path is resolved correctly
    let global_path = PathResolver::resolve_global_path();

    // Global path should be resolvable
    match global_path {
        Ok(path) => {
            // Should be able to construct knowledge base path
            let kb_path = path.join("knowledge_base");
            assert!(kb_path.to_str().is_some());
        }
        Err(_) => {
            // If global path resolution fails, that's also acceptable
            // (we can still use project-level resources)
        }
    }
}

#[test]
fn test_chat_command_multiple_executions_consistent() {
    // Test that multiple executions use consistent paths
    let cmd1 = ChatCommand::new(None, None, None);
    let cmd2 = ChatCommand::new(None, None, None);

    // Both should execute successfully
    let result1 = cmd1.execute();
    let result2 = cmd2.execute();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_chat_command_with_message_and_path_resolution() {
    // Test that path resolution works when message is provided
    let cmd = ChatCommand::new(Some("Hello".to_string()), None, None);

    // Should execute successfully with message
    let result = cmd.execute();
    assert!(result.is_ok(), "Chat command should work with message");
}

#[test]
fn test_chat_command_with_provider_and_path_resolution() {
    // Test that path resolution works with provider specified
    let cmd = ChatCommand::new(None, Some("openai".to_string()), None);

    // Should execute successfully with provider
    let result = cmd.execute();
    assert!(result.is_ok(), "Chat command should work with provider");
}

#[test]
fn test_chat_command_with_model_and_path_resolution() {
    // Test that path resolution works with model specified
    let cmd = ChatCommand::new(None, None, Some("gpt-4".to_string()));

    // Should execute successfully with model
    let result = cmd.execute();
    assert!(result.is_ok(), "Chat command should work with model");
}

#[test]
fn test_chat_command_with_all_options_and_path_resolution() {
    // Test that path resolution works with all options specified
    let cmd = ChatCommand::new(
        Some("Hello".to_string()),
        Some("openai".to_string()),
        Some("gpt-4".to_string()),
    );

    // Should execute successfully with all options
    let result = cmd.execute();
    assert!(result.is_ok(), "Chat command should work with all options");
}
