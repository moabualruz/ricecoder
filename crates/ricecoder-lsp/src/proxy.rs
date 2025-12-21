//! LSP Proxy for external LSP server integration
//!
//! This module provides a proxy layer that routes requests to external LSP servers
//! while maintaining backward compatibility with internal providers.
//!
//! # Architecture
//!
//! The proxy acts as a middleware between ricecoder's LSP server and external LSP servers:
//!
//! ```text
//! ricecoder-lsp (LspServer)
//!     ↓
//! LspProxy (routes requests)
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

use crate::types::{LspError, LspResult};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// LSP Proxy for routing requests to external LSP servers
///
/// This proxy maintains backward compatibility while enabling external LSP integration.
pub struct LspProxy {
    /// External LSP client pool (optional)
    external_lsp: Option<Arc<dyn ExternalLspClient>>,
    /// Enable fallback to internal providers
    enable_fallback: bool,
}

/// Trait for external LSP client
pub trait ExternalLspClient: Send + Sync {
    /// Forward completion request to external LSP
    fn forward_completion(
        &self,
        language: &str,
        uri: &str,
        position: Value,
        context: Value,
    ) -> LspResult<Option<Value>>;

    /// Forward diagnostics request to external LSP
    fn forward_diagnostics(&self, language: &str, uri: &str) -> LspResult<Option<Value>>;

    /// Forward hover request to external LSP
    fn forward_hover(&self, language: &str, uri: &str, position: Value)
        -> LspResult<Option<Value>>;

    /// Forward definition request to external LSP
    fn forward_definition(
        &self,
        language: &str,
        uri: &str,
        position: Value,
    ) -> LspResult<Option<Value>>;

    /// Forward references request to external LSP
    fn forward_references(
        &self,
        language: &str,
        uri: &str,
        position: Value,
    ) -> LspResult<Option<Value>>;

    /// Check if external LSP is available for language
    fn is_available(&self, language: &str) -> bool;
}

impl LspProxy {
    /// Create a new LSP proxy without external LSP
    pub fn new() -> Self {
        Self {
            external_lsp: None,
            enable_fallback: true,
        }
    }

    /// Create a new LSP proxy with external LSP client
    pub fn with_external_lsp(
        external_lsp: Arc<dyn ExternalLspClient>,
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
    /// * `position` - Cursor position
    /// * `context` - Completion context
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Completion items from external LSP or fallback provider
    pub fn route_completion<F>(
        &self,
        language: &str,
        uri: &str,
        position: Value,
        context: Value,
        fallback_fn: F,
    ) -> LspResult<Value>
    where
        F: FnOnce() -> LspResult<Value>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!(
                    "Routing completion to external LSP for language: {}",
                    language
                );
                match external_lsp.forward_completion(language, uri, position, context) {
                    Ok(Some(result)) => {
                        info!("Received completion from external LSP");
                        return Ok(result);
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
            fallback_fn()
        } else {
            Err(LspError::InternalError(
                "External LSP unavailable and fallback disabled".to_string(),
            ))
        }
    }

    /// Route diagnostics request
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Diagnostics from external LSP or fallback provider
    pub fn route_diagnostics<F>(
        &self,
        language: &str,
        uri: &str,
        fallback_fn: F,
    ) -> LspResult<Value>
    where
        F: FnOnce() -> LspResult<Value>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!(
                    "Routing diagnostics to external LSP for language: {}",
                    language
                );
                match external_lsp.forward_diagnostics(language, uri) {
                    Ok(Some(result)) => {
                        info!("Received diagnostics from external LSP");
                        return Ok(result);
                    }
                    Ok(None) => {
                        debug!("External LSP returned no diagnostics");
                    }
                    Err(e) => {
                        warn!("External LSP diagnostics failed: {}", e);
                        if !self.enable_fallback {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Fall back to internal provider
        if self.enable_fallback {
            debug!("Falling back to internal diagnostics provider");
            fallback_fn()
        } else {
            Err(LspError::InternalError(
                "External LSP unavailable and fallback disabled".to_string(),
            ))
        }
    }

    /// Route hover request
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Hover information from external LSP or fallback provider
    pub fn route_hover<F>(
        &self,
        language: &str,
        uri: &str,
        position: Value,
        fallback_fn: F,
    ) -> LspResult<Value>
    where
        F: FnOnce() -> LspResult<Value>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!("Routing hover to external LSP for language: {}", language);
                match external_lsp.forward_hover(language, uri, position) {
                    Ok(Some(result)) => {
                        info!("Received hover from external LSP");
                        return Ok(result);
                    }
                    Ok(None) => {
                        debug!("External LSP returned no hover information");
                    }
                    Err(e) => {
                        warn!("External LSP hover failed: {}", e);
                        if !self.enable_fallback {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Fall back to internal provider
        if self.enable_fallback {
            debug!("Falling back to internal hover provider");
            fallback_fn()
        } else {
            Err(LspError::InternalError(
                "External LSP unavailable and fallback disabled".to_string(),
            ))
        }
    }

    /// Route definition request
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Definition locations from external LSP or fallback provider
    pub fn route_definition<F>(
        &self,
        language: &str,
        uri: &str,
        position: Value,
        fallback_fn: F,
    ) -> LspResult<Value>
    where
        F: FnOnce() -> LspResult<Value>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!(
                    "Routing definition to external LSP for language: {}",
                    language
                );
                match external_lsp.forward_definition(language, uri, position) {
                    Ok(Some(result)) => {
                        info!("Received definition from external LSP");
                        return Ok(result);
                    }
                    Ok(None) => {
                        debug!("External LSP returned no definition");
                    }
                    Err(e) => {
                        warn!("External LSP definition failed: {}", e);
                        if !self.enable_fallback {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Fall back to internal provider
        if self.enable_fallback {
            debug!("Falling back to internal definition provider");
            fallback_fn()
        } else {
            Err(LspError::InternalError(
                "External LSP unavailable and fallback disabled".to_string(),
            ))
        }
    }

    /// Route references request
    ///
    /// # Arguments
    ///
    /// * `language` - Programming language
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    /// * `fallback_fn` - Fallback function for internal provider
    ///
    /// # Returns
    ///
    /// Reference locations from external LSP or fallback provider
    pub fn route_references<F>(
        &self,
        language: &str,
        uri: &str,
        position: Value,
        fallback_fn: F,
    ) -> LspResult<Value>
    where
        F: FnOnce() -> LspResult<Value>,
    {
        // Try external LSP first
        if let Some(external_lsp) = &self.external_lsp {
            if external_lsp.is_available(language) {
                debug!(
                    "Routing references to external LSP for language: {}",
                    language
                );
                match external_lsp.forward_references(language, uri, position) {
                    Ok(Some(result)) => {
                        info!("Received references from external LSP");
                        return Ok(result);
                    }
                    Ok(None) => {
                        debug!("External LSP returned no references");
                    }
                    Err(e) => {
                        warn!("External LSP references failed: {}", e);
                        if !self.enable_fallback {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Fall back to internal provider
        if self.enable_fallback {
            debug!("Falling back to internal references provider");
            fallback_fn()
        } else {
            Err(LspError::InternalError(
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

impl Default for LspProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_creation() {
        let proxy = LspProxy::new();
        assert!(!proxy.is_external_lsp_available("rust"));
    }

    #[test]
    fn test_proxy_fallback() {
        let proxy = LspProxy::new();
        let result =
            proxy.route_completion("rust", "file:///test.rs", Value::Null, Value::Null, || {
                Ok(Value::Array(vec![]))
            });
        assert!(result.is_ok());
    }
}
