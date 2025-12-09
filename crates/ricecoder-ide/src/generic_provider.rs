//! Generic text-based provider implementation
//!
//! This module implements the IdeProvider trait for generic text-based IDE features
//! that work for any language. This is the fallback provider when no language-specific
//! provider is available.

use crate::error::IdeResult;
use crate::provider::IdeProvider;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashSet;
use tracing::debug;

/// Generic text-based provider for any language
pub struct GenericProvider;

impl GenericProvider {
    /// Create a new generic provider
    pub fn new() -> Self {
        GenericProvider
    }

    /// Extract words from context
    fn extract_words(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty())
            .map(|w| w.to_string())
            .collect()
    }

    /// Get unique words from text
    fn get_unique_words(text: &str) -> Vec<String> {
        let words = Self::extract_words(text);
        let mut unique: Vec<String> = words.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique.sort();
        unique
    }

    /// Check if text looks like a syntax error
    fn has_syntax_error(source: &str) -> bool {
        let open_braces = source.matches('{').count();
        let close_braces = source.matches('}').count();
        let open_parens = source.matches('(').count();
        let close_parens = source.matches(')').count();
        let open_brackets = source.matches('[').count();
        let close_brackets = source.matches(']').count();

        open_braces != close_braces || open_parens != close_parens || open_brackets != close_brackets
    }
}

#[async_trait]
impl IdeProvider for GenericProvider {
    async fn get_completions(&self, params: &CompletionParams) -> IdeResult<Vec<CompletionItem>> {
        debug!("Getting completions from generic text-based provider");

        let mut completions = Vec::new();

        // Extract words from context and suggest them as completions
        let words = Self::get_unique_words(&params.context);

        for word in words.iter().take(10) {
            // Limit to 10 suggestions
            completions.push(CompletionItem {
                label: word.clone(),
                kind: CompletionItemKind::Text,
                detail: Some("word suggestion".to_string()),
                documentation: None,
                insert_text: word.clone(),
            });
        }

        Ok(completions)
    }

    async fn get_diagnostics(&self, params: &DiagnosticsParams) -> IdeResult<Vec<Diagnostic>> {
        debug!("Getting diagnostics from generic text-based provider");

        let mut diagnostics = Vec::new();

        // Check for basic syntax errors
        if Self::has_syntax_error(&params.source) {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 10,
                    },
                },
                severity: DiagnosticSeverity::Error,
                message: "Mismatched brackets or parentheses".to_string(),
                source: "generic".to_string(),
            });
        }

        // Check for trailing whitespace
        for (line_num, line) in params.source.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: (line.len() - 1) as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: DiagnosticSeverity::Hint,
                    message: "Trailing whitespace".to_string(),
                    source: "generic".to_string(),
                });
            }
        }

        Ok(diagnostics)
    }

    async fn get_hover(&self, _params: &HoverParams) -> IdeResult<Option<Hover>> {
        debug!("Getting hover from generic text-based provider");
        // Generic provider doesn't provide hover information
        Ok(None)
    }

    async fn get_definition(&self, _params: &DefinitionParams) -> IdeResult<Option<Location>> {
        debug!("Getting definition from generic text-based provider");
        // Generic provider doesn't provide definition information
        Ok(None)
    }

    fn is_available(&self, _language: &str) -> bool {
        // Generic provider is always available
        true
    }

    fn name(&self) -> &str {
        "generic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_words() {
        let text = "hello world test_var";
        let words = GenericProvider::extract_words(text);
        assert_eq!(words.len(), 3);
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
        assert!(words.contains(&"test_var".to_string()));
    }

    #[test]
    fn test_get_unique_words() {
        let text = "hello world hello test";
        let words = GenericProvider::get_unique_words(text);
        assert_eq!(words.len(), 3);
        assert!(words.contains(&"hello".to_string()));
        assert!(words.contains(&"world".to_string()));
        assert!(words.contains(&"test".to_string()));
    }

    #[test]
    fn test_has_syntax_error_balanced() {
        let source = "{ ( [ ] ) }";
        assert!(!GenericProvider::has_syntax_error(source));
    }

    #[test]
    fn test_has_syntax_error_unbalanced_braces() {
        let source = "{ ( [ ] ) ";
        assert!(GenericProvider::has_syntax_error(source));
    }

    #[test]
    fn test_has_syntax_error_unbalanced_parens() {
        let source = "{ ( [ ] }";
        assert!(GenericProvider::has_syntax_error(source));
    }

    #[tokio::test]
    async fn test_generic_provider_completions() {
        let provider = GenericProvider;
        let params = CompletionParams {
            language: "unknown".to_string(),
            file_path: "file.txt".to_string(),
            position: Position {
                line: 0,
                character: 0,
            },
            context: "hello world test".to_string(),
        };

        let result = provider.get_completions(&params).await;
        assert!(result.is_ok());
        let completions = result.unwrap();
        assert!(completions.len() > 0);
        assert!(completions.iter().any(|c| c.label == "hello"));
    }

    #[tokio::test]
    async fn test_generic_provider_diagnostics_syntax_error() {
        let provider = GenericProvider;
        let params = DiagnosticsParams {
            language: "unknown".to_string(),
            file_path: "file.txt".to_string(),
            source: "{ ( [ ] ) ".to_string(),
        };

        let result = provider.get_diagnostics(&params).await;
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(diagnostics.len() > 0);
        assert!(diagnostics[0].severity == DiagnosticSeverity::Error);
    }

    #[tokio::test]
    async fn test_generic_provider_diagnostics_trailing_whitespace() {
        let provider = GenericProvider;
        let params = DiagnosticsParams {
            language: "unknown".to_string(),
            file_path: "file.txt".to_string(),
            source: "hello world  \ntest".to_string(),
        };

        let result = provider.get_diagnostics(&params).await;
        assert!(result.is_ok());
        let diagnostics = result.unwrap();
        assert!(diagnostics.len() > 0);
        assert!(diagnostics.iter().any(|d| d.message.contains("Trailing whitespace")));
    }

    #[tokio::test]
    async fn test_generic_provider_is_available() {
        let provider = GenericProvider;
        assert!(provider.is_available("rust"));
        assert!(provider.is_available("typescript"));
        assert!(provider.is_available("unknown"));
    }

    #[tokio::test]
    async fn test_generic_provider_hover() {
        let provider = GenericProvider;
        let params = HoverParams {
            language: "unknown".to_string(),
            file_path: "file.txt".to_string(),
            position: Position {
                line: 0,
                character: 0,
            },
        };

        let result = provider.get_hover(&params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_generic_provider_definition() {
        let provider = GenericProvider;
        let params = DefinitionParams {
            language: "unknown".to_string(),
            file_path: "file.txt".to_string(),
            position: Position {
                line: 0,
                character: 0,
            },
        };

        let result = provider.get_definition(&params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
