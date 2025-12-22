//! Semantic index for fast symbol lookup and search

use std::{collections::HashMap, path::PathBuf};

use crate::models::{SearchResult, Symbol, SymbolKind, SymbolReference};

/// Semantic index for code symbols
#[derive(Debug, Clone)]
pub struct SemanticIndex {
    /// Map from symbol ID to symbol
    symbols_by_id: HashMap<String, Symbol>,
    /// Map from symbol name to symbol IDs
    symbols_by_name: HashMap<String, Vec<String>>,
    /// Map from file path to symbol IDs in that file
    symbols_by_file: HashMap<PathBuf, Vec<String>>,
    /// Map from symbol ID to all references to that symbol
    references_by_symbol: HashMap<String, Vec<SymbolReference>>,
}

impl SemanticIndex {
    /// Create a new semantic index
    pub fn new() -> Self {
        SemanticIndex {
            symbols_by_id: HashMap::new(),
            symbols_by_name: HashMap::new(),
            symbols_by_file: HashMap::new(),
            references_by_symbol: HashMap::new(),
        }
    }

    /// Add a symbol to the index
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let symbol_id = symbol.id.clone();
        let symbol_name = symbol.name.clone();
        let symbol_file = symbol.file.clone();

        // Add to symbols_by_id
        self.symbols_by_id.insert(symbol_id.clone(), symbol);

        // Add to symbols_by_name
        self.symbols_by_name
            .entry(symbol_name)
            .or_default()
            .push(symbol_id.clone());

        // Add to symbols_by_file
        self.symbols_by_file
            .entry(symbol_file)
            .or_default()
            .push(symbol_id);
    }

    /// Add a reference to the index
    pub fn add_reference(&mut self, reference: SymbolReference) {
        self.references_by_symbol
            .entry(reference.symbol_id.clone())
            .or_default()
            .push(reference);
    }

    /// Get a symbol by ID
    pub fn get_symbol(&self, symbol_id: &str) -> Option<&Symbol> {
        self.symbols_by_id.get(symbol_id)
    }

    /// Get all symbols with a given name
    pub fn get_symbols_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.symbols_by_name
            .get(name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.symbols_by_id.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all symbols in a file
    pub fn get_symbols_in_file(&self, file: &PathBuf) -> Vec<&Symbol> {
        self.symbols_by_file
            .get(file)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.symbols_by_id.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all references to a symbol
    pub fn get_references_to_symbol(&self, symbol_id: &str) -> Vec<&SymbolReference> {
        self.references_by_symbol
            .get(symbol_id)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Search for symbols by name (substring match)
    pub fn search_by_name(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for (name, symbol_ids) in &self.symbols_by_name {
            if name.contains(query) {
                for symbol_id in symbol_ids {
                    if let Some(symbol) = self.symbols_by_id.get(symbol_id) {
                        // Calculate relevance based on match quality
                        let relevance = if name == query {
                            1.0
                        } else if name.starts_with(query) {
                            0.8
                        } else {
                            0.5
                        };

                        results.push(SearchResult {
                            symbol: symbol.clone(),
                            relevance,
                            context: None,
                        });
                    }
                }
            }
        }

        // Sort by relevance (descending)
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Search for symbols by kind
    pub fn search_by_kind(&self, kind: SymbolKind) -> Vec<&Symbol> {
        self.symbols_by_id
            .values()
            .filter(|symbol| symbol.kind == kind)
            .collect()
    }

    /// Get all symbols in the index
    pub fn all_symbols(&self) -> Vec<&Symbol> {
        self.symbols_by_id.values().collect()
    }

    /// Get the total number of symbols
    pub fn symbol_count(&self) -> usize {
        self.symbols_by_id.len()
    }

    /// Get the total number of references
    pub fn reference_count(&self) -> usize {
        self.references_by_symbol
            .values()
            .map(|refs| refs.len())
            .sum()
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.symbols_by_id.clear();
        self.symbols_by_name.clear();
        self.symbols_by_file.clear();
        self.references_by_symbol.clear();
    }
}

impl Default for SemanticIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_symbol(id: &str, name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            references: Vec::new(),
        }
    }

    #[test]
    fn test_add_and_get_symbol() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);

        index.add_symbol(symbol.clone());

        assert_eq!(index.get_symbol("sym1"), Some(&symbol));
    }

    #[test]
    fn test_get_symbols_by_name() {
        let mut index = SemanticIndex::new();
        let symbol1 = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        let symbol2 = create_test_symbol("sym2", "my_function", SymbolKind::Function);

        index.add_symbol(symbol1.clone());
        index.add_symbol(symbol2.clone());

        let results = index.get_symbols_by_name("my_function");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_symbols_in_file() {
        let mut index = SemanticIndex::new();
        let symbol1 = create_test_symbol("sym1", "func1", SymbolKind::Function);
        let mut symbol2 = create_test_symbol("sym2", "func2", SymbolKind::Function);
        symbol2.file = PathBuf::from("other.rs");

        index.add_symbol(symbol1.clone());
        index.add_symbol(symbol2);

        let results = index.get_symbols_in_file(&PathBuf::from("test.rs"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "func1");
    }

    #[test]
    fn test_search_by_name_exact_match() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        let results = index.search_by_name("my_function");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].relevance, 1.0);
    }

    #[test]
    fn test_search_by_name_prefix_match() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        let results = index.search_by_name("my_");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].relevance, 0.8);
    }

    #[test]
    fn test_search_by_name_substring_match() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        let results = index.search_by_name("function");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].relevance, 0.5);
    }

    #[test]
    fn test_search_by_kind() {
        let mut index = SemanticIndex::new();
        let func = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        let class = create_test_symbol("sym2", "MyClass", SymbolKind::Class);

        index.add_symbol(func);
        index.add_symbol(class);

        let results = index.search_by_kind(SymbolKind::Function);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_add_reference() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        let reference = SymbolReference {
            symbol_id: "sym1".to_string(),
            file: PathBuf::from("test.rs"),
            line: 5,
            kind: crate::models::ReferenceKind::Usage,
        };

        index.add_reference(reference);

        let refs = index.get_references_to_symbol("sym1");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].line, 5);
    }

    #[test]
    fn test_symbol_count() {
        let mut index = SemanticIndex::new();
        let symbol1 = create_test_symbol("sym1", "func1", SymbolKind::Function);
        let symbol2 = create_test_symbol("sym2", "func2", SymbolKind::Function);

        index.add_symbol(symbol1);
        index.add_symbol(symbol2);

        assert_eq!(index.symbol_count(), 2);
    }

    #[test]
    fn test_reference_count() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        let ref1 = SymbolReference {
            symbol_id: "sym1".to_string(),
            file: PathBuf::from("test.rs"),
            line: 5,
            kind: crate::models::ReferenceKind::Usage,
        };

        let ref2 = SymbolReference {
            symbol_id: "sym1".to_string(),
            file: PathBuf::from("test.rs"),
            line: 10,
            kind: crate::models::ReferenceKind::Usage,
        };

        index.add_reference(ref1);
        index.add_reference(ref2);

        assert_eq!(index.reference_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut index = SemanticIndex::new();
        let symbol = create_test_symbol("sym1", "my_function", SymbolKind::Function);
        index.add_symbol(symbol);

        assert_eq!(index.symbol_count(), 1);

        index.clear();

        assert_eq!(index.symbol_count(), 0);
    }

    #[test]
    fn test_all_symbols() {
        let mut index = SemanticIndex::new();
        let symbol1 = create_test_symbol("sym1", "func1", SymbolKind::Function);
        let symbol2 = create_test_symbol("sym2", "func2", SymbolKind::Function);

        index.add_symbol(symbol1);
        index.add_symbol(symbol2);

        let all = index.all_symbols();
        assert_eq!(all.len(), 2);
    }
}
