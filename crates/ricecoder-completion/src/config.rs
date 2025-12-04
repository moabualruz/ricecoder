/// Configuration loading and management for completion engine
use crate::types::*;
use ricecoder_storage::manager::PathResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Completion configuration loader with storage integration
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load completion configuration from a YAML file
    pub fn load_from_yaml(path: &Path) -> CompletionResult<CompletionConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: CompletionConfig = serde_yaml::from_str(&content)?;
        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Load completion configuration from a JSON file
    pub fn load_from_json(path: &Path) -> CompletionResult<CompletionConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: CompletionConfig = serde_json::from_str(&content)?;
        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Load completion configuration from a string
    pub fn load_from_string(content: &str, format: ConfigFormat) -> CompletionResult<CompletionConfig> {
        let config = match format {
            ConfigFormat::Yaml => serde_yaml::from_str(content)?,
            ConfigFormat::Json => serde_json::from_str(content)?,
        };
        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Load completion configuration with hierarchy: Runtime → Project → User → Built-in → Fallback
    ///
    /// # Configuration Hierarchy
    ///
    /// 1. **Runtime**: Configuration passed at runtime (highest priority)
    /// 2. **Project**: Configuration in `.agent/completion/languages/`
    /// 3. **User**: Configuration in `~/.ricecoder/completion/languages/`
    /// 4. **Built-in**: Built-in configurations embedded in ricecoder-storage
    /// 5. **Fallback**: Default configuration for the language (lowest priority)
    pub fn load_with_hierarchy(language: &str) -> CompletionResult<CompletionConfig> {
        // Try project-level configuration first
        let project_path = PathResolver::resolve_project_path();
        let project_completion_path = project_path.join("completion").join("languages");
        
        if let Ok(config) = Self::load_from_directory(&project_completion_path, language) {
            return Ok(config);
        }

        // Try user-level configuration
        let global_path = PathResolver::resolve_global_path()?;
        let user_completion_path = global_path.join("completion").join("languages");
        
        if let Ok(config) = Self::load_from_directory(&user_completion_path, language) {
            return Ok(config);
        }

        // Fall back to default configuration
        Ok(Self::default_for_language(language))
    }

    /// Load configuration from a directory for a specific language
    fn load_from_directory(dir: &Path, language: &str) -> CompletionResult<CompletionConfig> {
        if !dir.is_dir() {
            return Err(CompletionError::ConfigError(format!(
                "Configuration directory not found: {}",
                dir.display()
            )));
        }

        // Try YAML first
        let yaml_path = dir.join(format!("{}.yaml", language));
        if yaml_path.exists() {
            return Self::load_from_yaml(&yaml_path);
        }

        let yml_path = dir.join(format!("{}.yml", language));
        if yml_path.exists() {
            return Self::load_from_yaml(&yml_path);
        }

        // Try JSON
        let json_path = dir.join(format!("{}.json", language));
        if json_path.exists() {
            return Self::load_from_json(&json_path);
        }

        Err(CompletionError::ConfigError(format!(
            "No configuration found for language: {}",
            language
        )))
    }

    /// Get the completion configuration directory path
    pub fn get_completion_config_dir() -> CompletionResult<PathBuf> {
        let global_path = PathResolver::resolve_global_path()?;
        Ok(global_path.join("completion").join("languages"))
    }

    /// Get the project completion configuration directory path
    pub fn get_project_completion_config_dir() -> PathBuf {
        let project_path = PathResolver::resolve_project_path();
        project_path.join("completion").join("languages")
    }

    /// Validate completion configuration
    fn validate_config(config: &CompletionConfig) -> CompletionResult<()> {
        if config.language.is_empty() {
            return Err(CompletionError::ConfigError(
                "Language name cannot be empty".to_string(),
            ));
        }

        if config.ranking_weights.relevance < 0.0
            || config.ranking_weights.frequency < 0.0
            || config.ranking_weights.recency < 0.0
        {
            return Err(CompletionError::ConfigError(
                "Ranking weights must be non-negative".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a default configuration for a language
    pub fn default_for_language(language: &str) -> CompletionConfig {
        CompletionConfig::new(language.to_string())
    }
}

/// Configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Json,
}

/// Language configuration registry with storage integration
pub struct LanguageConfigRegistry {
    configs: HashMap<String, CompletionConfig>,
}

impl LanguageConfigRegistry {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Create a registry and load configurations from storage hierarchy
    pub fn with_hierarchy() -> CompletionResult<Self> {
        let mut registry = Self::new();
        
        // Load from project directory
        let project_dir = ConfigLoader::get_project_completion_config_dir();
        let _ = registry.load_from_directory(&project_dir);
        
        // Load from user directory (overrides project)
        if let Ok(user_dir) = ConfigLoader::get_completion_config_dir() {
            let _ = registry.load_from_directory(&user_dir);
        }
        
        Ok(registry)
    }

    pub fn register(&mut self, config: CompletionConfig) {
        self.configs.insert(config.language.clone(), config);
    }

    pub fn get(&self, language: &str) -> Option<&CompletionConfig> {
        self.configs.get(language)
    }

    pub fn get_mut(&mut self, language: &str) -> Option<&mut CompletionConfig> {
        self.configs.get_mut(language)
    }

    pub fn list_languages(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    pub fn unregister(&mut self, language: &str) -> Option<CompletionConfig> {
        self.configs.remove(language)
    }

    pub fn load_from_directory(&mut self, dir: &Path) -> CompletionResult<()> {
        if !dir.is_dir() {
            return Err(CompletionError::ConfigError(format!(
                "Configuration directory not found: {}",
                dir.display()
            )));
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let config = match ext.to_str() {
                        Some("yaml") | Some("yml") => ConfigLoader::load_from_yaml(&path)?,
                        Some("json") => ConfigLoader::load_from_json(&path)?,
                        _ => continue,
                    };
                    self.register(config);
                }
            }
        }

        Ok(())
    }
}

impl Default for LanguageConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_validate_empty_language() {
        let config = CompletionConfig {
            language: String::new(),
            keywords: Vec::new(),
            snippets: Vec::new(),
            ranking_weights: RankingWeights::default(),
            provider: None,
        };

        let result = ConfigLoader::validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loader_validate_negative_weights() {
        let config = CompletionConfig {
            language: "rust".to_string(),
            keywords: Vec::new(),
            snippets: Vec::new(),
            ranking_weights: RankingWeights {
                relevance: -0.1,
                frequency: 0.3,
                recency: 0.2,
            },
            provider: None,
        };

        let result = ConfigLoader::validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loader_validate_valid() {
        let config = CompletionConfig {
            language: "rust".to_string(),
            keywords: vec!["fn".to_string(), "let".to_string()],
            snippets: Vec::new(),
            ranking_weights: RankingWeights::default(),
            provider: None,
        };

        let result = ConfigLoader::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_language_config_registry_register() {
        let mut registry = LanguageConfigRegistry::new();
        let config = CompletionConfig::new("rust".to_string());
        registry.register(config);

        assert!(registry.get("rust").is_some());
        assert_eq!(registry.list_languages().len(), 1);
    }

    #[test]
    fn test_language_config_registry_unregister() {
        let mut registry = LanguageConfigRegistry::new();
        let config = CompletionConfig::new("rust".to_string());
        registry.register(config);

        let removed = registry.unregister("rust");
        assert!(removed.is_some());
        assert!(registry.get("rust").is_none());
    }

    #[test]
    fn test_config_default_for_language() {
        let config = ConfigLoader::default_for_language("typescript");
        assert_eq!(config.language, "typescript");
        assert!(config.keywords.is_empty());
    }

    #[test]
    fn test_config_loader_hierarchy_fallback() {
        // Test that load_with_hierarchy returns default when no config found
        let config = ConfigLoader::load_with_hierarchy("unknown_language");
        assert!(config.is_ok());
        let cfg = config.unwrap();
        assert_eq!(cfg.language, "unknown_language");
    }

    #[test]
    fn test_language_config_registry_with_hierarchy() {
        // Test that registry can be created with hierarchy
        let registry = LanguageConfigRegistry::with_hierarchy();
        assert!(registry.is_ok());
    }
}
