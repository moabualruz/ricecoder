//! Symbol Position Resolution
//!
//! This module provides symbol resolution at specific positions in code,
//! handling nested scopes and symbol shadowing.

use std::collections::HashMap;

use crate::types::{Position, Symbol};

/// Symbol scope information
#[derive(Debug, Clone)]
pub struct SymbolScope {
    /// Symbols in this scope
    pub symbols: Vec<Symbol>,
    /// Parent scope (for nested scopes)
    pub parent: Option<Box<SymbolScope>>,
    /// Scope range (start and end positions)
    pub start_line: u32,
    pub end_line: u32,
}

impl SymbolScope {
    /// Create a new symbol scope
    pub fn new(start_line: u32, end_line: u32) -> Self {
        Self {
            symbols: Vec::new(),
            parent: None,
            start_line,
            end_line,
        }
    }

    /// Add a symbol to this scope
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Find symbol at position in this scope
    pub fn find_symbol_at_position(&self, position: Position) -> Option<Symbol> {
        // Check if position is within this scope
        if position.line < self.start_line || position.line > self.end_line {
            return None;
        }

        // Look for symbol in this scope
        for symbol in &self.symbols {
            if symbol.range.start.line <= position.line && position.line <= symbol.range.end.line {
                // Check if position is within the symbol's range
                if position.line == symbol.range.start.line
                    && position.character < symbol.range.start.character
                {
                    continue;
                }
                if position.line == symbol.range.end.line
                    && position.character > symbol.range.end.character
                {
                    continue;
                }
                return Some(symbol.clone());
            }
        }

        // Check parent scope
        if let Some(parent) = &self.parent {
            parent.find_symbol_at_position(position)
        } else {
            None
        }
    }

    /// Get all symbols visible at position (including parent scopes)
    pub fn get_visible_symbols(&self, position: Position) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        // Add symbols from parent scopes
        if let Some(parent) = &self.parent {
            symbols.extend(parent.get_visible_symbols(position));
        }

        // Add symbols from this scope
        for symbol in &self.symbols {
            if symbol.range.start.line <= position.line && position.line <= symbol.range.end.line {
                symbols.push(symbol.clone());
            }
        }

        symbols
    }
}

/// Symbol resolver for position-based lookups
pub struct SymbolResolver {
    /// Global symbol index
    global_symbols: HashMap<String, Symbol>,
    /// Scoped symbols
    scopes: Vec<SymbolScope>,
}

impl SymbolResolver {
    /// Create a new symbol resolver
    pub fn new() -> Self {
        Self {
            global_symbols: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    /// Index symbols for resolution
    pub fn index_symbols(&mut self, symbols: Vec<Symbol>) {
        self.global_symbols.clear();
        for symbol in symbols {
            self.global_symbols.insert(symbol.name.clone(), symbol);
        }
    }

    /// Add a scope with symbols
    pub fn add_scope(&mut self, scope: SymbolScope) {
        self.scopes.push(scope);
    }

    /// Resolve symbol at position
    pub fn resolve_at_position(&self, position: Position) -> Option<Symbol> {
        // First check scoped symbols (more specific)
        for scope in &self.scopes {
            if let Some(symbol) = scope.find_symbol_at_position(position) {
                return Some(symbol);
            }
        }

        // Fall back to global symbols
        // Find symbol by name at position (simple approach)
        for symbol in self.global_symbols.values() {
            if symbol.range.start.line <= position.line && position.line <= symbol.range.end.line {
                if position.line == symbol.range.start.line
                    && position.character >= symbol.range.start.character
                    && position.character <= symbol.range.end.character
                {
                    return Some(symbol.clone());
                }
                if position.line == symbol.range.end.line
                    && position.character >= symbol.range.start.character
                    && position.character <= symbol.range.end.character
                {
                    return Some(symbol.clone());
                }
                if position.line > symbol.range.start.line && position.line < symbol.range.end.line
                {
                    return Some(symbol.clone());
                }
            }
        }

        None
    }

    /// Get symbol by name
    pub fn get_symbol(&self, name: &str) -> Option<Symbol> {
        self.global_symbols.get(name).cloned()
    }

    /// Get all symbols
    pub fn get_all_symbols(&self) -> Vec<Symbol> {
        self.global_symbols.values().cloned().collect()
    }

    /// Handle symbol shadowing - get the most specific symbol at position
    pub fn resolve_with_shadowing(&self, position: Position) -> Option<Symbol> {
        // Check scoped symbols first (they shadow global symbols)
        for scope in &self.scopes {
            if position.line >= scope.start_line && position.line <= scope.end_line {
                if let Some(symbol) = scope.find_symbol_at_position(position) {
                    return Some(symbol);
                }
            }
        }

        // Fall back to global symbols
        self.resolve_at_position(position)
    }

    /// Get all visible symbols at position (for autocomplete, etc.)
    pub fn get_visible_at_position(&self, position: Position) -> Vec<Symbol> {
        let mut visible = Vec::new();

        // Add global symbols
        visible.extend(self.global_symbols.values().cloned());

        // Add scoped symbols
        for scope in &self.scopes {
            if position.line >= scope.start_line && position.line <= scope.end_line {
                visible.extend(scope.get_visible_symbols(position));
            }
        }

        // Remove duplicates (keep the most specific one)
        let mut seen = std::collections::HashSet::new();
        visible.retain(|s| seen.insert(s.name.clone()));

        visible
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Range, SymbolKind};

    #[test]
    fn test_symbol_scope_creation() {
        let scope = SymbolScope::new(0, 10);
        assert_eq!(scope.start_line, 0);
        assert_eq!(scope.end_line, 10);
        assert!(scope.symbols.is_empty());
    }

    #[test]
    fn test_add_symbol_to_scope() {
        let mut scope = SymbolScope::new(0, 10);
        let symbol = Symbol {
            name: "test_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(2, 0), Position::new(5, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        scope.add_symbol(symbol);
        assert_eq!(scope.symbols.len(), 1);
    }

    #[test]
    fn test_find_symbol_in_scope() {
        let mut scope = SymbolScope::new(0, 10);
        let symbol = Symbol {
            name: "my_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(2, 0), Position::new(5, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        scope.add_symbol(symbol);

        // Position within symbol range
        let found = scope.find_symbol_at_position(Position::new(3, 5));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "my_fn");

        // Position outside symbol range
        let found = scope.find_symbol_at_position(Position::new(6, 0));
        assert!(found.is_none());
    }

    #[test]
    fn test_symbol_resolver_creation() {
        let resolver = SymbolResolver::new();
        assert_eq!(resolver.global_symbols.len(), 0);
        assert_eq!(resolver.scopes.len(), 0);
    }

    #[test]
    fn test_index_symbols() {
        let mut resolver = SymbolResolver::new();
        let symbol = Symbol {
            name: "global_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        resolver.index_symbols(vec![symbol]);
        assert_eq!(resolver.global_symbols.len(), 1);
    }

    #[test]
    fn test_get_symbol_by_name() {
        let mut resolver = SymbolResolver::new();
        let symbol = Symbol {
            name: "my_function".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(5, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        resolver.index_symbols(vec![symbol]);
        let found = resolver.get_symbol("my_function");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "my_function");
    }

    #[test]
    fn test_resolve_at_position() {
        let mut resolver = SymbolResolver::new();
        let symbol = Symbol {
            name: "test_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(5, 0), Position::new(5, 8)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        resolver.index_symbols(vec![symbol]);
        let found = resolver.resolve_at_position(Position::new(5, 4));
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test_var");
    }

    #[test]
    fn test_symbol_shadowing() {
        let mut resolver = SymbolResolver::new();

        // Global symbol
        let global_symbol = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(0, 0), Position::new(20, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        resolver.index_symbols(vec![global_symbol]);

        // Local symbol (shadows global)
        let mut scope = SymbolScope::new(5, 15);
        let local_symbol = Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(6, 0), Position::new(10, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        scope.add_symbol(local_symbol);
        resolver.add_scope(scope);

        // At position within local scope, should get local symbol
        let found = resolver.resolve_with_shadowing(Position::new(7, 0));
        assert!(found.is_some());
        let symbol = found.unwrap();
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.range.start.line, 6);
    }

    #[test]
    fn test_get_visible_symbols() {
        let mut resolver = SymbolResolver::new();

        // Global symbols
        let global1 = Symbol {
            name: "global_fn".to_string(),
            kind: SymbolKind::Function,
            range: Range::new(Position::new(0, 0), Position::new(20, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        let global2 = Symbol {
            name: "global_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(0, 0), Position::new(20, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        resolver.index_symbols(vec![global1, global2]);

        // Local scope
        let mut scope = SymbolScope::new(5, 15);
        let local_symbol = Symbol {
            name: "local_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(6, 0), Position::new(10, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        scope.add_symbol(local_symbol);
        resolver.add_scope(scope);

        // Get visible symbols at position within local scope
        let visible = resolver.get_visible_at_position(Position::new(7, 0));
        assert!(visible.len() >= 3); // At least global_fn, global_var, local_var
    }

    #[test]
    fn test_nested_scopes() {
        let mut outer_scope = SymbolScope::new(0, 20);
        let outer_symbol = Symbol {
            name: "outer_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(1, 0), Position::new(19, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        outer_scope.add_symbol(outer_symbol);

        // Inner scope
        let mut inner_scope = SymbolScope::new(5, 15);
        let inner_symbol = Symbol {
            name: "inner_var".to_string(),
            kind: SymbolKind::Variable,
            range: Range::new(Position::new(6, 0), Position::new(14, 0)),
            definition: None,
            references: vec![],
            documentation: None,
        };

        inner_scope.add_symbol(inner_symbol);
        inner_scope.parent = Some(Box::new(outer_scope));

        // Find symbol in inner scope should also check parent
        let found = inner_scope.find_symbol_at_position(Position::new(10, 0));
        assert!(found.is_some());
    }
}
