//! Semantic Analysis Engine
//!
//! This module provides semantic analysis capabilities for multiple programming languages.
//! It includes language detection, symbol extraction, and semantic information gathering.

use crate::types::{Language, Position, SemanticInfo, Symbol};
use std::path::Path;

pub mod adapters;
pub mod fallback_analyzer;
pub mod generic_analyzer;
pub mod python_analyzer;
pub mod rust_analyzer;
pub mod typescript_analyzer;

pub use adapters::{
    FallbackAnalyzerAdapter, PythonAnalyzerAdapter, RustAnalyzerAdapter, TypeScriptAnalyzerAdapter,
};
pub use fallback_analyzer::FallbackAnalyzer;
pub use generic_analyzer::GenericSemanticAnalyzer;
pub use python_analyzer::PythonAnalyzer;
pub use rust_analyzer::RustAnalyzer;
pub use typescript_analyzer::TypeScriptAnalyzer;

/// Error type for semantic analysis
#[derive(Debug, thiserror::Error)]
pub enum SemanticError {
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Analysis error
    #[error("Analysis error: {0}")]
    AnalysisError(String),

    /// Unsupported language
    #[error("Unsupported language: {0:?}")]
    UnsupportedLanguage(Language),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for semantic analysis
pub type SemanticResult<T> = Result<T, SemanticError>;

/// Trait for semantic analysis of code
pub trait SemanticAnalyzer: Send + Sync {
    /// Analyze code and extract semantic information
    fn analyze(&self, code: &str) -> SemanticResult<SemanticInfo>;

    /// Extract symbols from code
    fn extract_symbols(&self, code: &str) -> SemanticResult<Vec<Symbol>>;

    /// Get hover information at a specific position
    fn get_hover_info(&self, code: &str, position: Position) -> SemanticResult<Option<String>>;

    /// Get supported language
    fn language(&self) -> Language;
}

/// Language detection utilities
pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect language from file extension
    pub fn from_extension(path: &Path) -> Language {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(Language::from_extension)
            .unwrap_or(Language::Unknown)
    }

    /// Detect language from file content (shebang or imports)
    pub fn from_content(content: &str) -> Language {
        // Check for shebang
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                if first_line.contains("python") {
                    return Language::Python;
                } else if first_line.contains("node") || first_line.contains("ts-node") {
                    return Language::TypeScript;
                }
            }
        }

        // Check for language-specific imports/patterns
        // Check Rust first (most specific)
        if content.contains("use ") && content.contains("fn ") {
            return Language::Rust;
        }

        // Check Python (look for def keyword which is Python-specific)
        if content.contains("def ") {
            return Language::Python;
        }

        // Check TypeScript/JavaScript (look for export which is more specific)
        if content.contains("export ") {
            return Language::TypeScript;
        }

        // Check for import statements (but only if no def keyword)
        if content.contains("import ") {
            return Language::TypeScript;
        }

        Language::Unknown
    }

    /// Detect language from both extension and content
    pub fn detect(path: &Path, content: &str) -> Language {
        let from_ext = Self::from_extension(path);
        if from_ext != Language::Unknown {
            return from_ext;
        }
        Self::from_content(content)
    }
}

/// Factory for creating appropriate semantic analyzer
pub struct SemanticAnalyzerFactory;

impl SemanticAnalyzerFactory {
    /// Create a semantic analyzer for the given language
    pub fn create(language: Language) -> Box<dyn SemanticAnalyzer> {
        match language {
            Language::Rust => Box::new(RustAnalyzer::new()),
            Language::TypeScript => Box::new(TypeScriptAnalyzer::new()),
            Language::Python => Box::new(PythonAnalyzer::new()),
            Language::Unknown => Box::new(FallbackAnalyzer::new()),
        }
    }

    /// Create a semantic analyzer from file path and content
    pub fn from_file(path: &Path, content: &str) -> Box<dyn SemanticAnalyzer> {
        let language = LanguageDetector::detect(path, content);
        Self::create(language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection_from_extension() {
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            LanguageDetector::from_extension(Path::new("test.unknown")),
            Language::Unknown
        );
    }

    #[test]
    fn test_language_detection_from_content_shebang() {
        let python_shebang = "#!/usr/bin/env python\nprint('hello')";
        assert_eq!(
            LanguageDetector::from_content(python_shebang),
            Language::Python
        );

        let node_shebang = "#!/usr/bin/env node\nconsole.log('hello')";
        assert_eq!(
            LanguageDetector::from_content(node_shebang),
            Language::TypeScript
        );
    }

    #[test]
    fn test_language_detection_from_content_patterns() {
        let rust_code = "use std::io;\nfn main() {}";
        assert_eq!(LanguageDetector::from_content(rust_code), Language::Rust);

        let ts_code = "import { foo } from 'bar';\nexport const x = 1;";
        assert_eq!(
            LanguageDetector::from_content(ts_code),
            Language::TypeScript
        );

        let py_code = "import os\ndef hello():\n    pass";
        assert_eq!(LanguageDetector::from_content(py_code), Language::Python);
    }

    #[test]
    fn test_language_detection_combined() {
        let path = Path::new("test.rs");
        let content = "fn main() {}";
        assert_eq!(LanguageDetector::detect(path, content), Language::Rust);
    }

    #[test]
    fn test_semantic_analyzer_factory() {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);
        assert_eq!(analyzer.language(), Language::Rust);

        let analyzer = SemanticAnalyzerFactory::create(Language::TypeScript);
        assert_eq!(analyzer.language(), Language::TypeScript);

        let analyzer = SemanticAnalyzerFactory::create(Language::Python);
        assert_eq!(analyzer.language(), Language::Python);

        let analyzer = SemanticAnalyzerFactory::create(Language::Unknown);
        assert_eq!(analyzer.language(), Language::Unknown);
    }
}
