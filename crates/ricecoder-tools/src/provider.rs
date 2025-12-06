//! Provider trait and registry for tool implementations
//!
//! Implements the hybrid provider pattern with MCP → Built-in → Error priority chain.
//! Supports hot-reload of MCP server availability without restart.

use crate::error::ToolError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, trace};
use uuid::Uuid;

/// Trait for tool providers
#[async_trait]
pub trait Provider: Send + Sync {
    /// Execute a tool operation
    async fn execute(&self, input: &str) -> Result<String, ToolError>;
}

/// MCP server availability cache entry
#[derive(Clone)]
struct AvailabilityEntry {
    available: bool,
    checked_at: Instant,
}

/// Provider selection result with trace information
#[derive(Debug, Clone)]
pub struct ProviderSelection {
    /// Name of the selected provider ("mcp" or "builtin")
    pub provider_name: String,
    /// Trace ID for debugging
    pub trace_id: String,
    /// Whether this was a fallback selection
    pub is_fallback: bool,
}

/// Registry for managing tool providers with fallback chain
pub struct ProviderRegistry {
    mcp_providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    builtin_providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    availability_cache: Arc<RwLock<HashMap<String, AvailabilityEntry>>>,
    cache_ttl: Duration,
    /// Callback for provider selection events
    on_provider_selected: Arc<RwLock<Option<Arc<dyn Fn(ProviderSelection) + Send + Sync>>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            mcp_providers: Arc::new(RwLock::new(HashMap::new())),
            builtin_providers: Arc::new(RwLock::new(HashMap::new())),
            availability_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5),
            on_provider_selected: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new provider registry with custom cache TTL
    pub fn with_cache_ttl(cache_ttl: Duration) -> Self {
        Self {
            mcp_providers: Arc::new(RwLock::new(HashMap::new())),
            builtin_providers: Arc::new(RwLock::new(HashMap::new())),
            availability_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            on_provider_selected: Arc::new(RwLock::new(None)),
        }
    }

    /// Set a callback for provider selection events
    pub async fn on_provider_selected<F>(&self, callback: F)
    where
        F: Fn(ProviderSelection) + Send + Sync + 'static,
    {
        *self.on_provider_selected.write().await = Some(Arc::new(callback));
    }

    /// Register an MCP provider for a tool
    pub async fn register_mcp_provider(
        &self,
        tool_name: impl Into<String>,
        provider: Arc<dyn Provider>,
    ) {
        let tool_name = tool_name.into();
        debug!("Registering MCP provider for tool: {}", tool_name);
        self.mcp_providers.write().await.insert(tool_name, provider);
    }

    /// Register a built-in provider for a tool
    pub async fn register_builtin_provider(
        &self,
        tool_name: impl Into<String>,
        provider: Arc<dyn Provider>,
    ) {
        let tool_name = tool_name.into();
        debug!("Registering built-in provider for tool: {}", tool_name);
        self.builtin_providers.write().await.insert(tool_name, provider);
    }

    /// Check if an MCP provider is available (with cache)
    async fn is_mcp_available(&self, tool_name: &str) -> bool {
        let cache = self.availability_cache.read().await;
        if let Some(entry) = cache.get(tool_name) {
            if entry.checked_at.elapsed() < self.cache_ttl {
                trace!("MCP availability cache hit for tool: {}", tool_name);
                return entry.available;
            }
        }
        drop(cache);

        // Check availability (in real implementation, would query ricecoder-mcp)
        let available = self.mcp_providers.read().await.contains_key(tool_name);

        // Update cache
        self.availability_cache.write().await.insert(
            tool_name.to_string(),
            AvailabilityEntry {
                available,
                checked_at: Instant::now(),
            },
        );

        trace!("MCP availability check for tool: {} = {}", tool_name, available);
        available
    }

    /// Get a provider for a tool using the priority chain
    ///
    /// Returns the provider and selection information including trace ID for debugging.
    pub async fn get_provider(
        &self,
        tool_name: &str,
    ) -> Result<(Arc<dyn Provider>, ProviderSelection), ToolError> {
        let trace_id = Uuid::new_v4().to_string();

        // Try MCP first
        if self.is_mcp_available(tool_name).await {
            if let Some(provider) = self.mcp_providers.read().await.get(tool_name) {
                let selection = ProviderSelection {
                    provider_name: "mcp".to_string(),
                    trace_id: trace_id.clone(),
                    is_fallback: false,
                };
                debug!(
                    trace_id = %trace_id,
                    tool = tool_name,
                    "Using MCP provider for tool"
                );
                self.notify_provider_selected(selection).await;
                return Ok((provider.clone(), ProviderSelection {
                    provider_name: "mcp".to_string(),
                    trace_id,
                    is_fallback: false,
                }));
            }
        }

        // Fall back to built-in
        if let Some(provider) = self.builtin_providers.read().await.get(tool_name) {
            let selection = ProviderSelection {
                provider_name: "builtin".to_string(),
                trace_id: trace_id.clone(),
                is_fallback: true,
            };
            debug!(
                trace_id = %trace_id,
                tool = tool_name,
                "Using built-in provider for tool (MCP unavailable)"
            );
            self.notify_provider_selected(selection.clone()).await;
            return Ok((provider.clone(), selection));
        }

        // No provider available
        warn!(
            trace_id = %trace_id,
            tool = tool_name,
            "No provider available for tool"
        );
        Err(ToolError::new(
            "PROVIDER_NOT_FOUND",
            format!("No provider available for tool: {}", tool_name),
        )
        .with_details(format!("trace_id: {}", trace_id))
        .with_suggestion("Ensure the tool is registered or MCP server is available"))
    }

    /// Get a provider without selection information (simplified API)
    pub async fn get_provider_simple(&self, tool_name: &str) -> Result<Arc<dyn Provider>, ToolError> {
        self.get_provider(tool_name)
            .await
            .map(|(provider, _)| provider)
    }

    /// Notify listeners of provider selection
    async fn notify_provider_selected(&self, selection: ProviderSelection) {
        if let Some(callback) = self.on_provider_selected.read().await.as_ref() {
            callback(selection);
        }
    }

    /// Invalidate availability cache for a tool
    pub async fn invalidate_cache(&self, tool_name: &str) {
        debug!("Invalidating availability cache for tool: {}", tool_name);
        self.availability_cache.write().await.remove(tool_name);
    }

    /// Invalidate all availability caches
    pub async fn invalidate_all_caches(&self) {
        debug!("Invalidating all availability caches");
        self.availability_cache.write().await.clear();
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn execute(&self, _input: &str) -> Result<String, ToolError> {
            Ok(format!("Mock response from {}", self.name))
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert!(registry.mcp_providers.read().await.is_empty());
        assert!(registry.builtin_providers.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_builtin_provider() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
        });
        registry
            .register_builtin_provider("test_tool", provider.clone())
            .await;

        let (retrieved, selection) = registry.get_provider("test_tool").await.unwrap();
        let result = retrieved.execute("test").await.unwrap();
        assert_eq!(result, "Mock response from test");
        assert_eq!(selection.provider_name, "builtin");
        assert!(selection.is_fallback);
    }

    #[tokio::test]
    async fn test_mcp_priority_over_builtin() {
        let registry = ProviderRegistry::new();
        let builtin = Arc::new(MockProvider {
            name: "builtin".to_string(),
        });
        let mcp = Arc::new(MockProvider {
            name: "mcp".to_string(),
        });

        registry
            .register_builtin_provider("test_tool", builtin)
            .await;
        registry.register_mcp_provider("test_tool", mcp).await;

        let (retrieved, selection) = registry.get_provider("test_tool").await.unwrap();
        let result = retrieved.execute("test").await.unwrap();
        assert_eq!(result, "Mock response from mcp");
        assert_eq!(selection.provider_name, "mcp");
        assert!(!selection.is_fallback);
    }

    #[tokio::test]
    async fn test_provider_not_found() {
        let registry = ProviderRegistry::new();
        let result = registry.get_provider("nonexistent").await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.code, "PROVIDER_NOT_FOUND");
        }
    }

    #[tokio::test]
    async fn test_provider_selection_callback() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
        });
        registry
            .register_builtin_provider("test_tool", provider)
            .await;

        let selected = Arc::new(RwLock::new(None));
        let selected_clone = selected.clone();
        registry
            .on_provider_selected(move |selection| {
                let selected = selected_clone.clone();
                let _ = std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        *selected.write().await = Some(selection);
                    });
                });
            })
            .await;

        let _ = registry.get_provider("test_tool").await;
        // Give callback time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_get_provider_simple() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
        });
        registry
            .register_builtin_provider("test_tool", provider)
            .await;

        let retrieved = registry.get_provider_simple("test_tool").await.unwrap();
        let result = retrieved.execute("test").await.unwrap();
        assert_eq!(result, "Mock response from test");
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider {
            name: "test".to_string(),
        });
        registry
            .register_builtin_provider("test_tool", provider)
            .await;

        // Check availability (populates cache)
        let _ = registry.get_provider("test_tool").await;
        assert!(!registry.availability_cache.read().await.is_empty());

        // Invalidate cache
        registry.invalidate_cache("test_tool").await;
        assert!(registry.availability_cache.read().await.is_empty());
    }
}
