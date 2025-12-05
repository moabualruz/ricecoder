//! Configuration loader for hooks
//!
//! Loads hook configurations from YAML files using the ricecoder-storage
//! PathResolver for cross-platform compatibility. Supports configuration
//! hierarchy: Runtime → Project → User → Built-in → Fallback.

use crate::error::{HooksError, Result};
use crate::types::Hook;
use ricecoder_storage::PathResolver;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration loader for hooks
///
/// Loads hooks from YAML configuration files following a hierarchy:
/// 1. Runtime configuration (passed programmatically)
/// 2. Project configuration (.ricecoder/hooks.yaml)
/// 3. User configuration (~/.ricecoder/hooks.yaml)
/// 4. Built-in configuration
/// 5. Fallback (empty configuration)
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load hooks from configuration files
    ///
    /// Attempts to load hooks from the following locations in order:
    /// 1. Project configuration (.ricecoder/hooks.yaml)
    /// 2. User configuration (~/.ricecoder/hooks.yaml)
    /// 3. Built-in configuration
    ///
    /// Returns a map of hook ID to Hook configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files are invalid or cannot be read.
    pub fn load() -> Result<HashMap<String, Hook>> {
        let mut hooks = HashMap::new();

        // Try to load project configuration first
        if let Ok(project_hooks) = Self::load_project_config() {
            hooks.extend(project_hooks);
        }

        // Try to load user configuration
        if let Ok(user_hooks) = Self::load_user_config() {
            // User config overrides project config
            hooks.extend(user_hooks);
        }

        // Try to load built-in configuration
        if let Ok(builtin_hooks) = Self::load_builtin_config() {
            // Built-in config only adds new hooks, doesn't override
            for (id, hook) in builtin_hooks {
                hooks.entry(id).or_insert(hook);
            }
        }

        Ok(hooks)
    }

    /// Load hooks from project configuration
    ///
    /// Looks for `.ricecoder/hooks.yaml` in the current directory.
    fn load_project_config() -> Result<HashMap<String, Hook>> {
        let project_path = PathBuf::from(".ricecoder/hooks.yaml");
        Self::load_from_path(&project_path)
    }

    /// Load hooks from user configuration
    ///
    /// Looks for `~/.ricecoder/hooks.yaml` in the user's home directory.
    fn load_user_config() -> Result<HashMap<String, Hook>> {
        let global_path = PathResolver::resolve_global_path()
            .map_err(|e| HooksError::StorageError(e.to_string()))?;
        let user_config_path = global_path.join("hooks.yaml");
        Self::load_from_path(&user_config_path)
    }

    /// Load hooks from built-in configuration
    ///
    /// Loads built-in hook templates and defaults.
    fn load_builtin_config() -> Result<HashMap<String, Hook>> {
        // For now, return empty map. Built-in templates will be added later.
        Ok(HashMap::new())
    }

    /// Load hooks from a specific file path
    ///
    /// Reads and parses a YAML configuration file containing hook definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist, cannot be read, or contains
    /// invalid YAML or hook configuration.
    fn load_from_path(path: &Path) -> Result<HashMap<String, Hook>> {
        // If file doesn't exist, return empty map (not an error)
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| HooksError::StorageError(format!("Failed to read config file: {}", e)))?;

        Self::parse_yaml(&content)
    }

    /// Parse YAML configuration content
    ///
    /// Parses YAML content and extracts hook definitions.
    ///
    /// Expected YAML format:
    /// ```yaml
    /// hooks:
    ///   - id: hook-id
    ///     name: Hook Name
    ///     event: event_type
    ///     action:
    ///       type: command
    ///       command: echo
    ///       args: ["hello"]
    ///     enabled: true
    /// ```
    fn parse_yaml(content: &str) -> Result<HashMap<String, Hook>> {
        let value: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| HooksError::InvalidConfiguration(format!("Invalid YAML: {}", e)))?;

        let mut hooks = HashMap::new();

        // Extract hooks array from YAML
        if let Some(hooks_array) = value.get("hooks").and_then(|v| v.as_sequence()) {
            for hook_value in hooks_array {
                match serde_yaml::from_value::<Hook>(hook_value.clone()) {
                    Ok(hook) => {
                        hooks.insert(hook.id.clone(), hook);
                    }
                    Err(e) => {
                        return Err(HooksError::InvalidConfiguration(format!(
                            "Failed to parse hook: {}",
                            e
                        )));
                    }
                }
            }
        }

        Ok(hooks)
    }

    /// Load hooks from a YAML string (for testing)
    ///
    /// Parses YAML content and returns hooks.
    #[cfg(test)]
    pub fn load_from_string(content: &str) -> Result<HashMap<String, Hook>> {
        Self::parse_yaml(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_with_command_action() {
        let yaml = r#"
hooks:
  - id: test-hook
    name: Test Hook
    description: A test hook
    event: file_saved
    action:
      type: command
      command: echo
      args:
        - hello
      timeout_ms: 5000
      capture_output: true
    enabled: true
    tags:
      - test
    metadata: {}
"#;

        let hooks = ConfigLoader::load_from_string(yaml).expect("Should parse YAML");
        assert_eq!(hooks.len(), 1);

        let hook = hooks.get("test-hook").expect("Should find hook");
        assert_eq!(hook.name, "Test Hook");
        assert_eq!(hook.event, "file_saved");
        assert!(hook.enabled);
        assert_eq!(hook.tags.len(), 1);
    }

    #[test]
    fn test_parse_yaml_multiple_hooks() {
        let yaml = r#"
hooks:
  - id: hook1
    name: Hook 1
    event: event1
    action:
      type: command
      command: cmd1
      args: []
      timeout_ms: null
      capture_output: false
    enabled: true
    tags: []
    metadata: {}
  - id: hook2
    name: Hook 2
    event: event2
    action:
      type: command
      command: cmd2
      args: []
      timeout_ms: null
      capture_output: false
    enabled: false
    tags: []
    metadata: {}
"#;

        let hooks = ConfigLoader::load_from_string(yaml).expect("Should parse YAML");
        assert_eq!(hooks.len(), 2);
        assert!(hooks.contains_key("hook1"));
        assert!(hooks.contains_key("hook2"));
    }

    #[test]
    fn test_parse_yaml_empty() {
        let yaml = "hooks: []";
        let hooks = ConfigLoader::load_from_string(yaml).expect("Should parse empty YAML");
        assert_eq!(hooks.len(), 0);
    }

    #[test]
    fn test_parse_yaml_invalid() {
        let yaml = "invalid: yaml: content:";
        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_missing_required_field() {
        let yaml = r#"
hooks:
  - id: test-hook
    name: Test Hook
"#;
        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_nonexistent_path() {
        let path = PathBuf::from("/nonexistent/path/hooks.yaml");
        let hooks = ConfigLoader::load_from_path(&path).expect("Should return empty map");
        assert_eq!(hooks.len(), 0);
    }
}
