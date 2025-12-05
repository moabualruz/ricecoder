//! Generic semantic analyzer that works with pluggable providers
//!
//! This module provides a language-agnostic semantic analyzer that delegates
//! to language-specific providers registered in the provider registry.

use crate::providers::{ProviderResult, SemanticAnalyzerProvider, SemanticAnalyzerRegistry};
use crate::semantic::{fallback_analyzer::FallbackAnalyzer, SemanticAnalyzer};
use crate::types::{Position, SemanticInfo, Symbol};

/// Generic semantic analyzer that delegates to pluggable providers
pub struct GenericSemanticAnalyzer {
    registry: SemanticAnalyzerRegistry,
    fallback: FallbackAnalyzer,
}

impl GenericSemanticAnalyzer {
    /// Create a new generic semantic analyzer
    pub fn new() -> Self {
        Self {
            registry: SemanticAnalyzerRegistry::new(),
            fallback: FallbackAnalyzer::new(),
        }
    }

    /// Register a semantic analyzer provider
    pub fn register_provider(&mut self, provider: Box<dyn SemanticAnalyzerProvider>) {
        self.registry.register(provider);
    }

    /// Get the provider registry
    pub fn registry(&self) -> &SemanticAnalyzerRegistry {
        &self.registry
    }

    /// Get a mutable reference to the provider registry
    pub fn registry_mut(&mut self) -> &mut SemanticAnalyzerRegistry {
        &mut self.registry
    }

    /// Analyze code using the appropriate provider or fallback
    pub fn analyze(&self, code: &str, language: &str) -> ProviderResult<SemanticInfo> {
        if let Some(provider) = self.registry.get(language) {
            provider.analyze(code)
        } else {
            // Gracefully degrade to fallback analysis
            tracing::debug!(
                "No provider found for language '{}', using fallback",
                language
            );
            self.fallback
                .analyze(code)
                .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
        }
    }

    /// Extract symbols using the appropriate provider or fallback
    pub fn extract_symbols(&self, code: &str, language: &str) -> ProviderResult<Vec<Symbol>> {
        if let Some(provider) = self.registry.get(language) {
            provider.extract_symbols(code)
        } else {
            // Gracefully degrade to fallback analysis
            tracing::debug!(
                "No provider found for language '{}', using fallback",
                language
            );
            self.fallback
                .extract_symbols(code)
                .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
        }
    }

    /// Get hover information using the appropriate provider or fallback
    pub fn get_hover_info(
        &self,
        code: &str,
        language: &str,
        position: Position,
    ) -> ProviderResult<Option<String>> {
        if let Some(provider) = self.registry.get(language) {
            provider.get_hover_info(code, position)
        } else {
            // Gracefully degrade to fallback analysis
            tracing::debug!(
                "No provider found for language '{}', using fallback",
                language
            );
            self.fallback
                .get_hover_info(code, position)
                .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
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

impl Default for GenericSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;

    impl SemanticAnalyzerProvider for MockProvider {
        fn language(&self) -> &str {
            "mock"
        }

        fn analyze(&self, _code: &str) -> ProviderResult<SemanticInfo> {
            Ok(SemanticInfo {
                symbols: vec![],
                imports: vec![],
                definitions: vec![],
                references: vec![],
            })
        }

        fn extract_symbols(&self, _code: &str) -> ProviderResult<Vec<Symbol>> {
            Ok(vec![])
        }

        fn get_hover_info(
            &self,
            _code: &str,
            _position: Position,
        ) -> ProviderResult<Option<String>> {
            Ok(Some("mock info".to_string()))
        }
    }

    #[test]
    fn test_generic_analyzer_with_provider() {
        let mut analyzer = GenericSemanticAnalyzer::new();
        analyzer.register_provider(Box::new(MockProvider));

        assert!(analyzer.has_provider("mock"));
        assert!(analyzer.analyze("test", "mock").is_ok());
    }

    #[test]
    fn test_generic_analyzer_fallback() {
        let analyzer = GenericSemanticAnalyzer::new();

        // Should use fallback for unknown language
        assert!(!analyzer.has_provider("unknown"));
        assert!(analyzer.analyze("test", "unknown").is_ok());
    }

    #[test]
    fn test_generic_analyzer_languages() {
        let mut analyzer = GenericSemanticAnalyzer::new();
        analyzer.register_provider(Box::new(MockProvider));

        let languages = analyzer.languages();
        assert!(languages.contains(&"mock"));
    }
}
