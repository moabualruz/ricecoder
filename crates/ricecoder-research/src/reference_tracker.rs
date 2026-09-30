//! Reference tracking across source files

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[cfg(feature = "parsers")]
use ricecoder_parsers::{
    parser::{create_supports, ParserConfig},
    ASTNode, Language as ParserLanguage, NodeType,
};

use crate::{
    error::ResearchError,
    models::{Language, ReferenceKind, SymbolReference},
};

/// Tracks symbol references across files
#[derive(Debug)]
pub struct ReferenceTracker;

/// Result of reference tracking
#[derive(Debug, Clone)]
pub struct ReferenceTrackingResult {
    /// Map from symbol ID to all references to that symbol
    pub references_by_symbol: HashMap<String, Vec<SymbolReference>>,
    /// Map from file path to all references in that file
    pub references_by_file: HashMap<PathBuf, Vec<SymbolReference>>,
}

#[cfg(feature = "parsers")]
impl ReferenceTracker {
    /// Track symbol references in a source file
    ///
    /// # Arguments
    /// * `path` - Path to the source file
    /// * `language` - Programming language of the file
    /// * `content` - File content as string
    /// * `known_symbols` - Map of symbol names to their IDs
    ///
    /// # Returns
    /// A vector of symbol references found in the file
    pub fn track_references(
        path: &Path,
        language: &Language,
        content: &str,
        known_symbols: &HashMap<String, String>,
    ) -> Result<Vec<SymbolReference>, ResearchError> {
        let parser_language = match language {
            Language::Rust => ParserLanguage::Rust,
            Language::TypeScript => ParserLanguage::TypeScript,
            Language::Python => ParserLanguage::Python,
            Language::Go => ParserLanguage::Go,
            unsupported => {
                return Err(ResearchError::AnalysisFailed {
                    reason: format!("Reference tracking does not support {unsupported:?}"),
                    context: "Supported languages are Rust, TypeScript, Python, and Go".to_string(),
                });
            }
        };

        let support = create_supports()
            .into_iter()
            .find(|support| support.language() == parser_language)
            .ok_or_else(|| ResearchError::AnalysisFailed {
                reason: format!("No parser is registered for {language:?}"),
                context: "Reference tracking requires a supported tree-sitter parser".to_string(),
            })?;
        let tree = support
            .parse(content, &ParserConfig::default())
            .map_err(|error| ResearchError::AnalysisFailed {
                reason: error.to_string(),
                context: format!("Failed to parse {language:?} source for reference tracking"),
            })?;

        let mut references = Vec::new();
        Self::track_references_recursive(&tree.root, path, known_symbols, &mut references);
        Ok(references)
    }

    fn track_references_recursive(
        node: &ASTNode,
        path: &Path,
        known_symbols: &HashMap<String, String>,
        references: &mut Vec<SymbolReference>,
    ) {
        if matches!(
            &node.node_type,
            NodeType::Custom(kind)
                if matches!(kind.as_str(), "identifier" | "field_identifier" | "type_identifier")
        ) {
            if let Some(symbol_id) = known_symbols.get(node.text.trim()) {
                references.push(SymbolReference {
                    symbol_id: symbol_id.clone(),
                    file: path.to_path_buf(),
                    line: node.range.start.line + 1,
                    kind: ReferenceKind::Usage,
                });
            }
        }

        for child in &node.children {
            Self::track_references_recursive(child, path, known_symbols, references);
        }
    }
}

#[cfg(not(feature = "parsers"))]
impl ReferenceTracker {
    /// Track symbol references in a source file (disabled when parsers feature is not enabled)
    pub fn track_references(
        _path: &Path,
        _language: &Language,
        _content: &str,
        _known_symbols: &HashMap<String, String>,
    ) -> Result<Vec<SymbolReference>, ResearchError> {
        // Return empty references when parsers are not available
        Ok(Vec::new())
    }
}

#[cfg(all(test, feature = "parsers"))]
mod tests {
    use super::*;

    #[test]
    fn test_track_rust_references() {
        let content = "fn main() { println!(\"{}\", x); }";
        let path = Path::new("test.rs");
        let mut known_symbols = HashMap::new();
        known_symbols.insert("x".to_string(), "test.rs:1:11".to_string());

        let references =
            ReferenceTracker::track_references(path, &Language::Rust, content, &known_symbols)
                .expect("Failed to track references");

        assert_eq!(references.len(), 1);
        assert_eq!(references[0].symbol_id, "test.rs:1:11");
        assert_eq!(references[0].file.as_path(), path);
        assert_eq!(references[0].line, 1);
        assert_eq!(references[0].kind, ReferenceKind::Usage);
    }

    #[test]
    fn test_track_python_references() {
        let content = "def foo():\n    print(x)";
        let path = Path::new("test.py");
        let mut known_symbols = HashMap::new();
        known_symbols.insert("x".to_string(), "test.py:2:5".to_string());

        let references =
            ReferenceTracker::track_references(path, &Language::Python, content, &known_symbols)
                .expect("Failed to track references");

        assert_eq!(references.len(), 1);
        assert_eq!(references[0].symbol_id, "test.py:2:5");
        assert_eq!(references[0].file.as_path(), path);
        assert_eq!(references[0].line, 2);
        assert_eq!(references[0].kind, ReferenceKind::Usage);
    }

    #[test]
    fn test_track_references_empty_symbols() {
        let content = "fn main() { let x = 5; }";
        let path = Path::new("test.rs");
        let known_symbols = HashMap::new();

        let references =
            ReferenceTracker::track_references(path, &Language::Rust, content, &known_symbols)
                .expect("Failed to track references");

        // Should find no references since no symbols are known
        assert!(references.is_empty());
    }

    #[test]
    fn test_unsupported_language() {
        let content = "some code";
        let path = Path::new("test.unknown");
        let known_symbols = HashMap::new();
        let result = ReferenceTracker::track_references(
            path,
            &Language::Other("unknown".to_string()),
            content,
            &known_symbols,
        );

        assert!(result.is_err());
    }
}
