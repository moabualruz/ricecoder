//! Configuration loading from YAML files

use std::path::Path;

use tracing::{debug, info};

use crate::{
    error::{ExternalLspError, Result},
    types::LspServerRegistry,
};

/// Loads LSP server configurations from YAML files
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file
    pub fn load_from_file(path: &Path) -> Result<LspServerRegistry> {
        debug!("Loading LSP configuration from: {:?}", path);

        let content = std::fs::read_to_string(path).map_err(|e| {
            ExternalLspError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        Self::load_from_string(&content)
    }

    /// Load configuration from a YAML string
    pub fn load_from_string(content: &str) -> Result<LspServerRegistry> {
        let registry: LspServerRegistry = serde_yaml::from_str(content)
            .map_err(|e| ExternalLspError::ConfigError(format!("Failed to parse YAML: {}", e)))?;

        // Validate configuration
        Self::validate(&registry)?;

        info!(
            "Successfully loaded LSP configuration with {} languages",
            registry.servers.len()
        );

        Ok(registry)
    }

    /// Validate configuration schema
    fn validate(registry: &LspServerRegistry) -> Result<()> {
        for (language, configs) in &registry.servers {
            if configs.is_empty() {
                return Err(ExternalLspError::InvalidConfiguration(format!(
                    "Language '{}' has no server configurations",
                    language
                )));
            }

            for (idx, config) in configs.iter().enumerate() {
                if config.language != *language {
                    return Err(ExternalLspError::InvalidConfiguration(format!(
                        "Server {} for language '{}' has mismatched language field: '{}'",
                        idx, language, config.language
                    )));
                }

                if config.executable.is_empty() {
                    return Err(ExternalLspError::InvalidConfiguration(format!(
                        "Server {} for language '{}' has empty executable",
                        idx, language
                    )));
                }

                if config.extensions.is_empty() {
                    return Err(ExternalLspError::InvalidConfiguration(format!(
                        "Server {} for language '{}' has no file extensions",
                        idx, language
                    )));
                }

                if config.timeout_ms == 0 {
                    return Err(ExternalLspError::InvalidConfiguration(format!(
                        "Server {} for language '{}' has invalid timeout_ms: 0",
                        idx, language
                    )));
                }
            }
        }

        Ok(())
    }

    /// Merge configurations with hierarchy: Runtime → Project → User → Built-in
    pub fn merge_configs(
        runtime: Option<LspServerRegistry>,
        project: Option<LspServerRegistry>,
        user: Option<LspServerRegistry>,
        builtin: LspServerRegistry,
    ) -> Result<LspServerRegistry> {
        let mut result = builtin;

        // Apply user config
        if let Some(user_config) = user {
            Self::merge_into(&mut result, user_config)?;
        }

        // Apply project config
        if let Some(project_config) = project {
            Self::merge_into(&mut result, project_config)?;
        }

        // Apply runtime config
        if let Some(runtime_config) = runtime {
            Self::merge_into(&mut result, runtime_config)?;
        }

        Ok(result)
    }

    /// Merge one registry into another
    fn merge_into(target: &mut LspServerRegistry, source: LspServerRegistry) -> Result<()> {
        // Merge servers
        for (language, configs) in source.servers {
            target.servers.insert(language, configs);
        }

        // Merge global settings (source overrides target)
        target.global = source.global;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
global:
  max_processes: 5
  default_timeout_ms: 5000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  rust:
    - language: rust
      extensions: [".rs"]
      executable: rust-analyzer
      args: []
      env: {}
      enabled: true
      timeout_ms: 10000
      max_restarts: 3
      idle_timeout_ms: 300000
"#;

        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_ok());

        let registry = result.unwrap();
        assert_eq!(registry.servers.len(), 1);
        assert!(registry.servers.contains_key("rust"));
    }

    #[test]
    fn test_load_invalid_yaml() {
        let yaml = "invalid: [yaml";
        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_executable() {
        let yaml = r#"
global:
  max_processes: 5
  default_timeout_ms: 5000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  rust:
    - language: rust
      extensions: [".rs"]
      executable: ""
      args: []
      env: {}
      enabled: true
      timeout_ms: 10000
      max_restarts: 3
      idle_timeout_ms: 300000
"#;

        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_no_extensions() {
        let yaml = r#"
global:
  max_processes: 5
  default_timeout_ms: 5000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  rust:
    - language: rust
      extensions: []
      executable: rust-analyzer
      args: []
      env: {}
      enabled: true
      timeout_ms: 10000
      max_restarts: 3
      idle_timeout_ms: 300000
"#;

        let result = ConfigLoader::load_from_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_configs() {
        let builtin_yaml = r#"
global:
  max_processes: 5
  default_timeout_ms: 5000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  rust:
    - language: rust
      extensions: [".rs"]
      executable: rust-analyzer
      args: []
      env: {}
      enabled: true
      timeout_ms: 10000
      max_restarts: 3
      idle_timeout_ms: 300000
"#;

        let user_yaml = r#"
global:
  max_processes: 10
  default_timeout_ms: 10000
  enable_fallback: true
  health_check_interval_ms: 30000

servers:
  python:
    - language: python
      extensions: [".py"]
      executable: pylsp
      args: []
      env: {}
      enabled: true
      timeout_ms: 5000
      max_restarts: 3
      idle_timeout_ms: 300000
"#;

        let builtin = ConfigLoader::load_from_string(builtin_yaml).unwrap();
        let user = ConfigLoader::load_from_string(user_yaml).ok();

        let result = ConfigLoader::merge_configs(None, None, user, builtin).unwrap();

        assert_eq!(result.servers.len(), 2);
        assert!(result.servers.contains_key("rust"));
        assert!(result.servers.contains_key("python"));
        assert_eq!(result.global.max_processes, 10);
    }
}
