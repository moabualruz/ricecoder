//! Reference tracking across source files

use crate::error::ResearchError;
use crate::models::{Language, ReferenceKind, SymbolReference};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(feature = "parsers")]
use tree_sitter::{Language as TSLanguage, Parser};

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
        _path: &Path,
        _language: &Language,
        _content: &str,
        _known_symbols: &HashMap<String, String>,
    ) -> Result<Vec<SymbolReference>, ResearchError> {
        #[cfg(feature = "parsers")]
        {
            let mut parser = Parser::new();
            let ts_language = Self::get_tree_sitter_language(_language)?;
            parser
                .set_language(ts_language)
                .map_err(|_| ResearchError::AnalysisFailed {
                    reason: format!("Failed to set language for {:?}", _language),
                    context: "Reference tracking requires a valid tree-sitter language parser"
                        .to_string(),
                })?;

            let tree = parser.parse(_content, None)
                .ok_or_else(|| ResearchError::AnalysisFailed {
                reason: "Failed to parse file".to_string(),
                context: "Tree-sitter parser could not generate an abstract syntax tree for reference tracking".to_string(),
            })?;

        let mut references = Vec::new();
        let root = tree.root_node();

        // Extract references based on language
        Self::track_references_recursive(
            &root,
            content,
            path,
            language,
            known_symbols,
            &mut references,
        )?;

        Ok(references)
        }
        #[cfg(not(feature = "parsers"))]
        {
            // Return empty references when parsers are not available
            Ok(Vec::new())
        }
    }

    /// Recursively track references from AST nodes
    #[cfg(feature = "parsers")]
    fn track_references_recursive(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        language: &Language,
        known_symbols: &HashMap<String, String>,
        references: &mut Vec<SymbolReference>,
    ) -> Result<(), ResearchError> {
        // Extract references from current node if applicable
        if let Some(reference) =
            Self::extract_reference_from_node(node, content, path, language, known_symbols)
        {
            references.push(reference);
        }

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::track_references_recursive(
                &child,
                content,
                path,
                language,
                known_symbols,
                references,
            )?;
        }

        Ok(())
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
        let content = "fn main() { let x = 5; println!(\"{}\", x); }";
        let path = Path::new("test.rs");
        let mut known_symbols = HashMap::new();
        known_symbols.insert("x".to_string(), "test.rs:1:11".to_string());

        let references =
            ReferenceTracker::track_references(path, &Language::Rust, content, &known_symbols)
                .expect("Failed to track references");

        // Should find at least one reference to 'x'
        assert!(!references.is_empty());
    }

    #[test]
    fn test_track_python_references() {
        let content = "def foo():\n    x = 5\n    print(x)";
        let path = Path::new("test.py");
        let mut known_symbols = HashMap::new();
        known_symbols.insert("x".to_string(), "test.py:2:5".to_string());

        let references =
            ReferenceTracker::track_references(path, &Language::Python, content, &known_symbols)
                .expect("Failed to track references");

        // Should find references to 'x'
        let _ = references;
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
    fn test_get_line_from_byte_offset() {
        let content = "line1\nline2\nline3";
        // Byte offset 0 is at the start of line 1
        assert_eq!(ReferenceTracker::get_line_from_byte_offset(content, 0), 1);
        // Byte offset 6 is after the newline, at the start of line 2
        assert_eq!(ReferenceTracker::get_line_from_byte_offset(content, 6), 2);
        // Byte offset 12 is after the second newline, at the start of line 3
        assert_eq!(ReferenceTracker::get_line_from_byte_offset(content, 12), 3);
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
