//! Integration tests for markdown configuration module
//!
//! Tests end-to-end workflows for loading and registering configurations

use ricecoder_storage::markdown_config::{
    loader::{ConfigFileType, ConfigurationLoader},
    registry::ConfigRegistry,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

// ============ Helper Functions ============

fn create_agent_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
    let path = dir.join(format!("{}.agent.md", name));
    fs::write(&path, content).unwrap();
    path
}

fn create_mode_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
    let path = dir.join(format!("{}.mode.md", name));
    fs::write(&path, content).unwrap();
    path
}

fn create_command_file(dir: &PathBuf, name: &str, content: &str) -> PathBuf {
    let path = dir.join(format!("{}.command.md", name));
    fs::write(&path, content).unwrap();
    path
}

// ============ End-to-End Agent Configuration Loading ============

#[tokio::test]
async fn test_end_to_end_agent_configuration_loading() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create test agent files
    let agent1_content = r#"---
name: code-review-agent
description: Reviews code for quality
model: gpt-4
temperature: 0.7
max_tokens: 2000
tools:
  - code-analyzer
  - linter
---
You are an expert code reviewer. Analyze the provided code and provide constructive feedback."#;

    let agent2_content = r#"---
name: documentation-agent
description: Generates documentation
model: gpt-3.5-turbo
temperature: 0.5
---
You are a documentation expert. Generate clear and comprehensive documentation."#;

    create_agent_file(&dir_path, "code-review", agent1_content);
    create_agent_file(&dir_path, "documentation", agent2_content);

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load all configurations
    let (success, errors, error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify loading succeeded
    assert_eq!(success, 2, "Should load 2 agents");
    assert_eq!(errors, 0, "Should have no errors");
    assert_eq!(error_list.len(), 0, "Should have no error details");

    // Verify agents are registered
    let code_review = registry
        .get_agent("code-review-agent")
        .unwrap()
        .expect("code-review-agent should be registered");
    assert_eq!(code_review.name, "code-review-agent");
    assert_eq!(code_review.model, Some("gpt-4".to_string()));
    assert_eq!(code_review.temperature, Some(0.7));
    assert_eq!(code_review.tools.len(), 2);

    let documentation = registry
        .get_agent("documentation-agent")
        .unwrap()
        .expect("documentation-agent should be registered");
    assert_eq!(documentation.name, "documentation-agent");
    assert_eq!(documentation.model, Some("gpt-3.5-turbo".to_string()));
}

// ============ End-to-End Mode Configuration Loading ============

#[tokio::test]
async fn test_end_to_end_mode_configuration_loading() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create test mode files
    let focus_mode_content = r#"---
name: focus-mode
description: Focus mode for deep work
keybinding: C-f
enabled: true
---
Enter focus mode to minimize distractions and maximize productivity."#;

    let debug_mode_content = r#"---
name: debug-mode
description: Debug mode for troubleshooting
keybinding: C-d
enabled: true
---
Enter debug mode to enable detailed logging and diagnostics."#;

    create_mode_file(&dir_path, "focus", focus_mode_content);
    create_mode_file(&dir_path, "debug", debug_mode_content);

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load all configurations
    let (success, errors, error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify loading succeeded
    assert_eq!(success, 2, "Should load 2 modes");
    assert_eq!(errors, 0, "Should have no errors");
    assert_eq!(error_list.len(), 0, "Should have no error details");

    // Verify modes are registered
    let focus = registry
        .get_mode("focus-mode")
        .unwrap()
        .expect("focus-mode should be registered");
    assert_eq!(focus.name, "focus-mode");
    assert_eq!(focus.keybinding, Some("C-f".to_string()));
    assert!(focus.enabled);

    let debug = registry
        .get_mode("debug-mode")
        .unwrap()
        .expect("debug-mode should be registered");
    assert_eq!(debug.name, "debug-mode");
    assert_eq!(debug.keybinding, Some("C-d".to_string()));
}

// ============ End-to-End Command Configuration Loading ============

#[tokio::test]
async fn test_end_to_end_command_configuration_loading() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create test command files
    let test_command_content = r#"---
name: run-tests
description: Run test suite
keybinding: C-t
parameters:
  - name: test-type
    description: Type of tests to run
    required: false
    default: all
---
cargo test {{test-type}}"#;

    let build_command_content = r#"---
name: build-project
description: Build the project
keybinding: C-b
---
cargo build --release"#;

    create_command_file(&dir_path, "test", test_command_content);
    create_command_file(&dir_path, "build", build_command_content);

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load all configurations
    let (success, errors, error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify loading succeeded
    assert_eq!(success, 2, "Should load 2 commands");
    assert_eq!(errors, 0, "Should have no errors");
    assert_eq!(error_list.len(), 0, "Should have no error details");

    // Verify commands are registered
    let test_cmd = registry
        .get_command("run-tests")
        .unwrap()
        .expect("run-tests should be registered");
    assert_eq!(test_cmd.name, "run-tests");
    assert_eq!(test_cmd.parameters.len(), 1);
    assert_eq!(test_cmd.parameters[0].name, "test-type");

    let build_cmd = registry
        .get_command("build-project")
        .unwrap()
        .expect("build-project should be registered");
    assert_eq!(build_cmd.name, "build-project");
    assert_eq!(build_cmd.template, "cargo build --release");
}

// ============ Mixed Configuration Loading ============

#[tokio::test]
async fn test_mixed_configuration_loading() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create mixed configuration files
    create_agent_file(
        &dir_path,
        "agent1",
        r#"---
name: agent1
---
Agent prompt"#,
    );

    create_mode_file(
        &dir_path,
        "mode1",
        r#"---
name: mode1
---
Mode prompt"#,
    );

    create_command_file(
        &dir_path,
        "cmd1",
        r#"---
name: cmd1
---
Command template"#,
    );

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load all configurations
    let (success, errors, _error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify loading succeeded
    assert_eq!(success, 3, "Should load 3 configurations");
    assert_eq!(errors, 0, "Should have no errors");

    // Verify all are registered
    assert!(registry.has_agent("agent1").unwrap());
    assert!(registry.has_mode("mode1").unwrap());
    assert!(registry.has_command("cmd1").unwrap());
}

// ============ Error Handling ============

#[tokio::test]
async fn test_partial_loading_with_errors() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create valid configuration
    create_agent_file(
        &dir_path,
        "valid",
        r#"---
name: valid-agent
---
Valid agent"#,
    );

    // Create invalid configuration (missing frontmatter)
    fs::write(dir_path.join("invalid.agent.md"), "# No frontmatter\nJust markdown").unwrap();

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load all configurations
    let (success, errors, error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify partial loading
    assert_eq!(success, 1, "Should load 1 valid configuration");
    assert_eq!(errors, 1, "Should have 1 error");
    assert_eq!(error_list.len(), 1, "Should have 1 error detail");

    // Verify valid configuration is registered
    assert!(registry.has_agent("valid-agent").unwrap());
}

// ============ Configuration Discovery ============

#[tokio::test]
async fn test_configuration_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create various files
    create_agent_file(&dir_path, "agent1", "---\nname: agent1\n---\nTest");
    create_mode_file(&dir_path, "mode1", "---\nname: mode1\n---\nTest");
    create_command_file(&dir_path, "cmd1", "---\nname: cmd1\n---\nTest");

    // Create non-configuration files
    fs::write(dir_path.join("readme.md"), "# README").unwrap();
    fs::write(dir_path.join("other.txt"), "Other file").unwrap();

    // Create loader
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry));

    // Discover configurations
    let discovered = loader.discover(&[dir_path]).unwrap();

    // Verify discovery
    assert_eq!(discovered.len(), 3, "Should discover 3 configuration files");
    assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Agent));
    assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Mode));
    assert!(discovered.iter().any(|f| f.config_type == ConfigFileType::Command));
}

// ============ Multiple Directories ============

#[tokio::test]
async fn test_loading_from_multiple_directories() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let dir1 = temp_dir1.path().to_path_buf();
    let dir2 = temp_dir2.path().to_path_buf();

    // Create configurations in first directory
    create_agent_file(&dir1, "agent1", "---\nname: agent1\n---\nTest");

    // Create configurations in second directory
    create_agent_file(&dir2, "agent2", "---\nname: agent2\n---\nTest");

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load from both directories
    let (success, errors, _) = loader.load_all(&[dir1, dir2]).await.unwrap();

    // Verify loading from both directories
    assert_eq!(success, 2, "Should load from both directories");
    assert_eq!(errors, 0, "Should have no errors");

    // Verify both are registered
    assert!(registry.has_agent("agent1").unwrap());
    assert!(registry.has_agent("agent2").unwrap());
}

// ============ Configuration Isolation ============

#[tokio::test]
async fn test_configuration_isolation() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create configurations
    create_agent_file(&dir_path, "agent1", "---\nname: agent1\n---\nAgent 1");
    create_agent_file(&dir_path, "agent2", "---\nname: agent2\n---\nAgent 2");

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry.clone()));

    // Load configurations
    loader.load_all(&[dir_path]).await.unwrap();

    // Verify isolation
    let agent1 = registry
        .get_agent("agent1")
        .unwrap()
        .expect("agent1 should exist");
    let agent2 = registry
        .get_agent("agent2")
        .unwrap()
        .expect("agent2 should exist");

    // Agents should be independent
    assert_ne!(agent1.name, agent2.name);
    assert_ne!(agent1.prompt, agent2.prompt);
}

// ============ Empty Directory Handling ============

#[tokio::test]
async fn test_empty_directory_handling() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry));

    // Load from empty directory
    let (success, errors, _error_list) = loader.load_all(&[dir_path]).await.unwrap();

    // Verify handling of empty directory
    assert_eq!(success, 0, "Should load 0 configurations");
    assert_eq!(errors, 0, "Should have no errors");
}

// ============ Nonexistent Directory Handling ============

#[tokio::test]
async fn test_nonexistent_directory_handling() {
    let nonexistent = PathBuf::from("/nonexistent/path/that/does/not/exist");

    // Create loader and registry
    let registry = Arc::new(ConfigRegistry::new());
    let loader = Arc::new(ConfigurationLoader::new(registry));

    // Load from nonexistent directory
    let (success, errors, _) = loader.load_all(&[nonexistent]).await.unwrap();

    // Verify handling of nonexistent directory
    assert_eq!(success, 0, "Should load 0 configurations");
    assert_eq!(errors, 0, "Should have no errors");
}
