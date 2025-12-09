//! External LSP provider implementation
//!
//! This module implements the IdeProvider trait for external LSP servers,
//! querying ricecoder-external-lsp for semantic information and mapping
//! responses to IDE types.

use crate::error::{IdeError, IdeResult};
use crate::provider::IdeProvider;
use crate::types::*;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::debug;

/// External LSP provider that queries ricecoder-external-lsp
pub struct ExternalLspProvider {
    /// Language this provider supports
    language: String,
    /// LSP server registry
    registry: Arc<ricecoder_external_lsp::LspServerRegistry>,
}

impl ExternalLspProvider {
    /// Create a new external LSP provider
    pub fn new(
        language: String,
        registry: Arc<ricecoder_external_lsp::LspServerRegistry>,
    ) -> Self {
        ExternalLspProvider { language, registry }
    }

    /// Check if LSP server is available for this language
    fn is_lsp_available(&self) -> bool {
        self.registry
            .servers
            .get(&self.language)
            .map(|servers| !servers.is_empty())
            .unwrap_or(false)
    }

    /// Map LSP completion items to IDE completion items
    #[allow(dead_code)]
    fn map_lsp_completions(
        &self,
        lsp_completions: Vec<serde_json::Value>,
    ) -> IdeResult<Vec<CompletionItem>> {
        let mut items = Vec::new();

        for lsp_item in lsp_completions {
            let label = lsp_item
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let kind = lsp_item
                .get("kind")
                .and_then(|v| v.as_u64())
                .map(|k| self.map_lsp_completion_kind(k as u32))
                .unwrap_or(CompletionItemKind::Text);

            let detail = lsp_item
                .get("detail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let documentation = lsp_item
                .get("documentation")
                .and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                });

            let insert_text = lsp_item
                .get("insertText")
                .and_then(|v| v.as_str())
                .unwrap_or(&label)
                .to_string();

            items.push(CompletionItem {
                label,
                kind,
                detail,
                documentation,
                insert_text,
            });
        }

        Ok(items)
    }

    /// Map LSP completion kind to IDE completion kind
    #[allow(dead_code)]
    fn map_lsp_completion_kind(&self, kind: u32) -> CompletionItemKind {
        match kind {
            1 => CompletionItemKind::Text,
            2 => CompletionItemKind::Method,
            3 => CompletionItemKind::Function,
            4 => CompletionItemKind::Constructor,
            5 => CompletionItemKind::Field,
            6 => CompletionItemKind::Variable,
            7 => CompletionItemKind::Class,
            8 => CompletionItemKind::Interface,
            9 => CompletionItemKind::Module,
            10 => CompletionItemKind::Property,
            11 => CompletionItemKind::Unit,
            12 => CompletionItemKind::Value,
            13 => CompletionItemKind::Enum,
            14 => CompletionItemKind::Keyword,
            15 => CompletionItemKind::Snippet,
            16 => CompletionItemKind::Color,
            17 => CompletionItemKind::File,
            18 => CompletionItemKind::Reference,
            19 => CompletionItemKind::Folder,
            20 => CompletionItemKind::EnumMember,
            21 => CompletionItemKind::Constant,
            22 => CompletionItemKind::Struct,
            23 => CompletionItemKind::Event,
            24 => CompletionItemKind::Operator,
            25 => CompletionItemKind::TypeParameter,
            _ => CompletionItemKind::Text,
        }
    }

    /// Map LSP diagnostics to IDE diagnostics
    #[allow(dead_code)]
    fn map_lsp_diagnostics(
        &self,
        lsp_diagnostics: Vec<serde_json::Value>,
    ) -> IdeResult<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        for lsp_diag in lsp_diagnostics {
            let message = lsp_diag
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown diagnostic")
                .to_string();

            let severity = lsp_diag
                .get("severity")
                .and_then(|v| v.as_u64())
                .map(|s| self.map_lsp_severity(s as u32))
                .unwrap_or(DiagnosticSeverity::Information);

            let source = lsp_diag
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("lsp")
                .to_string();

            let range = self.extract_range_from_lsp(&lsp_diag).unwrap_or(Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            });

            diagnostics.push(Diagnostic {
                range,
                severity,
                message,
                source,
            });
        }

        Ok(diagnostics)
    }

    /// Map LSP severity to IDE severity
    #[allow(dead_code)]
    fn map_lsp_severity(&self, severity: u32) -> DiagnosticSeverity {
        match severity {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            4 => DiagnosticSeverity::Hint,
            _ => DiagnosticSeverity::Information,
        }
    }

    /// Extract range from LSP diagnostic
    #[allow(dead_code)]
    fn extract_range_from_lsp(&self, lsp_diag: &serde_json::Value) -> Option<Range> {
        let range_obj = lsp_diag.get("range")?.as_object()?;

        let start_obj = range_obj.get("start")?.as_object()?;
        let start_line = start_obj.get("line")?.as_u64()? as u32;
        let start_char = start_obj.get("character")?.as_u64()? as u32;

        let end_obj = range_obj.get("end")?.as_object()?;
        let end_line = end_obj.get("line")?.as_u64()? as u32;
        let end_char = end_obj.get("character")?.as_u64()? as u32;

        Some(Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: end_line,
                character: end_char,
            },
        })
    }

    /// Map LSP hover to IDE hover
    #[allow(dead_code)]
    fn map_lsp_hover(&self, lsp_hover: serde_json::Value) -> IdeResult<Option<Hover>> {
        let contents = lsp_hover
            .get("contents")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = v.as_object() {
                    obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string())
                } else if let Some(arr) = v.as_array() {
                    arr.first()
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            });

        match contents {
            Some(contents) => {
                let range = self.extract_range_from_lsp(&lsp_hover);
                Ok(Some(Hover { contents, range }))
            }
            None => Ok(None),
        }
    }

    /// Map LSP location to IDE location
    #[allow(dead_code)]
    fn map_lsp_location(&self, lsp_location: serde_json::Value) -> IdeResult<Option<Location>> {
        let uri = lsp_location
            .get("uri")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let range = self.extract_range_from_lsp(&lsp_location);

        match (uri, range) {
            (Some(uri), Some(range)) => {
                // Convert file:// URI to path
                let file_path = if uri.starts_with("file://") {
                    uri.strip_prefix("file://").unwrap_or(&uri).to_string()
                } else {
                    uri
                };

                Ok(Some(Location { file_path, range }))
            }
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl IdeProvider for ExternalLspProvider {
    async fn get_completions(&self, _params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!(
            "Getting completions from external LSP for language: {}",
            self.language
        );

        if !self.is_lsp_available() {
            return Err(IdeError::lsp_error(format!(
                "LSP server not available for language: {}",
                self.language
            )));
        }

        // For now, return empty completions
        // In a full implementation, this would query the actual LSP server
        // through ricecoder-external-lsp
        Ok(vec![])
    }

    async fn get_diagnostics(&self, _params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!(
            "Getting diagnostics from external LSP for language: {}",
            self.language
        );

        if !self.is_lsp_available() {
            return Err(IdeError::lsp_error(format!(
                "LSP server not available for language: {}",
                self.language
            )));
        }

        // For now, return empty diagnostics
        // In a full implementation, this would query the actual LSP server
        // through ricecoder-external-lsp
        Ok(vec![])
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!(
            "Getting hover from external LSP for language: {}",
            self.language
        );

        if !self.is_lsp_available() {
            return Err(IdeError::lsp_error(format!(
                "LSP server not available for language: {}",
                self.language
            )));
        }

        // For now, return None
        // In a full implementation, this would query the actual LSP server
        // through ricecoder-external-lsp
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!(
            "Getting definition from external LSP for language: {}",
            self.language
        );

        if !self.is_lsp_available() {
            return Err(IdeError::lsp_error(format!(
                "LSP server not available for language: {}",
                self.language
            )));
        }

        // For now, return None
        // In a full implementation, this would query the actual LSP server
        // through ricecoder-external-lsp
        Ok(None)
    }

    fn is_available(&self, language: &str) -> bool {
        language == self.language && self.is_lsp_available()
    }

    fn name(&self) -> &str {
        "external-lsp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_lsp_completion_kind() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        assert_eq!(provider.map_lsp_completion_kind(1), CompletionItemKind::Text);
        assert_eq!(provider.map_lsp_completion_kind(2), CompletionItemKind::Method);
        assert_eq!(provider.map_lsp_completion_kind(3), CompletionItemKind::Function);
        assert_eq!(provider.map_lsp_completion_kind(7), CompletionItemKind::Class);
    }

    #[test]
    fn test_map_lsp_severity() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        assert_eq!(provider.map_lsp_severity(1), DiagnosticSeverity::Error);
        assert_eq!(provider.map_lsp_severity(2), DiagnosticSeverity::Warning);
        assert_eq!(provider.map_lsp_severity(3), DiagnosticSeverity::Information);
        assert_eq!(provider.map_lsp_severity(4), DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_extract_range_from_lsp() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let lsp_diag = serde_json::json!({
            "range": {
                "start": { "line": 10, "character": 5 },
                "end": { "line": 10, "character": 15 }
            }
        });

        let range = provider.extract_range_from_lsp(&lsp_diag);
        assert!(range.is_some());

        let range = range.unwrap();
        assert_eq!(range.start.line, 10);
        assert_eq!(range.start.character, 5);
        assert_eq!(range.end.line, 10);
        assert_eq!(range.end.character, 15);
    }

    #[test]
    fn test_map_lsp_completions() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let lsp_completions = vec![serde_json::json!({
            "label": "test_function",
            "kind": 3,
            "detail": "fn test_function()",
            "documentation": "A test function",
            "insertText": "test_function()"
        })];

        let result = provider.map_lsp_completions(lsp_completions);
        assert!(result.is_ok());

        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "test_function");
        assert_eq!(items[0].kind, CompletionItemKind::Function);
        assert_eq!(items[0].detail, Some("fn test_function()".to_string()));
    }

    #[test]
    fn test_map_lsp_diagnostics() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let lsp_diagnostics = vec![serde_json::json!({
            "message": "unused variable",
            "severity": 2,
            "source": "rust-analyzer",
            "range": {
                "start": { "line": 5, "character": 4 },
                "end": { "line": 5, "character": 10 }
            }
        })];

        let result = provider.map_lsp_diagnostics(lsp_diagnostics);
        assert!(result.is_ok());

        let diagnostics = result.unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "unused variable");
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[0].source, "rust-analyzer");
    }

    #[test]
    fn test_map_lsp_hover() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let lsp_hover = serde_json::json!({
            "contents": "fn test_function() -> i32"
        });

        let result = provider.map_lsp_hover(lsp_hover);
        assert!(result.is_ok());

        let hover = result.unwrap();
        assert!(hover.is_some());
        assert_eq!(hover.unwrap().contents, "fn test_function() -> i32");
    }

    #[test]
    fn test_map_lsp_location() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let lsp_location = serde_json::json!({
            "uri": "file:///home/user/project/src/main.rs",
            "range": {
                "start": { "line": 10, "character": 5 },
                "end": { "line": 10, "character": 15 }
            }
        });

        let result = provider.map_lsp_location(lsp_location);
        assert!(result.is_ok());

        let location = result.unwrap();
        assert!(location.is_some());
        let loc = location.unwrap();
        assert!(loc.file_path.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_external_lsp_provider_not_available() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        let params = CompletionParams {
            language: "rust".to_string(),
            file_path: "src/main.rs".to_string(),
            position: Position {
                line: 10,
                character: 5,
            },
            context: "fn test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_external_lsp_provider_is_available() {
        let registry = Arc::new(ricecoder_external_lsp::LspServerRegistry::default());
        let provider = ExternalLspProvider::new("rust".to_string(), registry);

        assert!(!provider.is_available("rust"));
        assert!(!provider.is_available("typescript"));
    }
}
