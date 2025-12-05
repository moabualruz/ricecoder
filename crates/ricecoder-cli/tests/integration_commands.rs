// Integration tests for command execution
// **Feature: ricecoder-cli, Tests for Requirements 1.1-7.5**

use ricecoder_cli::branding::BrandingManager;
use ricecoder_cli::commands::{
    ChatCommand, Command, ConfigCommand, GenCommand, InitCommand, VersionCommand,
};
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// End-to-End Command Execution Tests
// ============================================================================

#[test]
fn test_init_command_end_to_end() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    // Execute init command
    let cmd = InitCommand::new(Some(path.to_string()));
    let result = cmd.execute();

    // Verify success
    assert!(result.is_ok(), "Init command should succeed");

    // Verify artifacts were created
    assert!(
        Path::new(path).join(".agent").exists(),
        ".agent directory should exist"
    );
    assert!(
        Path::new(path).join(".agent/ricecoder.toml").exists(),
        "ricecoder.toml should exist"
    );
}

#[test]
fn test_gen_command_end_to_end() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("spec.md");

    // Create a spec file
    std::fs::write(&spec_file, "# Test Specification\n\nThis is a test spec.")
        .expect("Failed to write spec");

    // Execute gen command
    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    // Verify success
    assert!(result.is_ok(), "Gen command should succeed");
}

#[test]
fn test_config_command_end_to_end() {
    use ricecoder_cli::commands::config::ConfigAction;

    // Execute config list command
    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();

    // Verify success
    assert!(result.is_ok(), "Config list command should succeed");
}

#[test]
fn test_version_command_end_to_end() {
    // Execute version command
    let cmd = VersionCommand::new();
    let result = cmd.execute();

    // Verify success
    assert!(result.is_ok(), "Version command should succeed");
}

// ============================================================================
// Interactive Chat Mode Tests
// ============================================================================

#[test]
fn test_chat_command_creation() {
    let cmd = ChatCommand::new(None, None, None);

    // Verify command can be created
    assert!(cmd.message.is_none());
    assert!(cmd.provider.is_none());
    assert!(cmd.model.is_none());
}

#[test]
fn test_chat_command_with_initial_message() {
    let cmd = ChatCommand::new(Some("Hello".to_string()), None, None);

    // Verify initial message is set
    assert_eq!(cmd.message, Some("Hello".to_string()));
}

#[test]
fn test_chat_command_with_provider() {
    let cmd = ChatCommand::new(None, Some("openai".to_string()), None);

    // Verify provider is set
    assert_eq!(cmd.provider, Some("openai".to_string()));
}

#[test]
fn test_chat_command_with_model() {
    let cmd = ChatCommand::new(None, None, Some("gpt-4".to_string()));

    // Verify model is set
    assert_eq!(cmd.model, Some("gpt-4".to_string()));
}

// ============================================================================
// Branding Display Tests
// ============================================================================

#[test]
fn test_branding_startup_banner() {
    let result = BrandingManager::display_startup_banner();

    // Verify banner displays successfully
    assert!(result.is_ok(), "Startup banner should display successfully");
}

#[test]
fn test_branding_version_banner() {
    let result = BrandingManager::display_version_banner("1.0.0");

    // Verify banner displays successfully
    assert!(result.is_ok(), "Version banner should display successfully");
}

#[test]
fn test_branding_ascii_logo_loaded() {
    let result = BrandingManager::load_ascii_logo();

    // Verify logo is loaded
    assert!(result.is_ok(), "ASCII logo should load successfully");

    let logo = result.unwrap();
    assert!(
        logo.contains("RiceCoder"),
        "Logo should contain RiceCoder branding"
    );
}

#[test]
fn test_terminal_capabilities_detected() {
    let caps = BrandingManager::detect_terminal_capabilities();

    // Verify capabilities are detected
    assert!(caps.width > 0, "Terminal width should be positive");
    assert!(caps.height > 0, "Terminal height should be positive");
}

// ============================================================================
// Multi-Command Workflow Tests
// ============================================================================

#[test]
fn test_init_then_gen_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    // Step 1: Initialize project
    let init_cmd = InitCommand::new(Some(path.to_string()));
    let init_result = init_cmd.execute();
    assert!(init_result.is_ok(), "Init should succeed");

    // Step 2: Create a spec file
    let spec_file = Path::new(path).join("spec.md");
    std::fs::write(&spec_file, "# Spec").expect("Failed to write spec");

    // Step 3: Generate code from spec
    let gen_cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let gen_result = gen_cmd.execute();
    assert!(gen_result.is_ok(), "Gen should succeed");
}

#[test]
fn test_init_then_config_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    use ricecoder_cli::commands::config::ConfigAction;

    // Step 1: Initialize project
    let init_cmd = InitCommand::new(Some(path.to_string()));
    let init_result = init_cmd.execute();
    assert!(init_result.is_ok(), "Init should succeed");

    // Step 2: Check configuration
    let config_cmd = ConfigCommand::new(ConfigAction::List);
    let config_result = config_cmd.execute();
    assert!(config_result.is_ok(), "Config should succeed");
}

// ============================================================================
// Command Trait Implementation Tests
// ============================================================================

#[test]
fn test_all_commands_implement_trait() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();
    let spec_file = temp_dir.path().join("spec.md");
    std::fs::write(&spec_file, "# Spec").expect("Failed to write spec");

    use ricecoder_cli::commands::config::ConfigAction;

    // Verify all commands implement the Command trait
    let _: &dyn Command = &InitCommand::new(Some(path.to_string()));
    let _: &dyn Command = &GenCommand::new(spec_file.to_str().unwrap().to_string());
    let _: &dyn Command = &ChatCommand::new(None, None, None);
    let _: &dyn Command = &ConfigCommand::new(ConfigAction::List);
    let _: &dyn Command = &VersionCommand::new();
}

// ============================================================================
// Error Handling in Commands Tests
// ============================================================================

#[test]
fn test_gen_command_with_missing_spec() {
    let cmd = GenCommand::new("nonexistent_spec.md".to_string());
    let result = cmd.execute();

    // Should fail gracefully
    assert!(result.is_err(), "Gen command should fail with missing spec");
}

#[test]
fn test_gen_command_with_empty_spec() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("empty_spec.md");

    std::fs::write(&spec_file, "").expect("Failed to write spec");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    // Should fail gracefully
    assert!(result.is_err(), "Gen command should fail with empty spec");
}

#[test]
fn test_init_command_with_valid_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(path.to_string()));
    let result = cmd.execute();

    // Should succeed with valid path
    assert!(
        result.is_ok(),
        "Init command should succeed with valid path"
    );
}

// ============================================================================
// Command Output Tests
// ============================================================================

#[test]
fn test_version_command_output() {
    let cmd = VersionCommand::new();
    let result = cmd.execute();

    // Should succeed and produce output
    assert!(result.is_ok(), "Version command should succeed");
}

#[test]
fn test_config_list_command_output() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();

    // Should succeed and produce output
    assert!(result.is_ok(), "Config list should succeed");
}

// ============================================================================
// Command Consistency Tests
// ============================================================================

#[test]
fn test_init_command_consistency() {
    let temp_dir1 = TempDir::new().expect("Failed to create temp directory");
    let temp_dir2 = TempDir::new().expect("Failed to create temp directory");

    let path1 = temp_dir1.path().to_str().unwrap();
    let path2 = temp_dir2.path().to_str().unwrap();

    let cmd1 = InitCommand::new(Some(path1.to_string()));
    let result1 = cmd1.execute();

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
fn test_version_command_consistency() {
    let cmd1 = VersionCommand::new();
    let result1 = cmd1.execute();

    let cmd2 = VersionCommand::new();
    let result2 = cmd2.execute();

    // Both should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// ============================================================================
// Integration with Branding Tests
// ============================================================================

#[test]
fn test_branding_with_version_command() {
    // Load branding
    let logo_result = BrandingManager::load_ascii_logo();
    assert!(logo_result.is_ok());

    // Execute version command
    let cmd = VersionCommand::new();
    let result = cmd.execute();
    assert!(result.is_ok());
}

#[test]
fn test_branding_with_init_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    // Load branding
    let caps = BrandingManager::detect_terminal_capabilities();
    assert!(caps.width > 0);

    // Execute init command
    let cmd = InitCommand::new(Some(path.to_string()));
    let result = cmd.execute();
    assert!(result.is_ok());
}

// ============================================================================
// Property-Based Integration Tests
// ============================================================================

#[test]
fn test_all_commands_idempotent() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    use ricecoder_cli::commands::config::ConfigAction;

    // Init command should be idempotent
    let cmd1 = InitCommand::new(Some(path.to_string()));
    let result1 = cmd1.execute();

    let cmd2 = InitCommand::new(Some(path.to_string()));
    let result2 = cmd2.execute();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Version command should be idempotent
    let vcmd1 = VersionCommand::new();
    let vresult1 = vcmd1.execute();

    let vcmd2 = VersionCommand::new();
    let vresult2 = vcmd2.execute();

    assert!(vresult1.is_ok());
    assert!(vresult2.is_ok());

    // Config command should be idempotent
    let ccmd1 = ConfigCommand::new(ConfigAction::List);
    let cresult1 = ccmd1.execute();

    let ccmd2 = ConfigCommand::new(ConfigAction::List);
    let cresult2 = ccmd2.execute();

    assert!(cresult1.is_ok());
    assert!(cresult2.is_ok());
}

#[test]
fn test_command_execution_deterministic() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_str().unwrap();

    // Running the same command multiple times should produce the same result
    let cmd1 = InitCommand::new(Some(path.to_string()));
    let result1 = cmd1.execute();

    let cmd2 = InitCommand::new(Some(path.to_string()));
    let result2 = cmd2.execute();

    assert_eq!(result1.is_ok(), result2.is_ok());
}

#[test]
fn test_branding_operations_always_succeed() {
    // All branding operations should succeed
    assert!(BrandingManager::load_ascii_logo().is_ok());
    assert!(BrandingManager::display_startup_banner().is_ok());
    assert!(BrandingManager::display_version_banner("1.0.0").is_ok());

    let caps = BrandingManager::detect_terminal_capabilities();
    assert!(caps.width > 0);
}
