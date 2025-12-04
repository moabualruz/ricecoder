//! Generic code actions engine that works with pluggable providers
//!
//! This module provides a language-agnostic code actions engine that delegates
//! to language-specific providers registered in the provider registry.

use crate::providers::{CodeActionProvider, CodeActionRegistry, ProviderResult};
use crate::types::Diagnostic;

/// Generic code actions engine that delegates to pluggable providers
pub struct GenericCodeActionsEngine {
    registry: CodeActionRegistry,
}

impl GenericCodeActionsEngine {
    /// Create a new generic code actions engine
    pub fn new() -> Self {
        Self {
            registry: CodeActionRegistry::new(),
        }
    }

    /// Register a code action provider
    pub fn register_provider(&mut self, provider: Box<dyn CodeActionProvider>) {
        self.registry.register(provider);
    }

    /// Get the provider registry
    pub fn registry(&self) -> &CodeActionRegistry {
        &self.registry
    }

    /// Get a mutable reference to the provider registry
    pub fn registry_mut(&mut self) -> &mut CodeActionRegistry {
        &mut self.registry
    }

    /// Suggest code actions using the appropriate provider or fallback
    pub fn suggest_actions(&self, diagnostic: &Diagnostic, code: &str, language: &str) -> ProviderResult<Vec<String>> {
        if let Some(provider) = self.registry.get(language) {
            provider.suggest_actions(diagnostic, code)
        } else {
            // Gracefully degrade to empty actions for unconfigured languages
            tracing::debug!("No code action provider found for language '{}', returning empty", language);
            Ok(Vec::new())
        }
    }

    /// Apply a code action using the appropriate provider or fallback
    pub fn apply_action(&self, code: &str, action: &str, language: &str) -> ProviderResult<String> {
        if let Some(provider) = self.registry.get(language) {
            provider.apply_action(code, action)
        } else {
            // Gracefully degrade to returning original code for unconfigured languages
            tracing::debug!("No code action provider found for language '{}', returning original code", language);
            Ok(code.to_string())
        }
    }

    /// Check if a provider is registered for a language
    pub fn has_provider(&self, language: &str) -> bool {
        self.registry.has_provider(language)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.registry.languages()
    }
}

impl Default for GenericCodeActionsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCodeActionProvider;

    impl CodeActionProvider for MockCodeActionProvider {
        fn language(&self) -> &str {
            "mock"
        }

        fn suggest_actions(&self, _diagnostic: &Diagnostic, _code: &str) -> ProviderResult<Vec<String>> {
            Ok(vec!["action1".to_string()])
        }

        fn apply_action(&self, code: &str, _action: &str) -> ProviderResult<String> {
            Ok(code.to_string())
        }

        fn config(&self) -> Option<&crate::config::LanguageConfig> {
            None
        }
    }

    #[test]
    fn test_generic_code_actions_engine_with_provider() {
        let mut engine = GenericCodeActionsEngine::new();
        engine.register_provider(Box::new(MockCodeActionProvider));

        assert!(engine.has_provider("mock"));
    }

    #[test]
    fn test_generic_code_actions_engine_fallback() {
        use crate::types::{Position, Range, DiagnosticSeverity};

        let engine = GenericCodeActionsEngine::new();

        // Should gracefully degrade for unknown language
        assert!(!engine.has_provider("unknown"));
        let diagnostic = Diagnostic::new(
            Range::new(Position::new(0, 0), Position::new(0, 5)),
            DiagnosticSeverity::Error,
            "test".to_string(),
        );
        let result = engine.suggest_actions(&diagnostic, "test", "unknown");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_generic_code_actions_engine_languages() {
        let mut engine = GenericCodeActionsEngine::new();
        engine.register_provider(Box::new(MockCodeActionProvider));

        let languages = engine.languages();
        assert!(languages.contains(&"mock"));
    }
}
