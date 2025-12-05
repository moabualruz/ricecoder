//! Search engine for semantic and full-text search

use crate::models::{SearchResult, SymbolKind};
use crate::semantic_index::SemanticIndex;
use regex::Regex;
use std::collections::HashMap;

/// Search engine for code search
pub struct SearchEngine {
    index: SemanticIndex,
}

/// Search query options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub max_results: usize,
    /// Filter by symbol kind
    pub kind_filter: Option<SymbolKind>,
    /// Case-sensitive search
    pub case_sensitive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            max_results: 100,
            kind_filter: None,
            case_sensitive: false,
        }
    }
}

impl SearchEngine {
    /// Create a new search engine with an index
    pub fn new(index: SemanticIndex) -> Self {
        SearchEngine { index }
    }

    /// Search for symbols by name
    pub fn search_by_name(&self, query: &str, options: &SearchOptions) -> Vec<SearchResult> {
        let mut results = self.index.search_by_name(query);

        // Apply kind filter if specified
        if let Some(kind) = options.kind_filter {
            results.retain(|r| r.symbol.kind == kind);
        }

        // Limit results
        results.truncate(options.max_results);

        results
    }

    /// Search for symbols by kind
    pub fn search_by_kind(&self, kind: SymbolKind, options: &SearchOptions) -> Vec<SearchResult> {
        let symbols = self.index.search_by_kind(kind);

        let mut results: Vec<SearchResult> = symbols
            .into_iter()
            .map(|symbol| SearchResult {
                symbol: symbol.clone(),
                relevance: 1.0,
                context: None,
            })
            .collect();

        // Limit results
        results.truncate(options.max_results);

        results
    }

    /// Full-text search across all symbols
    pub fn full_text_search(&self, query: &str, options: &SearchOptions) -> Vec<SearchResult> {
        let query_lower = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut results = Vec::new();

        for symbol in self.index.all_symbols() {
            // Check if symbol name matches
            let symbol_name = if options.case_sensitive {
                symbol.name.clone()
            } else {
                symbol.name.to_lowercase()
            };

            if symbol_name.contains(&query_lower) {
                // Calculate relevance based on match quality
                let relevance = if symbol_name == query_lower {
                    1.0
                } else if symbol_name.starts_with(&query_lower) {
                    0.8
                } else {
                    0.5
                };

                // Apply kind filter if specified
                if let Some(kind) = options.kind_filter {
                    if symbol.kind != kind {
                        continue;
                    }
                }

                results.push(SearchResult {
                    symbol: symbol.clone(),
                    relevance,
                    context: None,
                });
            }
        }

        // Sort by relevance (descending)
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(options.max_results);

        results
    }

    /// Search using a regex pattern
    pub fn regex_search(&self, pattern: &str, options: &SearchOptions) -> Vec<SearchResult> {
        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();

        for symbol in self.index.all_symbols() {
            if regex.is_match(&symbol.name) {
                // Apply kind filter if specified
                if let Some(kind) = options.kind_filter {
                    if symbol.kind != kind {
                        continue;
                    }
                }

                results.push(SearchResult {
                    symbol: symbol.clone(),
                    relevance: 0.7,
                    context: None,
                });
            }
        }

        // Limit results
        results.truncate(options.max_results);

        results
    }

    /// Get all symbols
    pub fn all_symbols(&self) -> Vec<SearchResult> {
        self.index
            .all_symbols()
            .into_iter()
            .map(|symbol| SearchResult {
                symbol: symbol.clone(),
                relevance: 1.0,
                context: None,
            })
            .collect()
    }

    /// Get statistics about the index
    pub fn get_statistics(&self) -> SearchStatistics {
        let all_symbols = self.index.all_symbols();
        let mut kind_counts: HashMap<String, usize> = HashMap::new();

        for symbol in &all_symbols {
            let kind_str = format!("{:?}", symbol.kind);
            *kind_counts.entry(kind_str).or_insert(0) += 1;
        }

        SearchStatistics {
            total_symbols: self.index.symbol_count(),
            total_references: self.index.reference_count(),
            kind_distribution: kind_counts,
        }
    }
}

/// Statistics about the search index
#[derive(Debug, Clone)]
pub struct SearchStatistics {
    /// Total number of symbols
    pub total_symbols: usize,
    /// Total number of references
    pub total_references: usize,
    /// Distribution of symbols by kind
    pub kind_distribution: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Symbol;
    use std::path::PathBuf;

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

    fn create_test_index() -> SemanticIndex {
        let mut index = SemanticIndex::new();
        index.add_symbol(create_test_symbol(
            "sym1",
            "my_function",
            SymbolKind::Function,
        ));
        index.add_symbol(create_test_symbol("sym2", "MyClass", SymbolKind::Class));
        index.add_symbol(create_test_symbol(
            "sym3",
            "my_constant",
            SymbolKind::Constant,
        ));
        index
    }

    #[test]
    fn test_search_by_name() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions::default();

        let results = engine.search_by_name("my_", &options);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_by_kind() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions::default();

        let results = engine.search_by_kind(SymbolKind::Function, &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.name, "my_function");
    }

    #[test]
    fn test_full_text_search() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions::default();

        let results = engine.full_text_search("function", &options);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_full_text_search_case_insensitive() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions {
            case_sensitive: false,
            ..Default::default()
        };

        let results = engine.full_text_search("MY_FUNCTION", &options);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_full_text_search_case_sensitive() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions {
            case_sensitive: true,
            ..Default::default()
        };

        let results = engine.full_text_search("MY_FUNCTION", &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_regex_search() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions::default();

        let results = engine.regex_search("my_.*", &options);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_regex_search_invalid_pattern() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions::default();

        let results = engine.regex_search("[invalid(", &options);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_with_kind_filter() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions {
            kind_filter: Some(SymbolKind::Function),
            ..Default::default()
        };

        let results = engine.full_text_search("my", &options);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.kind, SymbolKind::Function);
    }

    #[test]
    fn test_search_max_results() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);
        let options = SearchOptions {
            max_results: 1,
            ..Default::default()
        };

        let results = engine.full_text_search("my", &options);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_all_symbols() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);

        let results = engine.all_symbols();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_get_statistics() {
        let index = create_test_index();
        let engine = SearchEngine::new(index);

        let stats = engine.get_statistics();
        assert_eq!(stats.total_symbols, 3);
        assert!(stats.kind_distribution.contains_key("Function"));
        assert!(stats.kind_distribution.contains_key("Class"));
        assert!(stats.kind_distribution.contains_key("Constant"));
    }
}
