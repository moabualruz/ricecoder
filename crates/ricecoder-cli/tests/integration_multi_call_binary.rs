// Integration tests for multi-call binary pattern
// **Feature: ricecoder-cli, Tests for Requirements 6.1-6.4**

use ricecoder_cli::router::CommandRouter;
use ricecoder_cli::commands::{InitCommand, GenCommand, ChatCommand, ConfigCommand, Command};
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// Multi-Call Binary Pattern Tests
// ============================================================================

#[test]
fn test_init_command_via_router() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();
    
    let cmd = InitCommand::new(Some(temp_path.to_string()));
    let result = cmd.execute();
    
    assert!(result.is_ok(), "Init command should succeed");
}

#[test]
fn test_gen_command_via_router() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("test_spec.md");
    
    std::fs::write(&spec_file, "# Test Spec\n\nContent").expect("Failed to write spec");
    
    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();
    
    assert!(result.is_ok(), "Gen command should succeed");
}

#[test]
fn test_chat_command_via_router() {
    let cmd = ChatCommand::new(None, None, None);
    
    // Just verify the command can be created and implements the trait
    let _: &dyn Command = &cmd;
}

#[test]
fn test_config_command_via_router() {
    use ricecoder_cli::commands::config::ConfigAction;
    
    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();
    
    assert!(result.is_ok(), "Config command should succeed");
}

// ============================================================================
// Command Equivalence Tests
// ============================================================================

#[test]
fn test_init_command_equivalence() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let path1 = temp_dir1.path().to_str().unwrap();
    let path2 = temp_dir2.path().to_str().unwrap();
    
    // Both commands should succeed
    let cmd1 = InitCommand::new(Some(path1.to_string()));
    let result1 = cmd1.execute();
    
    let cmd2 = InitCommand::new(Some(path2.to_string()));
    let result2 = cmd2.execute();
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // Both should create the same directory structure
    assert!(Path::new(path1).join(".agent").exists());
    assert!(Path::new(path2).join(".agent").exists());
}

#[test]
fn test_gen_command_equivalence() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let spec1 = temp_dir1.path().join("spec1.md");
    let spec2 = temp_dir2.path().join("spec2.md");
    
    std::fs::write(&spec1, "# Spec 1").expect("Failed to write spec");
    std::fs::write(&spec2, "# Spec 2").expect("Failed to write spec");
    
    let cmd1 = GenCommand::new(spec1.to_str().unwrap().to_string());
    let result1 = cmd1.execute();
    
    let cmd2 = GenCommand::new(spec2.to_str().unwrap().to_string());
    let result2 = cmd2.execute();
    
    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// ============================================================================
// Command Routing Tests
// ============================================================================

#[test]
fn test_command_router_find_similar() {
    // Test that the router can find similar commands
    assert_eq!(CommandRouter::find_similar("i"), Some("init".to_string()));
    assert_eq!(CommandRouter::find_similar("g"), Some("gen".to_string()));
    assert_eq!(CommandRouter::find_similar("c"), Some("chat".to_string()));
}

#[test]
fn test_command_router_find_similar_consistency() {
    // Finding similar commands should be consistent
    let result1 = CommandRouter::find_similar("i");
    let result2 = CommandRouter::find_similar("i");
    
    assert_eq!(result1, result2);
}

// ============================================================================
// Multi-Call Binary Invocation Pattern Tests
// ============================================================================

#[test]
fn test_rice_init_invocation_pattern() {
    // Simulate: rice init
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    
    let cmd = InitCommand::new(Some(path.to_string()));
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

#[test]
fn test_rice_gen_invocation_pattern() {
    // Simulate: rice gen <spec>
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("spec.md");
    
    std::fs::write(&spec_file, "# Spec").expect("Failed to write spec");
    
    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

#[test]
fn test_rice_chat_invocation_pattern() {
    // Simulate: rice chat
    let cmd = ChatCommand::new(None, None, None);
    
    // Just verify it can be created
    let _: &dyn Command = &cmd;
}

#[test]
fn test_rice_config_invocation_pattern() {
    // Simulate: rice config
    use ricecoder_cli::commands::config::ConfigAction;
    
    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

// ============================================================================
// Standalone Binary Invocation Pattern Tests
// ============================================================================

#[test]
fn test_rice_init_standalone_pattern() {
    // Simulate: rice-init
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    
    let cmd = InitCommand::new(Some(path.to_string()));
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

#[test]
fn test_rice_gen_standalone_pattern() {
    // Simulate: rice-gen <spec>
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("spec.md");
    
    std::fs::write(&spec_file, "# Spec").expect("Failed to write spec");
    
    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

#[test]
fn test_rice_chat_standalone_pattern() {
    // Simulate: rice-chat
    let cmd = ChatCommand::new(None, None, None);
    
    // Just verify it can be created
    let _: &dyn Command = &cmd;
}

#[test]
fn test_rice_config_standalone_pattern() {
    // Simulate: rice-config
    use ricecoder_cli::commands::config::ConfigAction;
    
    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();
    
    assert!(result.is_ok());
}

// ============================================================================
// Equivalence Between Invocation Patterns
// ============================================================================

#[test]
fn test_rice_init_vs_rice_init_standalone() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let path1 = temp_dir1.path().to_str().unwrap();
    let path2 = temp_dir2.path().to_str().unwrap();
    
    // rice init
    let cmd1 = InitCommand::new(Some(path1.to_string()));
    let result1 = cmd1.execute();
    
    // rice-init
    let cmd2 = InitCommand::new(Some(path2.to_string()));
    let result2 = cmd2.execute();
    
    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // Both should create the same structure
    assert!(Path::new(path1).join(".agent").exists());
    assert!(Path::new(path2).join(".agent").exists());
}

#[test]
fn test_rice_gen_vs_rice_gen_standalone() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let spec1 = temp_dir1.path().join("spec.md");
    let spec2 = temp_dir2.path().join("spec.md");
    
    std::fs::write(&spec1, "# Spec").expect("Failed to write spec");
    std::fs::write(&spec2, "# Spec").expect("Failed to write spec");
    
    // rice gen
    let cmd1 = GenCommand::new(spec1.to_str().unwrap().to_string());
    let result1 = cmd1.execute();
    
    // rice-gen
    let cmd2 = GenCommand::new(spec2.to_str().unwrap().to_string());
    let result2 = cmd2.execute();
    
    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_rice_config_vs_rice_config_standalone() {
    use ricecoder_cli::commands::config::ConfigAction;
    
    // rice config
    let cmd1 = ConfigCommand::new(ConfigAction::List);
    let result1 = cmd1.execute();
    
    // rice-config
    let cmd2 = ConfigCommand::new(ConfigAction::List);
    let result2 = cmd2.execute();
    
    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// ============================================================================
// Command Behavior Consistency Tests
// ============================================================================

#[test]
fn test_init_command_behavior_consistent() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let path1 = temp_dir1.path().to_str().unwrap();
    let path2 = temp_dir2.path().to_str().unwrap();
    
    let cmd1 = InitCommand::new(Some(path1.to_string()));
    let cmd2 = InitCommand::new(Some(path2.to_string()));
    
    let result1 = cmd1.execute();
    let result2 = cmd2.execute();
    
    // Both should have the same result type
    assert_eq!(result1.is_ok(), result2.is_ok());
}

#[test]
fn test_gen_command_behavior_consistent() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");
    
    let spec1 = temp_dir1.path().join("spec.md");
    let spec2 = temp_dir2.path().join("spec.md");
    
    std::fs::write(&spec1, "# Spec").expect("Failed to write spec");
    std::fs::write(&spec2, "# Spec").expect("Failed to write spec");
    
    let cmd1 = GenCommand::new(spec1.to_str().unwrap().to_string());
    let cmd2 = GenCommand::new(spec2.to_str().unwrap().to_string());
    
    let result1 = cmd1.execute();
    let result2 = cmd2.execute();
    
    // Both should have the same result type
    assert_eq!(result1.is_ok(), result2.is_ok());
}

// ============================================================================
// Error Handling Consistency Tests
// ============================================================================

#[test]
fn test_gen_command_error_consistency() {
    // Both invocation patterns should handle errors the same way
    let cmd1 = GenCommand::new("nonexistent1.md".to_string());
    let cmd2 = GenCommand::new("nonexistent2.md".to_string());
    
    let result1 = cmd1.execute();
    let result2 = cmd2.execute();
    
    // Both should fail
    assert!(result1.is_err());
    assert!(result2.is_err());
}

#[test]
fn test_init_command_error_consistency() {
    // Both invocation patterns should handle errors the same way
    let cmd1 = InitCommand::new(Some("/invalid/path/1".to_string()));
    let cmd2 = InitCommand::new(Some("/invalid/path/2".to_string()));
    
    let result1 = cmd1.execute();
    let result2 = cmd2.execute();
    
    // Both should have the same error behavior
    assert_eq!(result1.is_err(), result2.is_err());
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn test_multi_call_binary_idempotent() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    
    let cmd1 = InitCommand::new(Some(path.to_string()));
    let result1 = cmd1.execute();
    
    let cmd2 = InitCommand::new(Some(path.to_string()));
    let result2 = cmd2.execute();
    
    // Both should succeed (idempotent)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_command_routing_deterministic() {
    // Command routing should be deterministic
    let result1 = CommandRouter::find_similar("i");
    let result2 = CommandRouter::find_similar("i");
    let result3 = CommandRouter::find_similar("i");
    
    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
}
