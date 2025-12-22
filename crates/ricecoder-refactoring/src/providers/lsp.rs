//! LSP provider trait and implementation for external LSP server integration
//!
//! This module provides traits and implementations for integrating external LSP servers
//! as the highest-priority provider in the refactoring engine.

use std::sync::Arc;

use crate::{
    error::Result,
    types::{Refactoring, ValidationResult},
};

/// Trait for LSP-based refactoring providers
///
/// LSP providers delegate refactoring operations to external Language Server Protocol servers.
/// This enables leveraging specialized, maintained language tools instead of building them into ricecoder.
pub trait LspProvider: Send + Sync {
    /// Check if the LSP server is currently available
    fn is_available(&self) -> bool;

    /// Perform a refactoring operation via LSP
    fn perform_refactoring(
        &self,
        code: &str,
        language: &str,
        refactoring: &Refactoring,
    ) -> Result<String>;

    /// Validate refactored code via LSP
    fn validate_refactoring(
        &self,
        original: &str,
        refactored: &str,
        language: &str,
    ) -> Result<ValidationResult>;

    /// Register a callback for availability changes
    fn on_availability_changed(&self, callback: Box<dyn Fn(bool) + Send + Sync>);
}

/// Registry for LSP providers
///
/// Manages LSP providers for different languages and supports hot-reload
/// of provider availability without system restart.
#[derive(Clone)]
pub struct LspProviderRegistry {
    providers: Arc<std::sync::Mutex<std::collections::HashMap<String, Arc<dyn LspProvider>>>>,
}

impl LspProviderRegistry {
    /// Create a new LSP provider registry
    pub fn new() -> Self {
        Self {
            providers: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Register an LSP provider for a language
    pub fn register(&self, language: String, provider: Arc<dyn LspProvider>) -> Result<()> {
        let mut providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other(
                "Failed to acquire lock on LSP provider registry".to_string(),
            )
        })?;
        providers.insert(language, provider);
        Ok(())
    }

    /// Get an LSP provider for a language
    pub fn get_provider(&self, language: &str) -> Option<Arc<dyn LspProvider>> {
        if let Ok(providers) = self.providers.lock() {
            providers.get(language).cloned()
        } else {
            None
        }
    }

    /// Check if an LSP provider is available for a language
    pub fn is_available(&self, language: &str) -> bool {
        if let Some(provider) = self.get_provider(language) {
            provider.is_available()
        } else {
            false
        }
    }

    /// Get all registered languages with LSP providers
    pub fn get_languages(&self) -> Result<Vec<String>> {
        let providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other(
                "Failed to acquire lock on LSP provider registry".to_string(),
            )
        })?;
        Ok(providers.keys().cloned().collect())
    }

    /// Unregister an LSP provider for a language
    pub fn unregister(&self, language: &str) -> Result<()> {
        let mut providers = self.providers.lock().map_err(|_| {
            crate::error::RefactoringError::Other(
                "Failed to acquire lock on LSP provider registry".to_string(),
            )
        })?;
        providers.remove(language);
        Ok(())
    }

    /// Update provider availability for a language
    pub fn update_availability(&self, language: &str, available: bool) -> Result<()> {
        if let Some(provider) = self.get_provider(language) {
            if provider.is_available() != available {
                // Provider availability changed - this would trigger callbacks
                // In a real implementation, we'd notify watchers here
            }
        }
        Ok(())
    }
}

impl Default for LspProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_lsp_provider_registry() -> Result<()> {
        let registry = LspProviderRegistry::new();

        let available = std::sync::Arc::new(std::sync::Mutex::new(true));
        let provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider {
            available: available.clone(),
        });

        registry.register("rust".to_string(), provider.clone())?;

        assert!(registry.is_available("rust"));
        assert!(!registry.is_available("python"));

        // Change availability
        *available.lock().unwrap() = false;
        assert!(!registry.is_available("rust"));

        Ok(())
    }

    #[test]
    fn test_get_languages() -> Result<()> {
        let registry = LspProviderRegistry::new();

        let available = std::sync::Arc::new(std::sync::Mutex::new(true));
        let provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider {
            available: available.clone(),
        });

        registry.register("rust".to_string(), provider.clone())?;
        registry.register("typescript".to_string(), provider.clone())?;

        let languages = registry.get_languages()?;
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"typescript".to_string()));

        Ok(())
    }

    #[test]
    fn test_unregister_provider() -> Result<()> {
        let registry = LspProviderRegistry::new();

        let available = std::sync::Arc::new(std::sync::Mutex::new(true));
        let provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider {
            available: available.clone(),
        });

        registry.register("rust".to_string(), provider)?;
        assert!(registry.is_available("rust"));

        registry.unregister("rust")?;
        assert!(!registry.is_available("rust"));

        Ok(())
    }
}
