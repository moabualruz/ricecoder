// Unit tests for core commands
// **Feature: ricecoder-cli, Tests for Requirements 2.1-2.6**

use std::{fs, path::Path};

use ricecoder_cli::commands::{Command, ConfigCommand, GenCommand, InitCommand};
use tempfile::TempDir;

// ============================================================================
// InitCommand Tests
// ============================================================================

#[test]
fn test_init_command_creates_agent_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(temp_path.to_string()));
    let result = cmd.execute();

    assert!(result.is_ok(), "Init command should succeed");

    // Verify .agent directory was created
    let agent_dir = Path::new(temp_path).join(".agent");
    assert!(
        agent_dir.exists(),
        "Expected .agent directory to be created at {}",
        agent_dir.display()
    );
    assert!(
        agent_dir.is_dir(),
        "Expected .agent to be a directory, not a file"
    );
}

#[test]
fn test_init_command_creates_config_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(temp_path.to_string()));
    let result = cmd.execute();

    assert!(result.is_ok(), "Init command should succeed");

    // Verify ricecoder.toml was created
    let config_file = Path::new(temp_path).join(".agent/ricecoder.toml");
    assert!(
        config_file.exists(),
        "Expected ricecoder.toml to be created at {}",
        config_file.display()
    );
    assert!(
        config_file.is_file(),
        "Expected ricecoder.toml to be a file"
    );
}

#[test]
fn test_init_command_config_contains_required_sections() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(temp_path.to_string()));
    let _ = cmd.execute();

    // Read and verify config content
    let config_file = Path::new(temp_path).join(".agent/ricecoder.toml");
    let content = fs::read_to_string(&config_file).expect("Failed to read ricecoder.toml");

    // Verify required sections exist
    assert!(
        content.contains("[project]"),
        "Config should contain [project] section"
    );
    assert!(
        content.contains("[providers]"),
        "Config should contain [providers] section"
    );
    assert!(
        content.contains("[storage]"),
        "Config should contain [storage] section"
    );
}

#[test]
fn test_init_command_uses_current_directory_by_default() {
    // This test verifies the behavior when no path is provided
    let cmd = InitCommand::new(None);

    // The command should use "." as the default path
    // We can't easily test this without modifying the current directory,
    // so we just verify the command can be created with None
    assert_eq!(cmd.project_path, None);
}

#[test]
fn test_init_command_accepts_custom_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(temp_path.to_string()));
    assert_eq!(cmd.project_path, Some(temp_path.to_string()));
}

// ============================================================================
// GenCommand Tests
// ============================================================================

#[test]
fn test_gen_command_validates_spec_file_exists() {
    let cmd = GenCommand::new("nonexistent_spec.md".to_string());
    let result = cmd.execute();

    assert!(
        result.is_err(),
        "Gen command should fail when spec file doesn't exist"
    );
}

#[test]
fn test_gen_command_accepts_valid_spec_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("test_spec.md");

    // Create a valid spec file
    fs::write(&spec_file, "# Test Specification\n\nThis is a test spec.")
        .expect("Failed to write spec file");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    assert!(
        result.is_ok(),
        "Gen command should succeed with valid spec file"
    );
}

#[test]
fn test_gen_command_rejects_empty_spec_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("empty_spec.md");

    // Create an empty spec file
    fs::write(&spec_file, "").expect("Failed to write spec file");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    assert!(
        result.is_err(),
        "Gen command should fail with empty spec file"
    );
}

#[test]
fn test_gen_command_rejects_directory_as_spec() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = GenCommand::new(temp_path.to_string());
    let result = cmd.execute();

    assert!(
        result.is_err(),
        "Gen command should fail when spec path is a directory"
    );
}

#[test]
fn test_gen_command_loads_spec_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("test_spec.md");
    let spec_content = "# Test Specification\n\nGenerate code from this spec.";

    fs::write(&spec_file, spec_content).expect("Failed to write spec file");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    // The command should succeed and load the spec
    assert!(result.is_ok(), "Gen command should load spec successfully");
}

#[test]
fn test_gen_command_with_whitespace_only_spec() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("whitespace_spec.md");

    // Create a spec file with only whitespace
    fs::write(&spec_file, "   \n\n  \t  \n").expect("Failed to write spec file");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());
    let result = cmd.execute();

    assert!(
        result.is_err(),
        "Gen command should fail with whitespace-only spec file"
    );
}

// ============================================================================
// ConfigCommand Tests
// ============================================================================

#[test]
fn test_config_command_list_action() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::List);
    let result = cmd.execute();

    // List action should succeed
    assert!(result.is_ok(), "Config list should succeed");
}

#[test]
fn test_config_command_get_existing_key() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::Get("provider.default".to_string()));
    let result = cmd.execute();

    // Get action for existing key should succeed
    assert!(result.is_ok(), "Config get for existing key should succeed");
}

#[test]
fn test_config_command_get_nonexistent_key() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::Get("nonexistent.key".to_string()));
    let result = cmd.execute();

    // Get action for non-existent key should still succeed (with warning)
    assert!(
        result.is_ok(),
        "Config get for non-existent key should succeed with warning"
    );
}

#[test]
fn test_config_command_set_value() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::Set(
        "test.key".to_string(),
        "test_value".to_string(),
    ));
    let result = cmd.execute();

    // Set action should succeed
    assert!(result.is_ok(), "Config set should succeed");
}

#[test]
fn test_config_command_set_with_empty_value() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::Set("test.key".to_string(), "".to_string()));
    let result = cmd.execute();

    // Set action with empty value should still succeed
    assert!(result.is_ok(), "Config set with empty value should succeed");
}

#[test]
fn test_config_command_set_with_special_characters() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::Set(
        "test.key".to_string(),
        "value with spaces and special chars: !@#$%".to_string(),
    ));
    let result = cmd.execute();

    // Set action with special characters should succeed
    assert!(
        result.is_ok(),
        "Config set with special characters should succeed"
    );
}

// ============================================================================
// Integration Tests for Command Trait
// ============================================================================

#[test]
fn test_init_command_implements_command_trait() {
    let cmd = InitCommand::new(None);

    // Verify InitCommand implements Command trait
    let _: &dyn Command = &cmd;
}

#[test]
fn test_gen_command_implements_command_trait() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let spec_file = temp_dir.path().join("test.md");
    fs::write(&spec_file, "# Test").expect("Failed to write file");

    let cmd = GenCommand::new(spec_file.to_str().unwrap().to_string());

    // Verify GenCommand implements Command trait
    let _: &dyn Command = &cmd;
}

#[test]
fn test_config_command_implements_command_trait() {
    use ricecoder_cli::commands::config::ConfigAction;

    let cmd = ConfigCommand::new(ConfigAction::List);

    // Verify ConfigCommand implements Command trait
    let _: &dyn Command = &cmd;
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn test_init_command_idempotent() {
    // Running init twice should not cause errors
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    let cmd = InitCommand::new(Some(temp_path.to_string()));

    // First execution
    let result1 = cmd.execute();
    assert!(result1.is_ok(), "First init should succeed");

    // Second execution (should also succeed or handle gracefully)
    let result2 = cmd.execute();
    assert!(result2.is_ok(), "Second init should succeed (idempotent)");
}

#[test]
fn test_gen_command_with_various_spec_formats() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test with markdown spec
    let md_spec = temp_dir.path().join("spec.md");
    fs::write(&md_spec, "# Spec\n\nContent").expect("Failed to write");
    let cmd = GenCommand::new(md_spec.to_str().unwrap().to_string());
    assert!(cmd.execute().is_ok(), "Should handle markdown spec");

    // Test with yaml spec
    let yaml_spec = temp_dir.path().join("spec.yaml");
    fs::write(&yaml_spec, "name: test\ndescription: test spec").expect("Failed to write");
    let cmd = GenCommand::new(yaml_spec.to_str().unwrap().to_string());
    assert!(cmd.execute().is_ok(), "Should handle yaml spec");

    // Test with json spec
    let json_spec = temp_dir.path().join("spec.json");
    fs::write(&json_spec, r#"{"name": "test"}"#).expect("Failed to write");
    let cmd = GenCommand::new(json_spec.to_str().unwrap().to_string());
    assert!(cmd.execute().is_ok(), "Should handle json spec");
}

#[test]
fn test_config_command_all_actions_succeed() {
    use ricecoder_cli::commands::config::ConfigAction;

    // All config actions should succeed without panicking
    let actions = vec![
        ConfigAction::List,
        ConfigAction::Get("provider.default".to_string()),
        ConfigAction::Set("test.key".to_string(), "value".to_string()),
    ];

    for action in actions {
        let cmd = ConfigCommand::new(action);
        let result = cmd.execute();
        assert!(result.is_ok(), "Config action should succeed");
    }
}
