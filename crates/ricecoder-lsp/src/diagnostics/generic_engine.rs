//! Generic diagnostics engine that works with pluggable providers
//!
//! This module provides a language-agnostic diagnostics engine that delegates
//! to language-specific providers registered in the provider registry.

use crate::{
    providers::{DiagnosticsProvider, DiagnosticsRegistry, ProviderResult},
    types::{Diagnostic, Range},
};

/// Generic diagnostics engine that delegates to pluggable providers
pub struct GenericDiagnosticsEngine {
    registry: DiagnosticsRegistry,
}

impl GenericDiagnosticsEngine {
    /// Create a new generic diagnostics engine
    pub fn new() -> Self {
        Self {
            registry: DiagnosticsRegistry::new(),
        }
    }

    /// Register a diagnostics provider
    pub fn register_provider(&mut self, provider: Box<dyn DiagnosticsProvider>) {
        self.registry.register(provider);
    }

    /// Get the provider registry
    pub fn registry(&self) -> &DiagnosticsRegistry {
        &self.registry
    }

    /// Get a mutable reference to the provider registry
    pub fn registry_mut(&mut self) -> &mut DiagnosticsRegistry {
        &mut self.registry
    }

    /// Generate diagnostics using the appropriate provider or fallback
    pub fn generate_diagnostics(
        &self,
        code: &str,
        language: &str,
    ) -> ProviderResult<Vec<Diagnostic>> {
        if let Some(provider) = self.registry.get(language) {
            provider.generate_diagnostics(code)
        } else {
            // Gracefully degrade to empty diagnostics for unconfigured languages
            tracing::debug!(
                "No diagnostics provider found for language '{}', returning empty",
                language
            );
            Ok(Vec::new())
        }
    }

    /// Generate diagnostics for a specific range
    pub fn generate_diagnostics_for_range(
        &self,
        code: &str,
        language: &str,
        range: Range,
    ) -> ProviderResult<Vec<Diagnostic>> {
        let all_diagnostics = self.generate_diagnostics(code, language)?;

        // Filter diagnostics that fall within the specified range
        let filtered = all_diagnostics
            .into_iter()
            .filter(|diag| {
                diag.range.start.line >= range.start.line && diag.range.end.line <= range.end.line
            })
            .collect();

        Ok(filtered)
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

impl Default for GenericDiagnosticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDiagnosticsProvider;

    impl DiagnosticsProvider for MockDiagnosticsProvider {
        fn language(&self) -> &str {
            "mock"
        }

        fn generate_diagnostics(&self, _code: &str) -> ProviderResult<Vec<Diagnostic>> {
            Ok(vec![])
        }

        fn config(&self) -> Option<&crate::config::LanguageConfig> {
            None
        }
    }

    #[test]
    fn test_generic_diagnostics_engine_with_provider() {
        let mut engine = GenericDiagnosticsEngine::new();
        engine.register_provider(Box::new(MockDiagnosticsProvider));

        assert!(engine.has_provider("mock"));
        assert!(engine.generate_diagnostics("test", "mock").is_ok());
    }

    #[test]
    fn test_generic_diagnostics_engine_fallback() {
        let engine = GenericDiagnosticsEngine::new();

        // Should gracefully degrade for unknown language
        assert!(!engine.has_provider("unknown"));
        let result = engine.generate_diagnostics("test", "unknown");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_generic_diagnostics_engine_languages() {
        let mut engine = GenericDiagnosticsEngine::new();
        engine.register_provider(Box::new(MockDiagnosticsProvider));

        let languages = engine.languages();
        assert!(languages.contains(&"mock"));
    }
}
