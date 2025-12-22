//! Property-based tests for storage modes
//!
//! **Feature: ricecoder-storage, Property 6: Project-Isolated Mode Isolation**
//! **Feature: ricecoder-storage, Property 7: Global-Only Mode Isolation**
//! **Validates: Requirements 3.1, 3.2**

use std::fs;

use ricecoder_storage::{config::StorageModeHandler, types::StorageMode};
use tempfile::TempDir;

#[test]
fn test_global_only_loads_global() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Create global config
    let global_config = r#"
providers:
  default_provider: openai
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(global_dir.path().join("config.yaml"), global_config)
        .expect("Failed to write global config");

    // Create project config
    let project_config = r#"
providers:
  default_provider: anthropic
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(project_dir.path().join("config.yaml"), project_config)
        .expect("Failed to write project config");

    let config = StorageModeHandler::load_for_mode(
        StorageMode::GlobalOnly,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should have global provider, not project provider
    assert_eq!(
        config.providers.default_provider,
        Some("openai".to_string())
    );
    assert_ne!(
        config.providers.default_provider,
        Some("anthropic".to_string())
    );
}

#[test]
fn test_project_only_loads_project() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Create global config
    let global_config = r#"
providers:
  default_provider: openai
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(global_dir.path().join("config.yaml"), global_config)
        .expect("Failed to write global config");

    // Create project config
    let project_config = r#"
providers:
  default_provider: anthropic
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(project_dir.path().join("config.yaml"), project_config)
        .expect("Failed to write project config");

    let config = StorageModeHandler::load_for_mode(
        StorageMode::ProjectOnly,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should have project provider, not global provider
    assert_eq!(
        config.providers.default_provider,
        Some("anthropic".to_string())
    );
    assert_ne!(
        config.providers.default_provider,
        Some("openai".to_string())
    );
}

#[test]
fn test_merged_mode_combines_configs() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Create global config
    let global_config = r#"
providers:
  default_provider: openai
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(global_dir.path().join("config.yaml"), global_config)
        .expect("Failed to write global config");

    // Create project config
    let project_config = r#"
providers:
  default_provider: anthropic
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(project_dir.path().join("config.yaml"), project_config)
        .expect("Failed to write project config");

    let config = StorageModeHandler::load_for_mode(
        StorageMode::Merged,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should have project provider (project overrides global)
    assert_eq!(
        config.providers.default_provider,
        Some("anthropic".to_string())
    );
}

#[test]
fn test_global_only_ignores_missing_project() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Create only global config
    let global_config = r#"
providers:
  default_provider: openai
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(global_dir.path().join("config.yaml"), global_config)
        .expect("Failed to write global config");

    let config = StorageModeHandler::load_for_mode(
        StorageMode::GlobalOnly,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should have global provider
    assert_eq!(
        config.providers.default_provider,
        Some("openai".to_string())
    );
}

#[test]
fn test_project_only_ignores_missing_global() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Create only project config
    let project_config = r#"
providers:
  default_provider: anthropic
  api_keys: {}
  endpoints: {}
defaults: {}
"#;
    fs::write(project_dir.path().join("config.yaml"), project_config)
        .expect("Failed to write project config");

    let config = StorageModeHandler::load_for_mode(
        StorageMode::ProjectOnly,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should have project provider
    assert_eq!(
        config.providers.default_provider,
        Some("anthropic".to_string())
    );
}

#[test]
fn test_merged_mode_handles_missing_configs() {
    let global_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = TempDir::new().expect("Failed to create temp dir");

    // Don't create any configs

    let config = StorageModeHandler::load_for_mode(
        StorageMode::Merged,
        Some(global_dir.path()),
        Some(project_dir.path()),
    )
    .expect("Failed to load config");

    // Should return defaults
    assert_eq!(config.providers.default_provider, None);
}
