//! Configuration manager for loading and managing language configurations
//!
//! This module provides utilities for loading language configurations from files,
//! managing provider registries, and supporting hot-reload of configurations.
//! Integrates with ricecoder-storage for cross-platform path resolution and
//! configuration hierarchy management.

use super::types::{ConfigRegistry, LanguageConfig, ConfigResult};
use super::loader::ConfigLoader;
use crate::providers::{
    SemanticAnalyzerRegistry, DiagnosticsRegistry, CodeActionRegistry,
};
use crate::semantic::adapters::{
    RustAnalyzerAdapter, TypeScriptAnalyzerAdapter, PythonAnalyzerAdapter, FallbackAnalyzerAdapter,
};
use crate::diagnostics::adapters::{
    RustDiagnosticsAdapter, TypeScriptDiagnosticsAdapter, PythonDiagnosticsAdapter,
};
use crate::code_actions::adapters::{
    RustCodeActionAdapter, TypeScriptCodeActionAdapter, PythonCodeActionAdapter,
};
use ricecoder_storage::PathResolver;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Configuration manager for language configurations
pub struct ConfigurationManager {
    /// Configuration registry
    config_registry: Arc<RwLock<ConfigRegistry>>,
    /// Semantic analyzer provider registry
    semantic_registry: Arc<RwLock<SemanticAnalyzerRegistry>>,
    /// Diagnostics provider registry
    diagnostics_registry: Arc<RwLock<DiagnosticsRegistry>>,
    /// Code action provider registry
    code_action_registry: Arc<RwLock<CodeActionRegistry>>,
}

impl ConfigurationManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config_registry: Arc::new(RwLock::new(ConfigRegistry::new())),
            semantic_registry: Arc::new(RwLock::new(SemanticAnalyzerRegistry::new())),
            diagnostics_registry: Arc::new(RwLock::new(DiagnosticsRegistry::new())),
            code_action_registry: Arc::new(RwLock::new(CodeActionRegistry::new())),
        }
    }

    /// Load default language configurations
    pub fn load_defaults(&self) -> ConfigResult<()> {
        // Register default semantic analyzers
        {
            let mut registry = self.semantic_registry.write().unwrap();
            registry.register(Box::new(RustAnalyzerAdapter::new()));
            registry.register(Box::new(TypeScriptAnalyzerAdapter::new()));
            registry.register(Box::new(PythonAnalyzerAdapter::new()));
            registry.register(Box::new(FallbackAnalyzerAdapter::new()));
        }

        // Register default diagnostics providers
        {
            let mut registry = self.diagnostics_registry.write().unwrap();
            registry.register(Box::new(RustDiagnosticsAdapter::new()));
            registry.register(Box::new(TypeScriptDiagnosticsAdapter::new()));
            registry.register(Box::new(PythonDiagnosticsAdapter::new()));
        }

        // Register default code action providers
        {
            let mut registry = self.code_action_registry.write().unwrap();
            registry.register(Box::new(RustCodeActionAdapter::new()));
            registry.register(Box::new(TypeScriptCodeActionAdapter::new()));
            registry.register(Box::new(PythonCodeActionAdapter::new()));
        }

        Ok(())
    }

    /// Get the LSP language configuration directory from storage
    ///
    /// Returns paths in priority order:
    /// 1. Runtime configuration (if provided)
    /// 2. Project-level configuration (.agent/lsp/languages/)
    /// 3. User-level configuration (~/.ricecoder/lsp/languages/)
    /// 4. Built-in configuration (from ricecoder-storage)
    pub fn get_language_config_paths() -> ConfigResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Try project-level configuration
        let project_path = PathResolver::resolve_project_path();
        let project_lsp_path = project_path.join("lsp").join("languages");
        if project_lsp_path.exists() {
            paths.push(project_lsp_path);
        }

        // Try user-level configuration
        if let Ok(global_path) = PathResolver::resolve_global_path() {
            let user_lsp_path = global_path.join("lsp").join("languages");
            if user_lsp_path.exists() {
                paths.push(user_lsp_path);
            }
        }

        // Built-in configurations are loaded separately
        Ok(paths)
    }

    /// Load configurations from storage hierarchy
    ///
    /// Loads configurations in priority order:
    /// 1. Project-level (.agent/lsp/languages/)
    /// 2. User-level (~/.ricecoder/lsp/languages/)
    /// 3. Built-in defaults
    pub fn load_from_storage(&self) -> ConfigResult<()> {
        // Load from storage paths
        if let Ok(paths) = Self::get_language_config_paths() {
            for path in paths {
                if path.exists() {
                    self.load_from_directory(&path)?;
                }
            }
        }

        // Always load defaults as fallback
        self.load_defaults()?;

        Ok(())
    }

    /// Load configurations from a directory
    pub fn load_from_directory(&self, path: &std::path::Path) -> ConfigResult<()> {
        let registry = ConfigLoader::load_directory(path)?;

        // Update configuration registry
        {
            let mut config_reg = self.config_registry.write().unwrap();
            for language in registry.languages() {
                if let Some(config) = registry.get(language) {
                    config_reg.register(config.clone())?;
                }
            }
        }

        // Update provider registries with configurations
        self.update_providers_from_configs()?;

        Ok(())
    }

    /// Load a single configuration file
    pub fn load_config_file(&self, path: &std::path::Path) -> ConfigResult<()> {
        let config = ConfigLoader::load(path)?;

        // Update configuration registry
        {
            let mut registry = self.config_registry.write().unwrap();
            registry.register(config.clone())?;
        }

        // Update provider registries with configuration
        self.update_providers_from_configs()?;

        Ok(())
    }

    /// Update provider registries from configurations
    fn update_providers_from_configs(&self) -> ConfigResult<()> {
        let config_reg = self.config_registry.read().unwrap();

        for language in config_reg.languages() {
            if let Some(config) = config_reg.get(language) {
                // Update diagnostics providers with configuration
                {
                    let mut diag_reg = self.diagnostics_registry.write().unwrap();
                    match language {
                        "rust" => {
                            diag_reg.register(Box::new(RustDiagnosticsAdapter::with_config(config.clone())));
                        }
                        "typescript" => {
                            diag_reg.register(Box::new(TypeScriptDiagnosticsAdapter::with_config(config.clone())));
                        }
                        "python" => {
                            diag_reg.register(Box::new(PythonDiagnosticsAdapter::with_config(config.clone())));
                        }
                        _ => {}
                    }
                }

                // Update code action providers with configuration
                {
                    let mut action_reg = self.code_action_registry.write().unwrap();
                    match language {
                        "rust" => {
                            action_reg.register(Box::new(RustCodeActionAdapter::with_config(config.clone())));
                        }
                        "typescript" => {
                            action_reg.register(Box::new(TypeScriptCodeActionAdapter::with_config(config.clone())));
                        }
                        "python" => {
                            action_reg.register(Box::new(PythonCodeActionAdapter::with_config(config.clone())));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the configuration registry
    pub fn config_registry(&self) -> Arc<RwLock<ConfigRegistry>> {
        Arc::clone(&self.config_registry)
    }

    /// Get the semantic analyzer registry
    pub fn semantic_registry(&self) -> Arc<RwLock<SemanticAnalyzerRegistry>> {
        Arc::clone(&self.semantic_registry)
    }

    /// Get the diagnostics registry
    pub fn diagnostics_registry(&self) -> Arc<RwLock<DiagnosticsRegistry>> {
        Arc::clone(&self.diagnostics_registry)
    }

    /// Get the code action registry
    pub fn code_action_registry(&self) -> Arc<RwLock<CodeActionRegistry>> {
        Arc::clone(&self.code_action_registry)
    }

    /// Check if a language is configured
    pub fn has_language(&self, language: &str) -> bool {
        let registry = self.config_registry.read().unwrap();
        registry.has_language(language)
    }

    /// List all configured languages
    pub fn languages(&self) -> Vec<String> {
        let registry = self.config_registry.read().unwrap();
        registry.languages().into_iter().map(|s| s.to_string()).collect()
    }

    /// Get configuration for a language
    pub fn get_config(&self, language: &str) -> Option<LanguageConfig> {
        let registry = self.config_registry.read().unwrap();
        registry.get(language).cloned()
    }
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_manager_creation() {
        let manager = ConfigurationManager::new();
        assert!(manager.config_registry.read().unwrap().languages().is_empty());
    }

    #[test]
    fn test_load_defaults() {
        let manager = ConfigurationManager::new();
        assert!(manager.load_defaults().is_ok());

        // Check that default providers are registered
        let semantic_reg = manager.semantic_registry.read().unwrap();
        assert!(semantic_reg.has_provider("rust"));
        assert!(semantic_reg.has_provider("typescript"));
        assert!(semantic_reg.has_provider("python"));
        assert!(semantic_reg.has_provider("unknown"));
    }

    #[test]
    fn test_has_language() {
        let manager = ConfigurationManager::new();
        manager.load_defaults().unwrap();

        // After loading defaults, no configurations are registered yet
        assert!(!manager.has_language("rust"));
    }

    #[test]
    fn test_languages_list() {
        let manager = ConfigurationManager::new();
        manager.load_defaults().unwrap();

        let languages = manager.languages();
        assert!(languages.is_empty());
    }
}
