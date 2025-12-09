//! Provider traits and interfaces for IDE features

use crate::error::IdeResult;
use crate::types::*;
use async_trait::async_trait;

/// IDE provider trait for IDE features
#[async_trait]
pub trait IdeProvider: Send + Sync {
    /// Get completions for a given context
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>>;

    /// Get diagnostics for a given source
    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>>;

    /// Get hover information for a given position
    async fn get_hover(&self, params: &HoverParams) -> IdeResult<Option<Hover>>;

    /// Get definition location for a given position
    async fn get_definition(&self, params: &DefinitionParams) -> IdeResult<Option<Location>>;

    /// Check if this provider is available for the given language
    fn is_available(&self, language: &str) -> bool;

    /// Get the provider name
    fn name(&self) -> &str;
}

/// Provider chain trait for managing provider priority
#[async_trait]
pub trait ProviderChain: Send + Sync {
    /// Get completions through the provider chain
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>>;

    /// Get diagnostics through the provider chain
    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>>;

    /// Get hover information through the provider chain
    async fn get_hover(&self, params: &HoverParams) -> IdeResult<Option<Hover>>;

    /// Get definition location through the provider chain
    async fn get_definition(&self, params: &DefinitionParams) -> IdeResult<Option<Location>>;

    /// Register a provider availability change callback
    fn on_provider_availability_changed(
        &self,
        callback: Box<dyn Fn(ProviderChange) + Send + Sync>,
    );

    /// Reload configuration without restart
    async fn reload_configuration(&self) -> IdeResult<()>;
}

/// Provider availability change event
#[derive(Debug, Clone)]
pub struct ProviderChange {
    /// Provider name
    pub provider_name: String,
    /// Language
    pub language: String,
    /// Whether the provider became available (true) or unavailable (false)
    pub available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_change_creation() {
        let change = ProviderChange {
            provider_name: "rust-analyzer".to_string(),
            language: "rust".to_string(),
            available: true,
        };

        assert_eq!(change.provider_name, "rust-analyzer");
        assert_eq!(change.language, "rust");
        assert!(change.available);
    }
}
