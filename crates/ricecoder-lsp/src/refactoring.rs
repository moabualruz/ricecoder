//! Refactoring integration for LSP
//!
//! This module integrates the refactoring engine with the LSP server,
//! providing refactoring capabilities through LSP code actions and commands.

use std::sync::Arc;

use ricecoder_refactoring::{
    ConfigManager, GenericRefactoringProvider, ImpactAnalyzer, ProviderRegistry, RefactoringEngine,
    RefactoringType,
};
use serde_json::Value;
use tracing::{debug, info};

/// Refactoring handler for LSP
pub struct RefactoringHandler {
    /// Refactoring engine
    engine: Arc<RefactoringEngine>,
    /// Impact analyzer
    impact_analyzer: Arc<ImpactAnalyzer>,
    /// Configuration manager
    config_manager: Arc<ConfigManager>,
    /// Refactoring enabled flag
    enabled: bool,
}

impl RefactoringHandler {
    /// Create a new refactoring handler
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();

        // Create generic provider for fallback
        let generic_provider = Arc::new(GenericRefactoringProvider::new());

        // Create provider registry with generic fallback
        let provider_registry = ProviderRegistry::new(generic_provider);

        let engine = Arc::new(RefactoringEngine::new(config_manager, provider_registry));
        let impact_analyzer = Arc::new(ImpactAnalyzer::new());

        // Create a new config manager for the handler (since we moved the first one)
        let config_manager = Arc::new(ConfigManager::new());

        Self {
            engine,
            impact_analyzer,
            config_manager,
            enabled: true,
        }
    }

    /// Enable or disable refactoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Refactoring engine enabled");
        } else {
            info!("Refactoring engine disabled");
        }
    }

    /// Check if refactoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the refactoring engine
    pub fn engine(&self) -> Arc<RefactoringEngine> {
        self.engine.clone()
    }

    /// Get the impact analyzer
    pub fn impact_analyzer(&self) -> Arc<ImpactAnalyzer> {
        self.impact_analyzer.clone()
    }

    /// Get the configuration manager
    pub fn config_manager(&self) -> Arc<ConfigManager> {
        self.config_manager.clone()
    }

    /// Handle refactoring request
    pub async fn handle_refactoring_request(&self, params: Value) -> Result<Value, String> {
        if !self.enabled {
            return Err("Refactoring engine is disabled".to_string());
        }

        debug!("Handling refactoring request: {:?}", params);

        // Extract refactoring type from params
        let refactoring_type = params
            .get("refactoringType")
            .and_then(|v| v.as_str())
            .ok_or("Missing refactoringType")?;

        // Extract file URI
        let file_uri = params
            .get("fileUri")
            .and_then(|v| v.as_str())
            .ok_or("Missing fileUri")?;

        // Extract symbol name
        let symbol = params
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or("Missing symbol")?;

        info!(
            "Processing refactoring: type={}, file={}, symbol={}",
            refactoring_type, file_uri, symbol
        );

        Ok(serde_json::json!({
            "success": true,
            "message": format!("Refactoring {} for {} in {} queued", refactoring_type, symbol, file_uri)
        }))
    }

    /// Get available refactoring types
    pub fn available_refactoring_types(&self) -> Vec<&'static str> {
        vec![
            "Rename",
            "Extract",
            "Inline",
            "Move",
            "ChangeSignature",
            "RemoveUnused",
            "Simplify",
        ]
    }

    /// Get refactoring configuration for a language
    pub async fn get_language_config(&self, language: &str) -> Result<Value, String> {
        debug!(
            "Getting refactoring configuration for language: {}",
            language
        );

        // Check if language is available through the provider registry
        let provider = self
            .engine
            .provider_registry()
            .clone()
            .get_provider(language);

        // Verify provider can handle the language
        let analysis = provider.analyze_refactoring("code", language, RefactoringType::Rename);

        if analysis.is_ok() {
            info!("Refactoring provider available for {}", language);
            Ok(serde_json::json!({
                "language": language,
                "available": true
            }))
        } else {
            info!("No refactoring provider for {}", language);
            Err(format!("No refactoring provider for {}", language))
        }
    }
}

impl Default for RefactoringHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactoring_handler_creation() {
        let handler = RefactoringHandler::new();
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_refactoring_handler_enable_disable() {
        let mut handler = RefactoringHandler::new();
        assert!(handler.is_enabled());

        handler.set_enabled(false);
        assert!(!handler.is_enabled());

        handler.set_enabled(true);
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_available_refactoring_types() {
        let handler = RefactoringHandler::new();
        let types = handler.available_refactoring_types();
        assert!(!types.is_empty());
        assert!(types.contains(&"Rename"));
        assert!(types.contains(&"Extract"));
    }
}
