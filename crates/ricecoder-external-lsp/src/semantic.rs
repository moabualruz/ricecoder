//! Semantic feature integration (completion, diagnostics, hover, navigation)
//!
//! This module provides forwarding and merging of semantic features from external LSP servers.

use crate::client::LspConnection;
use crate::error::Result;
use crate::mapping::{CompletionMapper, DiagnosticsMapper, HoverMapper};
use crate::types::{CompletionMappingRules, HoverMappingRules, MergeConfig};
use ricecoder_completion::types::{CompletionContext, CompletionItem};
use ricecoder_lsp::types::{Diagnostic, Position, Range};
use serde_json::{json, Value};
use std::time::Duration;

/// Semantic feature forwarder and merger
pub struct SemanticFeatures {
    /// LSP connection for forwarding requests
    connection: std::sync::Arc<LspConnection>,
    /// Completion mapper for transforming LSP responses
    completion_mapper: CompletionMapper,
    /// Diagnostics mapper for transforming LSP responses
    #[allow(dead_code)]
    diagnostics_mapper: DiagnosticsMapper,
    /// Hover mapper for transforming LSP responses
    hover_mapper: HoverMapper,
    /// Merge configuration
    #[allow(dead_code)]
    merge_config: MergeConfig,
    /// Request timeout
    timeout: Duration,
}

impl SemanticFeatures {
    /// Create a new semantic features handler
    pub fn new(
        connection: std::sync::Arc<LspConnection>,
        completion_mapper: CompletionMapper,
        diagnostics_mapper: DiagnosticsMapper,
        hover_mapper: HoverMapper,
        merge_config: MergeConfig,
        timeout: Duration,
    ) -> Self {
        Self {
            connection,
            completion_mapper,
            diagnostics_mapper,
            hover_mapper,
            merge_config,
            timeout,
        }
    }

    /// Forward completion request to external LSP server
    ///
    /// # Arguments
    ///
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    /// * `context` - Completion context
    ///
    /// # Returns
    ///
    /// Vector of completion items from external LSP, or None if unavailable
    pub async fn forward_completion(
        &self,
        uri: &str,
        position: Position,
        _context: &CompletionContext,
    ) -> Result<Option<Vec<CompletionItem>>> {
        // Create textDocument/completion request
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": position.line,
                "character": position.character
            }
        });

        // Send request to LSP server
        let (_request, mut rx) = self
            .connection
            .create_tracked_request("textDocument/completion", Some(params), self.timeout)
            .await?;

        // Wait for response with timeout
        match tokio::time::timeout(self.timeout, &mut rx).await {
            Ok(Ok(result)) => {
                // Parse and transform response
                match result {
                    Ok(response) => {
                        // Transform LSP completion response to ricecoder CompletionItem
                        // Use default mapping rules for standard LSP response format
                        let rules = CompletionMappingRules {
                            items_path: "$.result.items".to_string(),
                            field_mappings: Default::default(),
                            transform: None,
                        };

                        let mapped_items = self.completion_mapper.map(&response, &rules)?;

                        // Convert mapped items to CompletionItem
                        let items = mapped_items
                            .into_iter()
                            .filter_map(|item| {
                                let label = item.get("label")?.as_str()?.to_string();
                                let insert_text = item
                                    .get("insertText")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(&label)
                                    .to_string();

                                Some(CompletionItem::new(
                                    label,
                                    ricecoder_completion::types::CompletionItemKind::Variable,
                                    insert_text,
                                ))
                            })
                            .collect();

                        Ok(Some(items))
                    }
                    Err(e) => {
                        // Log error but don't fail - will fall back to internal provider
                        tracing::warn!("LSP completion request failed: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(Err(_)) => {
                // Receiver was dropped
                Ok(None)
            }
            Err(_) => {
                // Timeout
                tracing::warn!("LSP completion request timed out");
                Ok(None)
            }
        }
    }

    /// Forward diagnostics request to external LSP server
    ///
    /// # Arguments
    ///
    /// * `_uri` - Document URI
    ///
    /// # Returns
    ///
    /// Vector of diagnostics from external LSP, or None if unavailable
    pub async fn forward_diagnostics(&self, _uri: &str) -> Result<Option<Vec<Diagnostic>>> {
        // Note: Diagnostics are typically pushed by the server via textDocument/publishDiagnostics
        // This method is for requesting diagnostics on demand if needed
        // For now, we return None as diagnostics are handled via notifications

        // In a full implementation, you might send a custom request or use a language-specific
        // extension to request diagnostics on demand
        Ok(None)
    }

    /// Forward hover request to external LSP server
    ///
    /// # Arguments
    ///
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    ///
    /// # Returns
    ///
    /// Hover information from external LSP, or None if unavailable
    pub async fn forward_hover(&self, uri: &str, position: Position) -> Result<Option<String>> {
        // Create textDocument/hover request
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": position.line,
                "character": position.character
            }
        });

        // Send request to LSP server
        let (_request, mut rx) = self
            .connection
            .create_tracked_request("textDocument/hover", Some(params), self.timeout)
            .await?;

        // Wait for response with timeout
        match tokio::time::timeout(self.timeout, &mut rx).await {
            Ok(Ok(result)) => {
                // Parse and transform response
                match result {
                    Ok(response) => {
                        // Transform LSP hover response to ricecoder format
                        let rules = HoverMappingRules {
                            content_path: "$.result.contents".to_string(),
                            field_mappings: Default::default(),
                            transform: None,
                        };

                        let hover_value = self.hover_mapper.map(&response, &rules)?;

                        // Convert Value to String
                        let hover_info = if let Some(s) = hover_value.as_str() {
                            s.to_string()
                        } else {
                            hover_value.to_string()
                        };

                        Ok(Some(hover_info))
                    }
                    Err(e) => {
                        // Log error but don't fail - will fall back to internal provider
                        tracing::warn!("LSP hover request failed: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(Err(_)) => {
                // Receiver was dropped
                Ok(None)
            }
            Err(_) => {
                // Timeout
                tracing::warn!("LSP hover request timed out");
                Ok(None)
            }
        }
    }

    /// Forward definition request to external LSP server
    ///
    /// # Arguments
    ///
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    ///
    /// # Returns
    ///
    /// Vector of definition locations from external LSP, or None if unavailable
    pub async fn forward_definition(
        &self,
        uri: &str,
        position: Position,
    ) -> Result<Option<Vec<(String, Range)>>> {
        // Create textDocument/definition request
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": position.line,
                "character": position.character
            }
        });

        // Send request to LSP server
        let (_request, mut rx) = self
            .connection
            .create_tracked_request("textDocument/definition", Some(params), self.timeout)
            .await?;

        // Wait for response with timeout
        match tokio::time::timeout(self.timeout, &mut rx).await {
            Ok(Ok(result)) => {
                // Parse response
                match result {
                    Ok(response) => {
                        // Parse definition locations from response
                        let locations = parse_locations(&response)?;
                        Ok(Some(locations))
                    }
                    Err(e) => {
                        // Log error but don't fail - will fall back to internal provider
                        tracing::warn!("LSP definition request failed: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(Err(_)) => {
                // Receiver was dropped
                Ok(None)
            }
            Err(_) => {
                // Timeout
                tracing::warn!("LSP definition request timed out");
                Ok(None)
            }
        }
    }

    /// Forward references request to external LSP server
    ///
    /// # Arguments
    ///
    /// * `uri` - Document URI
    /// * `position` - Cursor position
    ///
    /// # Returns
    ///
    /// Vector of reference locations from external LSP, or None if unavailable
    pub async fn forward_references(
        &self,
        uri: &str,
        position: Position,
    ) -> Result<Option<Vec<(String, Range)>>> {
        // Create textDocument/references request
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": {
                "line": position.line,
                "character": position.character
            },
            "context": {
                "includeDeclaration": true
            }
        });

        // Send request to LSP server
        let (_request, mut rx) = self
            .connection
            .create_tracked_request("textDocument/references", Some(params), self.timeout)
            .await?;

        // Wait for response with timeout
        match tokio::time::timeout(self.timeout, &mut rx).await {
            Ok(Ok(result)) => {
                // Parse response
                match result {
                    Ok(response) => {
                        // Parse reference locations from response
                        let locations = parse_locations(&response)?;
                        Ok(Some(locations))
                    }
                    Err(e) => {
                        // Log error but don't fail - will fall back to internal provider
                        tracing::warn!("LSP references request failed: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(Err(_)) => {
                // Receiver was dropped
                Ok(None)
            }
            Err(_) => {
                // Timeout
                tracing::warn!("LSP references request timed out");
                Ok(None)
            }
        }
    }
}

/// Parse location information from LSP response
fn parse_locations(response: &Value) -> Result<Vec<(String, Range)>> {
    let mut locations = Vec::new();

    // Handle both single location and array of locations
    let items = if response.is_array() {
        response.as_array().unwrap().clone()
    } else if response.is_object() {
        vec![response.clone()]
    } else {
        return Ok(locations);
    };

    for item in items {
        if let (Some(uri), Some(range)) =
            (item.get("uri").and_then(|v| v.as_str()), item.get("range"))
        {
            if let Some(parsed_range) = parse_range(range) {
                locations.push((uri.to_string(), parsed_range));
            }
        }
    }

    Ok(locations)
}

/// Parse range information from LSP response
fn parse_range(range: &Value) -> Option<Range> {
    let start = range.get("start")?;
    let end = range.get("end")?;

    let start_line = start.get("line")?.as_u64()? as u32;
    let start_char = start.get("character")?.as_u64()? as u32;
    let end_line = end.get("line")?.as_u64()? as u32;
    let end_char = end.get("character")?.as_u64()? as u32;

    Some(Range::new(
        Position::new(start_line, start_char),
        Position::new(end_line, end_char),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_location() {
        let response = json!({
            "uri": "file:///test.rs",
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 5}
            }
        });

        let locations = parse_locations(&response).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].0, "file:///test.rs");
        assert_eq!(locations[0].1.start.line, 0);
        assert_eq!(locations[0].1.start.character, 0);
        assert_eq!(locations[0].1.end.line, 0);
        assert_eq!(locations[0].1.end.character, 5);
    }

    #[test]
    fn test_parse_multiple_locations() {
        let response = json!([
            {
                "uri": "file:///test1.rs",
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 5}
                }
            },
            {
                "uri": "file:///test2.rs",
                "range": {
                    "start": {"line": 1, "character": 10},
                    "end": {"line": 1, "character": 15}
                }
            }
        ]);

        let locations = parse_locations(&response).unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].0, "file:///test1.rs");
        assert_eq!(locations[1].0, "file:///test2.rs");
    }

    #[test]
    fn test_parse_empty_response() {
        let response = json!([]);
        let locations = parse_locations(&response).unwrap();
        assert_eq!(locations.len(), 0);
    }

    #[test]
    fn test_parse_invalid_range() {
        let response = json!({
            "uri": "file:///test.rs",
            "range": {
                "start": {"line": "invalid"},
                "end": {"line": 0, "character": 5}
            }
        });

        let locations = parse_locations(&response).unwrap();
        assert_eq!(locations.len(), 0);
    }
}
