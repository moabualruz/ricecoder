//! Diagnostics Engine Module
//!
//! This module provides the diagnostics engine for analyzing code and generating
//! diagnostics (errors, warnings, hints) for identified issues.
//!
//! # Architecture
//!
//! The diagnostics engine is organized into:
//! - `DiagnosticsEngine`: Main trait for generating diagnostics
//! - Language-specific rule modules: `rust_rules`, `typescript_rules`, `python_rules`
//! - `Diagnostic` types: Error, warning, and hint severity levels
//!
//! # Example
//!
//! ```ignore
//! use ricecoder_lsp::diagnostics::DiagnosticsEngine;
//! use ricecoder_lsp::types::Language;
//!
//! let engine = DefaultDiagnosticsEngine::new();
//! let diagnostics = engine.generate_diagnostics(code, Language::Rust)?;
//! ```

pub mod adapters;
pub mod generic_engine;
pub mod python_rules;
pub mod rust_rules;
pub mod typescript_rules;

pub use adapters::{
    PythonDiagnosticsAdapter, RustDiagnosticsAdapter, TypeScriptDiagnosticsAdapter,
};
pub use generic_engine::GenericDiagnosticsEngine;

use crate::types::{Diagnostic, Language, Range};
use std::error::Error;
use std::fmt;

/// Error type for diagnostics operations
#[derive(Debug, Clone)]
pub enum DiagnosticsError {
    /// Analysis failed
    AnalysisFailed(String),
    /// Invalid input
    InvalidInput(String),
    /// Unsupported language
    UnsupportedLanguage(String),
}

impl fmt::Display for DiagnosticsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticsError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            DiagnosticsError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DiagnosticsError::UnsupportedLanguage(lang) => {
                write!(f, "Unsupported language: {}", lang)
            }
        }
    }
}

impl Error for DiagnosticsError {}

/// Result type for diagnostics operations
pub type DiagnosticsResult<T> = Result<T, DiagnosticsError>;

/// Trait for generating diagnostics from code
pub trait DiagnosticsEngine: Send + Sync {
    /// Generate diagnostics for the given code
    fn generate_diagnostics(
        &self,
        code: &str,
        language: Language,
    ) -> DiagnosticsResult<Vec<Diagnostic>>;

    /// Generate diagnostics for a specific range
    fn generate_diagnostics_for_range(
        &self,
        code: &str,
        language: Language,
        range: Range,
    ) -> DiagnosticsResult<Vec<Diagnostic>>;
}

/// Default diagnostics engine implementation
pub struct DefaultDiagnosticsEngine;

impl DefaultDiagnosticsEngine {
    /// Create a new diagnostics engine
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultDiagnosticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsEngine for DefaultDiagnosticsEngine {
    fn generate_diagnostics(
        &self,
        code: &str,
        language: Language,
    ) -> DiagnosticsResult<Vec<Diagnostic>> {
        if code.is_empty() {
            return Ok(Vec::new());
        }

        match language {
            Language::Rust => rust_rules::generate_rust_diagnostics(code),
            Language::TypeScript => typescript_rules::generate_typescript_diagnostics(code),
            Language::Python => python_rules::generate_python_diagnostics(code),
            Language::Unknown => {
                // Gracefully degrade for unknown languages
                Ok(Vec::new())
            }
        }
    }

    fn generate_diagnostics_for_range(
        &self,
        code: &str,
        language: Language,
        range: Range,
    ) -> DiagnosticsResult<Vec<Diagnostic>> {
        let all_diagnostics = self.generate_diagnostics(code, language)?;

        // Filter diagnostics that fall within the specified range
        let filtered = all_diagnostics
            .into_iter()
            .filter(|diag| {
                diag.range.start.line >= range.start.line && diag.range.end.line <= range.end.line
            })
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_engine_empty_code() {
        let engine = DefaultDiagnosticsEngine::new();
        let result = engine.generate_diagnostics("", Language::Rust);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_diagnostics_engine_unknown_language() {
        let engine = DefaultDiagnosticsEngine::new();
        let result = engine.generate_diagnostics("some code", Language::Unknown);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_diagnostics_error_display() {
        let err = DiagnosticsError::AnalysisFailed("test error".to_string());
        assert_eq!(err.to_string(), "Analysis failed: test error");
    }
}
