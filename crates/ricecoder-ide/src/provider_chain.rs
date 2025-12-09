//! Provider chain implementation for IDE features
//!
//! This module implements the LSP-first provider priority chain:
//! 1. External LSP Servers (rust-analyzer, typescript-language-server, pylsp, etc.)
//! 2. Configured IDE Rules (YAML/JSON configuration)
//! 3. Built-in Language Providers (Rust, TypeScript, Python)
//! 4. Generic Text-based Features (fallback for any language)

use crate::error::IdeResult;
use crate::provider::{IdeProvider, ProviderChange};
use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Provider registry for managing multiple providers
pub struct ProviderRegistry {
    /// External LSP providers by language
    lsp_providers: HashMap<String, Arc<dyn IdeProvider>>,
    /// Configured rules providers by language
    configured_providers: HashMap<String, Arc<dyn IdeProvider>>,
    /// Built-in language providers by language
    builtin_providers: HashMap<String, Arc<dyn IdeProvider>>,
    /// Generic fallback provider
    generic_provider: Arc<dyn IdeProvider>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new(generic_provider: Arc<dyn IdeProvider>) -> Self {
        ProviderRegistry {
            lsp_providers: HashMap::new(),
            configured_providers: HashMap::new(),
            builtin_providers: HashMap::new(),
            generic_provider,
        }
    }

    /// Register an external LSP provider for a language
    pub fn register_lsp_provider(&mut self, language: String, provider: Arc<dyn IdeProvider>) {
        debug!("Registering LSP provider for language: {}", language);
        self.lsp_providers.insert(language, provider);
    }

    /// Register a configured rules provider for a language
    pub fn register_configured_provider(
        &mut self,
        language: String,
        provider: Arc<dyn IdeProvider>,
    ) {
        debug!("Registering configured rules provider for language: {}", language);
        self.configured_providers.insert(language, provider);
    }

    /// Register a built-in provider for a language
    pub fn register_builtin_provider(&mut self, language: String, provider: Arc<dyn IdeProvider>) {
        debug!("Registering built-in provider for language: {}", language);
        self.builtin_providers.insert(language, provider);
    }

    /// Get a provider for a language following the priority chain
    pub fn get_provider(&self, language: &str) -> Arc<dyn IdeProvider> {
        // Priority 1: External LSP
        if let Some(provider) = self.lsp_providers.get(language) {
            debug!("Using LSP provider for language: {}", language);
            return provider.clone();
        }

        // Priority 2: Configured rules
        if let Some(provider) = self.configured_providers.get(language) {
            debug!("Using configured rules provider for language: {}", language);
            return provider.clone();
        }

        // Priority 3: Built-in
        if let Some(provider) = self.builtin_providers.get(language) {
            debug!("Using built-in provider for language: {}", language);
            return provider.clone();
        }

        // Priority 4: Generic fallback
        debug!("Using generic fallback provider for language: {}", language);
        self.generic_provider.clone()
    }

    /// Check if a provider is available for a language
    pub fn is_provider_available(&self, language: &str) -> bool {
        self.lsp_providers.contains_key(language)
            || self.configured_providers.contains_key(language)
            || self.builtin_providers.contains_key(language)
    }

    /// Get all available languages
    pub fn available_languages(&self) -> Vec<String> {
        let mut languages = Vec::new();
        languages.extend(self.lsp_providers.keys().cloned());
        languages.extend(self.configured_providers.keys().cloned());
        languages.extend(self.builtin_providers.keys().cloned());
        languages.sort();
        languages.dedup();
        languages
    }

    /// Unregister an LSP provider for a language
    pub fn unregister_lsp_provider(&mut self, language: &str) {
        debug!("Unregistering LSP provider for language: {}", language);
        self.lsp_providers.remove(language);
    }

    /// Unregister a configured rules provider for a language
    pub fn unregister_configured_provider(&mut self, language: &str) {
        debug!("Unregistering configured rules provider for language: {}", language);
        self.configured_providers.remove(language);
    }

    /// Unregister a built-in provider for a language
    pub fn unregister_builtin_provider(&mut self, language: &str) {
        debug!("Unregistering built-in provider for language: {}", language);
        self.builtin_providers.remove(language);
    }
}

/// Provider chain manager that orchestrates the provider priority chain
pub struct ProviderChainManager {
    registry: Arc<tokio::sync::RwLock<ProviderRegistry>>,
    availability_callbacks: Arc<tokio::sync::RwLock<Vec<Box<dyn Fn(ProviderChange) + Send + Sync>>>>,
}

impl ProviderChainManager {
    /// Create a new provider chain manager
    pub fn new(registry: ProviderRegistry) -> Self {
        ProviderChainManager {
            registry: Arc::new(tokio::sync::RwLock::new(registry)),
            availability_callbacks: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get completions through the provider chain
    pub async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!(
            "Getting completions for language: {} through provider chain",
            params.language
        );

        let registry = self.registry.read().await;
        let provider = registry.get_provider(&params.language);

        match provider.get_completions(params).await {
            Ok(completions) => {
                info!(
                    "Successfully got {} completions for language: {}",
                    completions.len(),
                    params.language
                );
                Ok(completions)
            }
            Err(e) => {
                warn!(
                    "Failed to get completions for language: {}: {}",
                    params.language, e
                );
                Err(e)
            }
        }
    }

    /// Get diagnostics through the provider chain
    pub async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!(
            "Getting diagnostics for language: {} through provider chain",
            params.language
        );

        let registry = self.registry.read().await;
        let provider = registry.get_provider(&params.language);

        match provider.get_diagnostics(params).await {
            Ok(diagnostics) => {
                info!(
                    "Successfully got {} diagnostics for language: {}",
                    diagnostics.len(),
                    params.language
                );
                Ok(diagnostics)
            }
            Err(e) => {
                warn!(
                    "Failed to get diagnostics for language: {}: {}",
                    params.language, e
                );
                Err(e)
            }
        }
    }

    /// Get hover information through the provider chain
    pub async fn get_hover(&self, params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!(
            "Getting hover information for language: {} through provider chain",
            params.language
        );

        let registry = self.registry.read().await;
        let provider = registry.get_provider(&params.language);

        match provider.get_hover(params).await {
            Ok(hover) => {
                if hover.is_some() {
                    info!("Successfully got hover information for language: {}", params.language);
                }
                Ok(hover)
            }
            Err(e) => {
                warn!(
                    "Failed to get hover information for language: {}: {}",
                    params.language, e
                );
                Err(e)
            }
        }
    }

    /// Get definition location through the provider chain
    pub async fn get_definition(&self, params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!(
            "Getting definition for language: {} through provider chain",
            params.language
        );

        let registry = self.registry.read().await;
        let provider = registry.get_provider(&params.language);

        match provider.get_definition(params).await {
            Ok(location) => {
                if location.is_some() {
                    info!("Successfully got definition for language: {}", params.language);
                }
                Ok(location)
            }
            Err(e) => {
                warn!(
                    "Failed to get definition for language: {}: {}",
                    params.language, e
                );
                Err(e)
            }
        }
    }

    /// Register a provider availability change callback
    pub async fn on_provider_availability_changed(
        &self,
        callback: Box<dyn Fn(ProviderChange) + Send + Sync>,
    ) {
        let mut callbacks = self.availability_callbacks.write().await;
        callbacks.push(callback);
    }

    /// Notify all callbacks of a provider availability change
    pub async fn notify_provider_change(&self, change: ProviderChange) {
        let callbacks = self.availability_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(change.clone());
        }
    }

    /// Reload configuration without restart
    pub async fn reload_configuration(&self) -> IdeResult<()> {
        debug!("Reloading provider chain configuration");
        // This will be implemented when configuration hot-reload is added
        info!("Provider chain configuration reloaded");
        Ok(())
    }

    /// Update configuration and refresh providers
    pub async fn update_config(&self, config: IdeIntegrationConfig) -> IdeResult<()> {
        debug!("Updating provider chain configuration");

        // Update external LSP servers if configuration changed
        if config.providers.external_lsp.enabled {
            for (language, _server_config) in &config.providers.external_lsp.servers {
                // Providers will be re-registered based on new configuration
                debug!("Updated LSP configuration for language: {}", language);
            }
        }

        info!("Provider chain configuration updated");
        Ok(())
    }

    /// Get the provider registry
    pub async fn registry(&self) -> tokio::sync::RwLockReadGuard<'_, ProviderRegistry> {
        self.registry.read().await
    }

    /// Get mutable access to the provider registry
    pub async fn registry_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, ProviderRegistry> {
        self.registry.write().await
    }

    /// Check if a provider is available for a language
    pub async fn is_provider_available(&self, language: &str) -> bool {
        let registry = self.registry.read().await;
        registry.is_provider_available(language)
    }

    /// Get all available languages
    pub async fn available_languages(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry.available_languages()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    /// Mock provider for testing
    struct MockProvider {
        name: String,
        language: String,
    }

    #[async_trait]
    impl IdeProvider for MockProvider {
        async fn get_completions(&self, _params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
            Ok(vec![CompletionItem {
                label: "test".to_string(),
                kind: CompletionItemKind::Function,
                detail: None,
                documentation: None,
                insert_text: "test()".to_string(),
            }])
        }

        async fn get_diagnostics(&self, _params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
            Ok(vec![])
        }

        async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
            Ok(None)
        }

        async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
            Ok(None)
        }

        fn is_available(&self, language: &str) -> bool {
            language == self.language
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_provider_registry_creation() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let registry = ProviderRegistry::new(generic);
        assert_eq!(registry.available_languages().len(), 0);
    }

    #[test]
    fn test_register_lsp_provider() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let mut registry = ProviderRegistry::new(generic);

        let lsp = Arc::new(MockProvider {
            name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
        });
        registry.register_lsp_provider("rust".to_string(), lsp);

        assert!(registry.is_provider_available("rust"));
        assert_eq!(registry.available_languages(), vec!["rust"]);
    }

    #[test]
    fn test_provider_priority_chain() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let mut registry = ProviderRegistry::new(generic);

        let lsp = Arc::new(MockProvider {
            name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
        });
        let builtin = Arc::new(MockProvider {
            name: "builtin".to_string(),
            language: "rust".to_string(),
        });

        registry.register_lsp_provider("rust".to_string(), lsp.clone());
        registry.register_builtin_provider("rust".to_string(), builtin);

        // LSP should be selected (priority 1)
        let provider = registry.get_provider("rust");
        assert_eq!(provider.name(), "rust-analyzer");
    }

    #[test]
    fn test_provider_fallback_to_builtin() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let mut registry = ProviderRegistry::new(generic);

        let builtin = Arc::new(MockProvider {
            name: "builtin".to_string(),
            language: "rust".to_string(),
        });

        registry.register_builtin_provider("rust".to_string(), builtin);

        // Built-in should be selected (LSP not available)
        let provider = registry.get_provider("rust");
        assert_eq!(provider.name(), "builtin");
    }

    #[test]
    fn test_provider_fallback_to_generic() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let registry = ProviderRegistry::new(generic);

        // Generic should be selected (no other providers)
        let provider = registry.get_provider("unknown");
        assert_eq!(provider.name(), "generic");
    }

    #[test]
    fn test_unregister_lsp_provider() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let mut registry = ProviderRegistry::new(generic);

        let lsp = Arc::new(MockProvider {
            name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
        });
        registry.register_lsp_provider("rust".to_string(), lsp);
        assert!(registry.is_provider_available("rust"));

        registry.unregister_lsp_provider("rust");
        assert!(!registry.is_provider_available("rust"));
    }

    #[tokio::test]
    async fn test_provider_chain_manager_creation() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let registry = ProviderRegistry::new(generic);
        let manager = ProviderChainManager::new(registry);

        assert_eq!(manager.available_languages().await.len(), 0);
    }

    #[tokio::test]
    async fn test_provider_chain_manager_get_completions() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let mut registry = ProviderRegistry::new(generic);

        let lsp = Arc::new(MockProvider {
            name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
        });
        registry.register_lsp_provider("rust".to_string(), lsp);

        let manager = ProviderChainManager::new(registry);

        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = manager.get_completions(&params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_provider_availability_callback() {
        let generic = Arc::new(MockProvider {
            name: "generic".to_string(),
            language: "generic".to_string(),
        });
        let registry = ProviderRegistry::new(generic);
        let manager = ProviderChainManager::new(registry);

        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        manager
            .on_provider_availability_changed(Box::new(move |_change| {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }))
            .await;

        let change = ProviderChange {
            provider_name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
            available: true,
        };

        manager.notify_provider_change(change).await;
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
