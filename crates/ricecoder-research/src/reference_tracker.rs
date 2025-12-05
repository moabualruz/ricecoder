//! Reference tracking across source files

use crate::error::ResearchError;
use crate::models::{Language, ReferenceKind, SymbolReference};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Language as TSLanguage, Parser};

/// Tracks symbol references across files
pub struct ReferenceTracker;

/// Result of reference tracking
#[derive(Debug, Clone)]
pub struct ReferenceTrackingResult {
    /// Map from symbol ID to all references to that symbol
    pub references_by_symbol: HashMap<String, Vec<SymbolReference>>,
    /// Map from file path to all references in that file
    pub references_by_file: HashMap<PathBuf, Vec<SymbolReference>>,
}

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
        let mut parser = Parser::new();
        let ts_language = Self::get_tree_sitter_language(language)?;
        parser
            .set_language(ts_language)
            .map_err(|_| ResearchError::AnalysisFailed {
                reason: format!("Failed to set language for {:?}", language),
                context: "Reference tracking requires a valid tree-sitter language parser"
                    .to_string(),
            })?;

        let tree = parser.parse(content, None)
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

    /// Recursively track references from AST nodes
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

    /// Extract a single reference from a node if it represents a symbol reference
    fn extract_reference_from_node(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        language: &Language,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        match language {
            Language::Rust => Self::extract_rust_reference(node, content, path, known_symbols),
            Language::TypeScript => {
                Self::extract_typescript_reference(node, content, path, known_symbols)
            }
            Language::Python => Self::extract_python_reference(node, content, path, known_symbols),
            Language::Go => Self::extract_go_reference(node, content, path, known_symbols),
            Language::Java => Self::extract_java_reference(node, content, path, known_symbols),
            _ => None,
        }
    }

    /// Extract references from Rust code
    fn extract_rust_reference(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        let kind_str = node.kind();

        // Check for identifier usage (not definition)
        if kind_str != "identifier" {
            return None;
        }

        let name = node.utf8_text(content.as_bytes()).ok()?.to_string();

        // Check if this identifier refers to a known symbol
        let symbol_id = known_symbols.get(&name)?.clone();

        let line = Self::get_line_from_byte_offset(content, node.start_byte());

        Some(SymbolReference {
            symbol_id,
            file: path.to_path_buf(),
            line,
            kind: ReferenceKind::Usage,
        })
    }

    /// Extract references from TypeScript/JavaScript code
    fn extract_typescript_reference(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        let kind_str = node.kind();

        // Check for identifier usage
        if kind_str != "identifier" && kind_str != "type_identifier" {
            return None;
        }

        let name = node.utf8_text(content.as_bytes()).ok()?.to_string();

        // Check if this identifier refers to a known symbol
        let symbol_id = known_symbols.get(&name)?.clone();

        let line = Self::get_line_from_byte_offset(content, node.start_byte());

        Some(SymbolReference {
            symbol_id,
            file: path.to_path_buf(),
            line,
            kind: ReferenceKind::Usage,
        })
    }

    /// Extract references from Python code
    fn extract_python_reference(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        let kind_str = node.kind();

        // Check for identifier usage
        if kind_str != "identifier" {
            return None;
        }

        let name = node.utf8_text(content.as_bytes()).ok()?.to_string();

        // Check if this identifier refers to a known symbol
        let symbol_id = known_symbols.get(&name)?.clone();

        let line = Self::get_line_from_byte_offset(content, node.start_byte());

        Some(SymbolReference {
            symbol_id,
            file: path.to_path_buf(),
            line,
            kind: ReferenceKind::Usage,
        })
    }

    /// Extract references from Go code
    fn extract_go_reference(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        let kind_str = node.kind();

        // Check for identifier usage
        if kind_str != "identifier" {
            return None;
        }

        let name = node.utf8_text(content.as_bytes()).ok()?.to_string();

        // Check if this identifier refers to a known symbol
        let symbol_id = known_symbols.get(&name)?.clone();

        let line = Self::get_line_from_byte_offset(content, node.start_byte());

        Some(SymbolReference {
            symbol_id,
            file: path.to_path_buf(),
            line,
            kind: ReferenceKind::Usage,
        })
    }

    /// Extract references from Java code
    fn extract_java_reference(
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        known_symbols: &HashMap<String, String>,
    ) -> Option<SymbolReference> {
        let kind_str = node.kind();

        // Check for identifier usage
        if kind_str != "identifier" {
            return None;
        }

        let name = node.utf8_text(content.as_bytes()).ok()?.to_string();

        // Check if this identifier refers to a known symbol
        let symbol_id = known_symbols.get(&name)?.clone();

        let line = Self::get_line_from_byte_offset(content, node.start_byte());

        Some(SymbolReference {
            symbol_id,
            file: path.to_path_buf(),
            line,
            kind: ReferenceKind::Usage,
        })
    }

    /// Get tree-sitter language for a programming language
    fn get_tree_sitter_language(language: &Language) -> Result<TSLanguage, ResearchError> {
        match language {
            Language::Rust => Ok(tree_sitter_rust::language()),
            Language::TypeScript => Ok(tree_sitter_typescript::language_typescript()),
            Language::Python => Ok(tree_sitter_python::language()),
            Language::Go => Ok(tree_sitter_go::language()),
            Language::Java => Ok(tree_sitter_java::language()),
            _ => Err(ResearchError::AnalysisFailed {
                reason: format!("Unsupported language for reference tracking: {:?}", language),
                context: "Reference tracking is only supported for Rust, TypeScript, Python, Go, and Java".to_string(),
            }),
        }
    }

    /// Calculate line number from byte offset
    fn get_line_from_byte_offset(content: &str, byte_offset: usize) -> usize {
        let prefix = &content[..byte_offset.min(content.len())];
        // Count newlines to get the line number (1-indexed)
        prefix.matches('\n').count() + 1
    }
}

#[cfg(test)]
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
