//! Provider traits for pluggable language-specific implementations
//!
//! This module defines traits for pluggable providers that enable
//! language-agnostic architecture with configuration-driven behavior.

use crate::types::{Diagnostic, SemanticInfo, Symbol, Position};
use crate::config::LanguageConfig;

/// Error type for provider operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum ProviderError {
    /// Provider not found
    #[error("Provider not found: {0}")]
    NotFound(String),

    /// Provider error
    #[error("Provider error: {0}")]
    Error(String),

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}



/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Trait for language-specific semantic analysis providers
pub trait SemanticAnalyzerProvider: Send + Sync {
    /// Get the language identifier this provider handles
    fn language(&self) -> &str;

    /// Analyze code and extract semantic information
    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo>;

    /// Extract symbols from code
    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>>;

    /// Get hover information at a specific position
    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>>;
}

/// Trait for language-specific diagnostic rule providers
pub trait DiagnosticsProvider: Send + Sync {
    /// Get the language identifier this provider handles
    fn language(&self) -> &str;

    /// Generate diagnostics for the given code
    fn generate_diagnostics(&self, code: &str) -> ProviderResult<Vec<Diagnostic>>;

    /// Get the configuration for this provider
    fn config(&self) -> Option<&LanguageConfig>;
}

/// Trait for language-specific code action providers
pub trait CodeActionProvider: Send + Sync {
    /// Get the language identifier this provider handles
    fn language(&self) -> &str;

    /// Suggest code actions for a diagnostic
    fn suggest_actions(&self, diagnostic: &Diagnostic, code: &str) -> ProviderResult<Vec<String>>;

    /// Apply a code action to code
    fn apply_action(&self, code: &str, action: &str) -> ProviderResult<String>;

    /// Get the configuration for this provider
    fn config(&self) -> Option<&LanguageConfig>;
}

/// Registry for semantic analyzer providers
pub struct SemanticAnalyzerRegistry {
    providers: std::collections::HashMap<String, Box<dyn SemanticAnalyzerProvider>>,
}

impl SemanticAnalyzerRegistry {
    /// Create a new semantic analyzer registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a semantic analyzer provider
    pub fn register(&mut self, provider: Box<dyn SemanticAnalyzerProvider>) {
        let language = provider.language().to_string();
        self.providers.insert(language, provider);
    }

    /// Get a semantic analyzer provider by language
    pub fn get(&self, language: &str) -> Option<&dyn SemanticAnalyzerProvider> {
        self.providers.get(language).map(|p| p.as_ref())
    }

    /// Check if a provider is registered for a language
    pub fn has_provider(&self, language: &str) -> bool {
        self.providers.contains_key(language)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SemanticAnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for diagnostics providers
pub struct DiagnosticsRegistry {
    providers: std::collections::HashMap<String, Box<dyn DiagnosticsProvider>>,
}

impl DiagnosticsRegistry {
    /// Create a new diagnostics registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a diagnostics provider
    pub fn register(&mut self, provider: Box<dyn DiagnosticsProvider>) {
        let language = provider.language().to_string();
        self.providers.insert(language, provider);
    }

    /// Get a diagnostics provider by language
    pub fn get(&self, language: &str) -> Option<&dyn DiagnosticsProvider> {
        self.providers.get(language).map(|p| p.as_ref())
    }

    /// Check if a provider is registered for a language
    pub fn has_provider(&self, language: &str) -> bool {
        self.providers.contains_key(language)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for DiagnosticsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for code action providers
pub struct CodeActionRegistry {
    providers: std::collections::HashMap<String, Box<dyn CodeActionProvider>>,
}

impl CodeActionRegistry {
    /// Create a new code action registry
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    /// Register a code action provider
    pub fn register(&mut self, provider: Box<dyn CodeActionProvider>) {
        let language = provider.language().to_string();
        self.providers.insert(language, provider);
    }

    /// Get a code action provider by language
    pub fn get(&self, language: &str) -> Option<&dyn CodeActionProvider> {
        self.providers.get(language).map(|p| p.as_ref())
    }

    /// Check if a provider is registered for a language
    pub fn has_provider(&self, language: &str) -> bool {
        self.providers.contains_key(language)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for CodeActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSemanticProvider;

    impl SemanticAnalyzerProvider for MockSemanticProvider {
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

        fn get_hover_info(&self, _code: &str, _position: Position) -> ProviderResult<Option<String>> {
            Ok(None)
        }
    }

    #[test]
    fn test_semantic_analyzer_registry() {
        let mut registry = SemanticAnalyzerRegistry::new();
        let provider = Box::new(MockSemanticProvider);

        registry.register(provider);

        assert!(registry.has_provider("mock"));
        assert!(registry.get("mock").is_some());
        assert!(!registry.has_provider("unknown"));
    }

    #[test]
    fn test_semantic_analyzer_registry_languages() {
        let mut registry = SemanticAnalyzerRegistry::new();
        let provider = Box::new(MockSemanticProvider);

        registry.register(provider);

        let languages = registry.languages();
        assert_eq!(languages.len(), 1);
        assert!(languages.contains(&"mock"));
    }
}
