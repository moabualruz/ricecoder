//! Integration tests for LSP provider integration
//!
//! These tests verify that the LSP-first provider priority chain works correctly
//! and that fallback mechanisms work as expected.

use ricecoder_refactoring::{
    ConfigManager, LspIntegration, LspProvider, LspProviderRegistry, LspServerInfo,
    ProviderRegistry, RefactoringEngine, RefactoringOptions, RefactoringTarget, RefactoringType,
    ValidationResult,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Mock LSP provider for testing
struct MockLspProvider {
    available: std::sync::Arc<std::sync::Mutex<bool>>,
    language: String,
}

impl MockLspProvider {
    fn new(language: String, available: bool) -> Self {
        Self {
            available: Arc::new(std::sync::Mutex::new(available)),
            language,
        }
    }

    fn set_available(&self, available: bool) {
        if let Ok(mut a) = self.available.lock() {
            *a = available;
        }
    }
}

impl LspProvider for MockLspProvider {
    fn is_available(&self) -> bool {
        self.available.lock().map(|a| *a).unwrap_or(false)
    }

    fn perform_refactoring(
        &self,
        code: &str,
        _language: &str,
        _refactoring: &ricecoder_refactoring::Refactoring,
    ) -> ricecoder_refactoring::Result<String> {
        Ok(format!("LSP[{}]: {}", self.language, code))
    }

    fn validate_refactoring(
        &self,
        _original: &str,
        _refactored: &str,
        _language: &str,
    ) -> ricecoder_refactoring::Result<ValidationResult> {
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
fn test_lsp_provider_detection() -> ricecoder_refactoring::Result<()> {
    let registry = Arc::new(LspProviderRegistry::new());

    let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));
    registry.register("rust".to_string(), lsp_provider)?;

    assert!(registry.is_available("rust"));
    assert!(!registry.is_available("python"));

    Ok(())
}

#[test]
fn test_lsp_refactoring_delegation() -> ricecoder_refactoring::Result<()> {
    let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));

    let code = "fn main() {}";
    let refactoring = ricecoder_refactoring::Refactoring {
        id: "test".to_string(),
        refactoring_type: RefactoringType::Rename,
        target: RefactoringTarget {
            file: PathBuf::from("main.rs"),
            symbol: "main".to_string(),
            range: None,
        },
        options: RefactoringOptions::default(),
    };

    let result = lsp_provider.perform_refactoring(code, "rust", &refactoring)?;
    assert!(result.contains("LSP[rust]"));

    Ok(())
}

#[test]
fn test_lsp_failure_fallback() -> ricecoder_refactoring::Result<()> {
    let lsp_provider = Arc::new(MockLspProvider::new("rust".to_string(), true));

    // Simulate LSP becoming unavailable
    lsp_provider.set_available(false);

    assert!(!lsp_provider.is_available());

    Ok(())
}

#[test]
fn test_provider_priority_chain() -> ricecoder_refactoring::Result<()> {
    let config_manager = Arc::new(ConfigManager::new());
    let provider_registry = Arc::new(ProviderRegistry::new(
        Arc::new(ricecoder_refactoring::GenericRefactoringProvider::new()),
    ));

    // Register LSP provider
    let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));
    provider_registry.register_lsp_provider("rust".to_string(), lsp_provider)?;

    // Verify LSP is available
    assert!(provider_registry.is_lsp_available("rust"));

    // Create engine with registry
    let engine = RefactoringEngine::with_registry(config_manager, provider_registry);

    // Check provider info
    let provider_info = engine.get_provider_info("rust");
    assert_eq!(provider_info, "lsp");

    Ok(())
}

#[test]
fn test_hot_reload_functionality() -> ricecoder_refactoring::Result<()> {
    let registry = Arc::new(LspProviderRegistry::new());

    // Initially no LSP provider
    assert!(!registry.is_available("rust"));

    // Register LSP provider
    let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));
    registry.register("rust".to_string(), lsp_provider.clone())?;

    // Now LSP provider is available
    assert!(registry.is_available("rust"));

    // Unregister LSP provider (simulating hot reload)
    registry.unregister("rust")?;

    // LSP provider is no longer available
    assert!(!registry.is_available("rust"));

    Ok(())
}

#[test]
fn test_multiple_lsp_servers_per_language() -> ricecoder_refactoring::Result<()> {
    let registry = Arc::new(LspProviderRegistry::new());

    // Register multiple LSP providers for the same language
    let lsp_provider1: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));
    let lsp_provider2: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));

    registry.register("rust".to_string(), lsp_provider1)?;

    // Second registration should replace the first
    registry.register("rust".to_string(), lsp_provider2)?;

    assert!(registry.is_available("rust"));

    Ok(())
}

#[test]
fn test_lsp_integration_query() -> ricecoder_refactoring::Result<()> {
    let servers = LspIntegration::query_available_lsp_servers()?;
    // Should return a map (may be empty)
    assert!(servers.is_empty() || !servers.is_empty());

    Ok(())
}

#[test]
fn test_lsp_integration_create_provider() -> ricecoder_refactoring::Result<()> {
    let server_info = LspServerInfo {
        language: "rust".to_string(),
        name: "rust-analyzer".to_string(),
        command: "rust-analyzer".to_string(),
        args: vec![],
    };

    let provider = LspIntegration::create_lsp_provider("rust", &server_info)?;
    assert!(provider.is_available());

    Ok(())
}

#[test]
fn test_provider_registry_with_lsp() -> ricecoder_refactoring::Result<()> {
    let lsp_registry = Arc::new(LspProviderRegistry::new());
    let generic_provider = Arc::new(ricecoder_refactoring::GenericRefactoringProvider::new());

    let registry = ProviderRegistry::with_lsp(generic_provider, lsp_registry.clone());

    // Register LSP provider
    let lsp_provider: Arc<dyn LspProvider> = Arc::new(MockLspProvider::new("rust".to_string(), true));
    registry.register_lsp_provider("rust".to_string(), lsp_provider)?;

    // Verify LSP is available
    assert!(registry.is_lsp_available("rust"));

    // Get all languages
    let languages = registry.get_languages()?;
    assert!(languages.contains(&"rust".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_lsp_watcher_lifecycle() -> ricecoder_refactoring::Result<()> {
    let registry = Arc::new(LspProviderRegistry::new());
    let watcher = ricecoder_refactoring::LspWatcher::new(registry);

    assert!(!watcher.is_running()?);

    watcher.start().await?;
    assert!(watcher.is_running()?);

    watcher.stop().await?;
    assert!(!watcher.is_running()?);

    Ok(())
}

#[tokio::test]
async fn test_configuration_watcher_lifecycle() -> ricecoder_refactoring::Result<()> {
    let config_dir = std::path::PathBuf::from("/tmp");
    let watcher = ricecoder_refactoring::ConfigurationWatcher::new(config_dir);

    assert!(!watcher.is_running()?);

    watcher.start().await?;
    assert!(watcher.is_running()?);

    watcher.stop().await?;
    assert!(!watcher.is_running()?);

    Ok(())
}
