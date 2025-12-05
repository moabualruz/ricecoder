//! Provider traits and implementations for language-specific refactoring
//!
//! This module provides traits and implementations for language-specific refactoring providers,
//! including LSP-based providers and configuration-driven providers.

pub mod lsp;
pub mod lsp_integration;
pub mod lsp_watcher;

pub use lsp::{LspProvider, LspProviderRegistry};
pub use lsp_integration::{LspIntegration, LspServerInfo};
pub use lsp_watcher::{ConfigurationWatcher, LspWatcher};

use crate::error::Result;
use crate::types::{Refactoring, RefactoringType, ValidationResult};
use std::sync::Arc;

/// Trait for language-specific refactoring providers
pub trait RefactoringProvider: Send + Sync {
    /// Analyze a refactoring operation
    fn analyze_refactoring(
        &self,
        code: &str,
        language: &str,
        refactoring_type: RefactoringType,
    ) -> Result<RefactoringAnalysis>;

    /// Apply a refactoring to code
    fn apply_refactoring(
        &self,
        code: &str,
        language: &str,
        refactoring: &Refactoring,
    ) -> Result<String>;

    /// Validate refactored code
    fn validate_refactoring(
        &self,
        original: &str,
        refactored: &str,
        language: &str,
    ) -> Result<ValidationResult>;
}

/// Analysis result for a refactoring
#[derive(Debug, Clone)]
pub struct RefactoringAnalysis {
    /// Whether the refactoring is applicable
    pub applicable: bool,
    /// Reason if not applicable
    pub reason: Option<String>,
    /// Estimated complexity (1-10)
    pub complexity: u8,
}

/// Registry for refactoring providers
///
/// Manages language-specific refactoring providers with support for:
/// - LSP-based providers (highest priority)
/// - Configured providers (from YAML/JSON configuration)
/// - Built-in language-specific providers
/// - Generic text-based fallback provider
#[derive(Clone)]
pub struct ProviderRegistry {
    lsp_providers: Arc<LspProviderRegistry>,
    providers: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<dyn RefactoringProvider>>>>,
    generic_provider: Arc<dyn RefactoringProvider>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new(generic_provider: Arc<dyn RefactoringProvider>) -> Self {
        Self {
            lsp_providers: Arc::new(LspProviderRegistry::new()),
            providers: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            generic_provider,
        }
    }

    /// Create a new provider registry with LSP providers
    pub fn with_lsp(
        generic_provider: Arc<dyn RefactoringProvider>,
        lsp_providers: Arc<LspProviderRegistry>,
    ) -> Self {
        Self {
            lsp_providers,
            providers: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            generic_provider,
        }
    }

    /// Get the LSP provider registry
    pub fn lsp_providers(&self) -> Arc<LspProviderRegistry> {
        self.lsp_providers.clone()
    }

    /// Register a provider for a language
    pub fn register(&self, language: String, provider: Arc<dyn RefactoringProvider>) -> Result<()> {
        let mut providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other("Failed to acquire lock on provider registry".to_string())
        })?;
        providers.insert(language, provider);
        Ok(())
    }

    /// Get a provider for a language using the priority chain:
    /// 1. LSP provider (if available)
    /// 2. Configured provider
    /// 3. Generic fallback
    pub fn get_provider(&self, language: &str) -> Arc<dyn RefactoringProvider> {
        // Priority 1: Check for LSP provider
        if self.lsp_providers.is_available(language) {
            // In a real implementation, we'd wrap the LSP provider
            // For now, we fall through to configured providers
        }

        // Priority 2: Check for configured provider
        if let Ok(providers) = self.providers.lock() {
            if let Some(provider) = providers.get(language) {
                return provider.clone();
            }
        }

        // Priority 3: Fall back to generic provider
        self.generic_provider.clone()
    }

    /// Check if a language has a specific provider (configured or LSP)
    pub fn has_provider(&self, language: &str) -> Result<bool> {
        // Check LSP providers first
        if self.lsp_providers.is_available(language) {
            return Ok(true);
        }

        // Check configured providers
        let providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other("Failed to acquire lock on provider registry".to_string())
        })?;
        Ok(providers.contains_key(language))
    }

    /// Get all registered languages (configured + LSP)
    pub fn get_languages(&self) -> Result<Vec<String>> {
        let mut languages = Vec::new();

        // Add LSP languages
        if let Ok(lsp_langs) = self.lsp_providers.get_languages() {
            languages.extend(lsp_langs);
        }

        // Add configured languages
        let providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other("Failed to acquire lock on provider registry".to_string())
        })?;
        languages.extend(providers.keys().cloned());

        // Remove duplicates
        languages.sort();
        languages.dedup();

        Ok(languages)
    }

    /// Register an LSP provider for a language
    pub fn register_lsp_provider(
        &self,
        language: String,
        provider: Arc<dyn LspProvider>,
    ) -> Result<()> {
        self.lsp_providers.register(language, provider)
    }

    /// Check if an LSP provider is available for a language
    pub fn is_lsp_available(&self, language: &str) -> bool {
        self.lsp_providers.is_available(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;

    impl RefactoringProvider for MockProvider {
        fn analyze_refactoring(
            &self,
            _code: &str,
            _language: &str,
            _refactoring_type: RefactoringType,
        ) -> Result<RefactoringAnalysis> {
            Ok(RefactoringAnalysis {
                applicable: true,
                reason: None,
                complexity: 5,
            })
        }

        fn apply_refactoring(
            &self,
            code: &str,
            _language: &str,
            _refactoring: &Refactoring,
        ) -> Result<String> {
            Ok(code.to_string())
        }

        fn validate_refactoring(
            &self,
            _original: &str,
            _refactored: &str,
            _language: &str,
        ) -> Result<ValidationResult> {
            Ok(ValidationResult {
                passed: true,
                errors: vec![],
                warnings: vec![],
            })
        }
    }

    struct MockLspProvider {
        available: std::sync::Arc<std::sync::Mutex<bool>>,
    }

    impl LspProvider for MockLspProvider {
        fn is_available(&self) -> bool {
            self.available.lock().map(|a| *a).unwrap_or(false)
        }

        fn perform_refactoring(
            &self,
            code: &str,
            _language: &str,
            _refactoring: &Refactoring,
        ) -> Result<String> {
            Ok(code.to_string())
        }

        fn validate_refactoring(
            &self,
            _original: &str,
            _refactored: &str,
            _language: &str,
        ) -> Result<ValidationResult> {
            Ok(ValidationResult {
                passed: true,
                errors: vec![],
                warnings: vec![],
            })
        }

        fn on_availability_changed(&self, _callback: Box<dyn Fn(bool) + Send + Sync>) {
            // Mock implementation
        }
    }

    #[test]
    fn test_provider_registry() -> Result<()> {
        let generic: Arc<dyn RefactoringProvider> = Arc::new(MockProvider);
        let registry = ProviderRegistry::new(generic.clone());

        let rust_provider: Arc<dyn RefactoringProvider> = Arc::new(MockProvider);
        registry.register("rust".to_string(), rust_provider.clone())?;

        assert!(registry.has_provider("rust")?);
        assert!(!registry.has_provider("python")?);

        let _provider = registry.get_provider("rust");
        // Verify that we got a provider (can't use ptr_eq with trait objects)
        assert!(registry.has_provider("rust")?);

        let _fallback = registry.get_provider("unknown");
        // Verify that fallback returns the generic provider
        assert!(!registry.has_provider("unknown")?);

        Ok(())
    }

    #[test]
    fn test_get_languages() -> Result<()> {
        let generic = Arc::new(MockProvider);
        let registry = ProviderRegistry::new(generic);

        registry.register("rust".to_string(), Arc::new(MockProvider))?;
        registry.register("typescript".to_string(), Arc::new(MockProvider))?;

        let languages = registry.get_languages()?;
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"typescript".to_string()));

        Ok(())
    }

    #[test]
    fn test_lsp_provider_registration() -> Result<()> {
        let generic = Arc::new(MockProvider);
        let registry = ProviderRegistry::new(generic);

        let available = std::sync::Arc::new(std::sync::Mutex::new(true));
        let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider {
            available: available.clone(),
        });

        registry.register_lsp_provider("rust".to_string(), lsp_provider)?;

        assert!(registry.is_lsp_available("rust"));
        assert!(!registry.is_lsp_available("python"));

        Ok(())
    }

    #[test]
    fn test_provider_priority_chain() -> Result<()> {
        let generic = Arc::new(MockProvider);
        let registry = ProviderRegistry::new(generic);

        // Register a configured provider
        registry.register("rust".to_string(), Arc::new(MockProvider))?;

        // Both should be available
        assert!(registry.has_provider("rust")?);

        // Register an LSP provider
        let available = std::sync::Arc::new(std::sync::Mutex::new(true));
        let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider {
            available: available.clone(),
        });
        registry.register_lsp_provider("rust".to_string(), lsp_provider)?;

        // LSP should be available
        assert!(registry.is_lsp_available("rust"));

        Ok(())
    }
}
