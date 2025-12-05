//! Hover Information Provider
//!
//! This module provides hover information for symbols in code, including type information,
//! documentation, and definition locations.

pub mod symbol_resolver;

pub use symbol_resolver::{SymbolResolver, SymbolScope};

use crate::types::{HoverInfo, MarkupContent, Position, Symbol};
use std::collections::HashMap;

/// Hover information provider
pub struct HoverProvider {
    /// Symbol index for fast lookup
    symbol_index: HashMap<String, Symbol>,
}

impl HoverProvider {
    /// Create a new hover provider
    pub fn new() -> Self {
        Self {
            symbol_index: HashMap::new(),
        }
    }

    /// Index symbols for fast lookup
    pub fn index_symbols(&mut self, symbols: Vec<Symbol>) {
        self.symbol_index.clear();
        for symbol in symbols {
            self.symbol_index.insert(symbol.name.clone(), symbol);
        }
    }

    /// Get hover information at a specific position
    pub fn get_hover_info(&self, code: &str, position: Position) -> Option<HoverInfo> {
        // Find the symbol at the given position
        let symbol = self.find_symbol_at_position(code, position)?;

        // Build hover content
        let mut content = String::new();

        // Add type information
        content.push_str(&format!("**{}** `{:?}`\n\n", symbol.name, symbol.kind));

        // Add documentation if available
        if let Some(doc) = &symbol.documentation {
            content.push_str(doc);
            content.push_str("\n\n");
        }

        // Add definition location
        if let Some(def) = &symbol.definition {
            content.push_str(&format!(
                "**Defined at**: `{}`:{}-{}\n",
                def.uri, def.range.start.line, def.range.start.character
            ));
        }

        // Add usage count
        let usage_count = symbol.references.len();
        content.push_str(&format!("**References**: {}", usage_count));

        let hover_info = HoverInfo::new(MarkupContent::markdown(content)).with_range(symbol.range);

        Some(hover_info)
    }

    /// Find symbol at a specific position in code
    fn find_symbol_at_position(&self, code: &str, position: Position) -> Option<Symbol> {
        // Convert position to byte offset
        let target_offset = self.position_to_offset(code, position)?;

        // Find symbol that contains this position
        for symbol in self.symbol_index.values() {
            let start_offset = self.position_to_offset(code, symbol.range.start)?;
            let end_offset = self.position_to_offset(code, symbol.range.end)?;

            if target_offset >= start_offset && target_offset <= end_offset {
                return Some(symbol.clone());
            }
        }

        None
    }

    /// Convert position to byte offset in code
    fn position_to_offset(&self, code: &str, position: Position) -> Option<usize> {
        let mut offset = 0;
        let mut current_line = 0;
        let mut current_char = 0;

        for ch in code.chars() {
            if current_line == position.line && current_char == position.character {
                return Some(offset);
            }

            if ch == '\n' {
                current_line += 1;
                current_char = 0;
            } else {
                current_char += 1;
            }

            offset += ch.len_utf8();
        }

        // Check if we're at the end
        if current_line == position.line && current_char == position.character {
            return Some(offset);
        }

        None
    }

    /// Get type information for a symbol
    pub fn get_type_info(&self, symbol_name: &str) -> Option<String> {
        self.symbol_index
            .get(symbol_name)
            .map(|symbol| format!("{:?}", symbol.kind))
    }

    /// Get definition location for a symbol
    pub fn get_definition_location(&self, symbol_name: &str) -> Option<(String, u32, u32)> {
        self.symbol_index.get(symbol_name).and_then(|symbol| {
            symbol.definition.as_ref().map(|def| {
                (
                    def.uri.clone(),
                    def.range.start.line,
                    def.range.start.character,
                )
            })
        })
    }

    /// Get documentation for a symbol
    pub fn get_documentation(&self, symbol_name: &str) -> Option<String> {
        self.symbol_index
            .get(symbol_name)
            .and_then(|symbol| symbol.documentation.clone())
    }

    /// Get usage count for a symbol
    pub fn get_usage_count(&self, symbol_name: &str) -> usize {
        self.symbol_index
            .get(symbol_name)
            .map(|symbol| symbol.references.len())
            .unwrap_or(0)
    }
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Definition, Range, SymbolKind};

    #[test]
    fn test_hover_provider_creation() {
        let provider = HoverProvider::new();
        assert_eq!(provider.symbol_index.len(), 0);
    }

    #[test]
    fn test_index_symbols() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "test_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 7)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        provider.index_symbols(vec![symbol.clone()]);
        assert_eq!(provider.symbol_index.len(), 1);
        assert!(provider.symbol_index.contains_key("test_fn"));
    }

    #[test]
    fn test_get_type_info() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "my_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(0, 0), Position::new(0, 6)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        provider.index_symbols(vec![symbol]);
        let type_info = provider.get_type_info("my_var");
        assert!(type_info.is_some());
    }

    #[test]
    fn test_get_definition_location() {
        let mut provider = HoverProvider::new();
        let definition = Definition {
            uri: "file://test.rs".to_string(),
            range: Range::new(Position::new(5, 0), Position::new(5, 10)),
        };

        let symbol = Symbol {
            name: "my_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 5)),
            definition: Some(definition),
            references: vec![],
            documentation: None,
        };

        provider.index_symbols(vec![symbol]);
        let location = provider.get_definition_location("my_fn");
        assert!(location.is_some());
        let (uri, line, char) = location.unwrap();
        assert_eq!(uri, "file://test.rs");
        assert_eq!(line, 5);
        assert_eq!(char, 0);
    }

    #[test]
    fn test_get_documentation() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "documented_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 13)),
            definition: None,
            references: vec![],
            documentation: Some("This is a test function".to_string()),
        };

        provider.index_symbols(vec![symbol]);
        let doc = provider.get_documentation("documented_fn");
        assert_eq!(doc, Some("This is a test function".to_string()));
    }

    #[test]
    fn test_get_usage_count() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "used_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 7)),
            definition: None,
            references: vec![
                crate::types::Reference {
                    uri: "file://test.rs".to_string(),
                    range: Range::new(Position::new(10, 0), Position::new(10, 7)),
                },
                crate::types::Reference {
                    uri: "file://test.rs".to_string(),
                    range: Range::new(Position::new(20, 0), Position::new(20, 7)),
                },
            ],
            documentation: None,
        };

        provider.index_symbols(vec![symbol]);
        let count = provider.get_usage_count("used_fn");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_position_to_offset() {
        let provider = HoverProvider::new();
        let code = "fn main() {\n    println!(\"hello\");\n}";

        // Test first position
        let offset = provider.position_to_offset(code, Position::new(0, 0));
        assert_eq!(offset, Some(0));

        // Test position after newline (line 1, char 0)
        let offset = provider.position_to_offset(code, Position::new(1, 0));
        assert_eq!(offset, Some(12)); // "fn main() {\n" = 11 bytes + 1 for newline

        // Test position in middle of line
        let offset = provider.position_to_offset(code, Position::new(0, 2));
        assert_eq!(offset, Some(2)); // "fn"
    }

    #[test]
    fn test_find_symbol_at_position() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "test_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 7)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        provider.index_symbols(vec![symbol.clone()]);

        let code = "test_fn";
        let found = provider.find_symbol_at_position(code, Position::new(0, 3));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test_fn");
    }

    #[test]
    fn test_get_hover_info() {
        let mut provider = HoverProvider::new();
        let symbol = Symbol {
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(0, 11)),
            definition: Some(Definition {
                uri: "file://test.rs".to_string(),
                range: Range::new(Position::new(5, 0), Position::new(5, 11)),
            }),
            references: vec![],
            documentation: Some("A test function".to_string()),
        };

        provider.index_symbols(vec![symbol]);

        let code = "my_function";
        let hover = provider.get_hover_info(code, Position::new(0, 5));
        assert!(hover.is_some());

        let hover_info = hover.unwrap();
        assert!(hover_info.contents.value.contains("my_function"));
        assert!(hover_info.contents.value.contains("A test function"));
        assert!(hover_info.range.is_some());
    }
}
