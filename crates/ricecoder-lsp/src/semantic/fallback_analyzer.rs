//! Fallback semantic analyzer for unsupported languages
//!
//! Provides graceful degradation for languages that are not explicitly supported.
//! Returns empty results and logs warnings when unsupported languages are encountered.

use super::{SemanticAnalyzer, SemanticResult};
use crate::types::{Language, Position, SemanticInfo, Symbol};

/// Fallback analyzer for unsupported languages
pub struct FallbackAnalyzer;

impl FallbackAnalyzer {
    /// Create a new fallback analyzer
    pub fn new() -> Self {
        Self
    }
}

impl Default for FallbackAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer for FallbackAnalyzer {
    fn analyze(&self, _code: &str) -> SemanticResult<SemanticInfo> {
        // Log warning about unsupported language
        tracing::warn!("Analyzing code with unsupported language; returning empty semantic info");

        // Return empty semantic information
        Ok(SemanticInfo {
            symbols: Vec::new(),
            imports: Vec::new(),
            definitions: Vec::new(),
            references: Vec::new(),
        })
    }

    fn extract_symbols(&self, _code: &str) -> SemanticResult<Vec<Symbol>> {
        // Log warning about unsupported language
        tracing::warn!("Extracting symbols from unsupported language; returning empty symbol list");

        // Return empty symbol list
        Ok(Vec::new())
    }

    fn get_hover_info(&self, _code: &str, _position: Position) -> SemanticResult<Option<String>> {
        // Log warning about unsupported language
        tracing::warn!("Getting hover info for unsupported language; returning None");

        // Return no hover information
        Ok(None)
    }

    fn language(&self) -> Language {
        Language::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_returns_empty() {
        let analyzer = FallbackAnalyzer::new();
        let code = "some code in unknown language";
        let info = analyzer.analyze(code).unwrap();
        assert!(info.symbols.is_empty());
        assert!(info.imports.is_empty());
        assert!(info.definitions.is_empty());
        assert!(info.references.is_empty());
    }

    #[test]
    fn test_extract_symbols_returns_empty() {
        let analyzer = FallbackAnalyzer::new();
        let code = "some code in unknown language";
        let symbols = analyzer.extract_symbols(code).unwrap();
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_get_hover_info_returns_none() {
        let analyzer = FallbackAnalyzer::new();
        let code = "some code in unknown language";
        let position = Position::new(0, 0);
        let hover = analyzer.get_hover_info(code, position).unwrap();
        assert!(hover.is_none());
    }

    #[test]
    fn test_language() {
        let analyzer = FallbackAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Unknown);
    }

    #[test]
    fn test_graceful_degradation() {
        let analyzer = FallbackAnalyzer::new();
        let code = "<?php echo 'hello'; ?>";

        // Should not crash, just return empty results
        let info = analyzer.analyze(code).unwrap();
        assert!(info.symbols.is_empty());

        let symbols = analyzer.extract_symbols(code).unwrap();
        assert!(symbols.is_empty());

        let hover = analyzer.get_hover_info(code, Position::new(0, 0)).unwrap();
        assert!(hover.is_none());
    }
}
