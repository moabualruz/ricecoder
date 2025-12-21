//! External LSP proxy for completion engine
//!
//! This module provides a proxy layer that routes completion requests to external LSP servers
//! while maintaining backward compatibility with internal providers.
//!
//! # Architecture
//!
//! The proxy acts as a middleware between the completion engine and external LSP servers:
//!
//! ```text
//! CompletionEngine
//!     ↓
//! ExternalLspCompletionProxy (routes requests)
//!     ↓
//! External LSP Servers (rust-analyzer, tsserver, pylsp, etc.)
//! ```
//!
//! # Request Routing
//!
//! Requests are routed based on language:
//! - If external LSP is configured for the language → forward to external LSP
//! - If external LSP is unavailable → fall back to internal provider
//! - If no external LSP configured → use internal provider

use crate::types::{CompletionItem, CompletionResult, Position};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Trait for external LSP completion client
#[async_trait]
pub trait ExternalLspCompletionClient: Send + Sync {
    /// Forward completion request to external LSP
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `code` - Source code
    /// * `position` - Cursor position
    ///
    /// # Returns
    ///
    /// Completion items from external LSP, or None if unavailable
    async fn forward_completion(
        &self,
        language: &str,
        uri: &str,
        code: &str,
        position: Position,
    ) -> CompletionResult<Option<Vec<CompletionItem>>>;

    /// Check if external LSP is available for language
    fn is_available(&self, language: &str) -> bool;
}

/// External LSP completion proxy
///
/// This proxy maintains backward compatibility while enabling external LSP integration.
pub struct ExternalLspCompletionProxy {
    /// External LSP client (optional)
    external_lsp: Option<Arc<dyn ExternalLspCompletionClient>>,
    /// Enable fallback to internal providers
    enable_fallback: bool,
}

impl ExternalLspCompletionProxy {
    /// Create a new completion proxy without external LSP
    pub fn new() -> Self {
        Self {
            external_lsp: None,
            enable_fallback: true,
        }
    }

    /// Create a new completion proxy with external LSP client
    pub fn with_external_lsp(
        external_lsp: Arc<dyn ExternalLspCompletionClient>,
        enable_fallback: bool,
    ) -> Self {
        Self {
            external_lsp: Some(external_lsp),
            enable_fallback,
        }
    }

    /// Route completion request
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `code` - Source code
    /// * `position` - Cursor position
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Completion items from external LSP or fallback provider
    pub async fn route_completion<F>(
        &self,
        language: &str,
        uri: &str,
        code: &str,
        position: Position,
        fallback_fn: F,
    ) -> CompletionResult<Vec<CompletionItem>>
    where
        F: std::future::Future<Output = CompletionResult<Vec<CompletionItem>>>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!(
                    "Routing completion to external LSP for language: {}",
                    language
                );
                match external_lsp
                    .forward_completion(language, uri, code, position)
                    .await
                {
                    Ok(Some(items)) => {
                        info!("Received {} completions from external LSP", items.len());
                        return Ok(items);
                    }
                    Ok(None) => {
                        debug!("External LSP returned no completions");
                    }
                    Err(e) => {
                        warn!("External LSP completion failed: {}", e);
                        if !self.enable_fallback {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Fall back to internal provider
        if self.enable_fallback {
            debug!("Falling back to internal completion provider");
            fallback_fn.await
        } else {
            Err(crate::types::CompletionError::InternalError(
                "External LSP unavailable and fallback disabled".to_string(),
            ))
        }
    }

    /// Check if external LSP is available for language
    pub fn is_external_lsp_available(&self, language: &str) -> bool {
        self.external_lsp
            .as_ref()
            .map(|lsp| lsp.is_available(language))
            .unwrap_or(false)
    }
}

impl Default for ExternalLspCompletionProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_creation() {
        let proxy = ExternalLspCompletionProxy::new();
        assert!(!proxy.is_external_lsp_available("rust"));
    }

    #[tokio::test]
    async fn test_proxy_fallback() {
        let proxy = ExternalLspCompletionProxy::new();
        let result = proxy
            .route_completion(
                "rust",
                "file:///test.rs",
                "fn main() {}",
                Position::new(0, 0),
                async { Ok(vec![]) },
            )
            .await;
        assert!(result.is_ok());
    }
}
