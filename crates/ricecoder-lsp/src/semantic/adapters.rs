//! Adapter implementations for existing analyzers to implement SemanticAnalyzerProvider trait
//!
//! This module provides adapter wrappers that allow existing language-specific analyzers
//! to be used as pluggable providers in the configuration-driven architecture.

use crate::providers::{SemanticAnalyzerProvider, ProviderResult};
use crate::types::{Position, SemanticInfo, Symbol};
use super::{RustAnalyzer, TypeScriptAnalyzer, PythonAnalyzer, FallbackAnalyzer, SemanticAnalyzer};

/// Adapter for Rust semantic analyzer
pub struct RustAnalyzerAdapter {
    analyzer: RustAnalyzer,
}

impl RustAnalyzerAdapter {
    /// Create a new Rust analyzer adapter
    pub fn new() -> Self {
        Self {
            analyzer: RustAnalyzer::new(),
        }
    }
}

impl Default for RustAnalyzerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzerProvider for RustAnalyzerAdapter {
    fn language(&self) -> &str {
        "rust"
    }

    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo> {
        self.analyzer
            .analyze(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>> {
        self.analyzer
            .extract_symbols(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>> {
        self.analyzer
            .get_hover_info(code, position)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }
}

/// Adapter for TypeScript semantic analyzer
pub struct TypeScriptAnalyzerAdapter {
    analyzer: TypeScriptAnalyzer,
}

impl TypeScriptAnalyzerAdapter {
    /// Create a new TypeScript analyzer adapter
    pub fn new() -> Self {
        Self {
            analyzer: TypeScriptAnalyzer::new(),
        }
    }
}

impl Default for TypeScriptAnalyzerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzerProvider for TypeScriptAnalyzerAdapter {
    fn language(&self) -> &str {
        "typescript"
    }

    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo> {
        self.analyzer
            .analyze(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>> {
        self.analyzer
            .extract_symbols(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>> {
        self.analyzer
            .get_hover_info(code, position)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }
}

/// Adapter for Python semantic analyzer
pub struct PythonAnalyzerAdapter {
    analyzer: PythonAnalyzer,
}

impl PythonAnalyzerAdapter {
    /// Create a new Python analyzer adapter
    pub fn new() -> Self {
        Self {
            analyzer: PythonAnalyzer::new(),
        }
    }
}

impl Default for PythonAnalyzerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzerProvider for PythonAnalyzerAdapter {
    fn language(&self) -> &str {
        "python"
    }

    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo> {
        self.analyzer
            .analyze(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>> {
        self.analyzer
            .extract_symbols(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>> {
        self.analyzer
            .get_hover_info(code, position)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }
}

/// Adapter for fallback semantic analyzer
pub struct FallbackAnalyzerAdapter {
    analyzer: FallbackAnalyzer,
}

impl FallbackAnalyzerAdapter {
    /// Create a new fallback analyzer adapter
    pub fn new() -> Self {
        Self {
            analyzer: FallbackAnalyzer::new(),
        }
    }
}

impl Default for FallbackAnalyzerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzerProvider for FallbackAnalyzerAdapter {
    fn language(&self) -> &str {
        "unknown"
    }

    fn analyze(&self, code: &str) -> ProviderResult<SemanticInfo> {
        self.analyzer
            .analyze(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn extract_symbols(&self, code: &str) -> ProviderResult<Vec<Symbol>> {
        self.analyzer
            .extract_symbols(code)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }

    fn get_hover_info(&self, code: &str, position: Position) -> ProviderResult<Option<String>> {
        self.analyzer
            .get_hover_info(code, position)
            .map_err(|e| crate::providers::ProviderError::Error(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_adapter_language() {
        let adapter = RustAnalyzerAdapter::new();
        assert_eq!(adapter.language(), "rust");
    }

    #[test]
    fn test_typescript_adapter_language() {
        let adapter = TypeScriptAnalyzerAdapter::new();
        assert_eq!(adapter.language(), "typescript");
    }

    #[test]
    fn test_python_adapter_language() {
        let adapter = PythonAnalyzerAdapter::new();
        assert_eq!(adapter.language(), "python");
    }

    #[test]
    fn test_fallback_adapter_language() {
        let adapter = FallbackAnalyzerAdapter::new();
        assert_eq!(adapter.language(), "unknown");
    }
}
