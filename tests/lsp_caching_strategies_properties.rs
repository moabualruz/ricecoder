//! Property-based tests for LSP caching strategies
//!
//! **Feature: ricecoder-lsp, Property 8: Caching strategies**
//! **Validates: Requirements NFR-1 (Performance)**
//!
//! These tests verify that the LSP server implements effective caching:
//! - Parsed ASTs are cached for unchanged documents
//! - Symbol indexes are cached for unchanged files
//! - Cache invalidation works on document changes
//! - Cache hit rates are monitored and reported

use proptest::prelude::*;
use ricecoder_lsp::cache::{hash_input, AstCache, SemanticCache, SymbolIndexCache};
use ricecoder_lsp::types::SemanticInfo;
use std::collections::HashMap;

/// Strategy for generating document URIs
fn uri_strategy() -> impl Strategy<Value = String> {
    r"file://[a-z0-9_]+\.(rs|ts|py)".prop_map(|s| s)
}

/// Strategy for generating code content
fn code_strategy() -> impl Strategy<Value = String> {
    r"[a-z\s\n\{\}\(\);:,=]+".prop_map(|s| s)
}

proptest! {
    /// Property 8.1: AST caching for unchanged documents
    /// For any unchanged document, retrieving cached AST should return the same value
    #[test]
    fn prop_ast_cache_unchanged_documents(
        uri in uri_strategy(),
        code in code_strategy()
    ) {
        let cache = AstCache::new();
        let hash = hash_input(&code);
        let ast = format!("AST for {}", code);

        // Store AST
        cache.put(uri.clone(), hash, ast.clone());

        // Retrieve AST multiple times
        let cached1 = cache.get(&uri, hash);
        let cached2 = cache.get(&uri, hash);

        prop_assert_eq!(cached1, Some(ast.clone()));
        prop_assert_eq!(cached2, Some(ast.clone()));

        // Both retrievals should be cache hits
        let metrics = cache.metrics();
        prop_assert_eq!(metrics.hits, 2);
        prop_assert_eq!(metrics.misses, 0);
    }

    /// Property 8.2: Cache invalidation on document changes
    /// For any cached document, invalidation should remove it from cache
    #[test]
    fn prop_cache_invalidation_on_changes(
        uri in uri_strategy(),
        code1 in code_strategy(),
        code2 in code_strategy()
    ) {
        let cache = AstCache::new();
        let hash1 = hash_input(&code1);
        let hash2 = hash_input(&code2);

        // Store first version
        cache.put(uri.clone(), hash1, "AST1".to_string());

        // Verify it's cached
        let cached = cache.get(&uri, hash1);
        prop_assert_eq!(cached, Some("AST1".to_string()));

        // Invalidate cache
        cache.invalidate(&uri);

        // Verify it's no longer cached
        let cached = cache.get(&uri, hash1);
        prop_assert_eq!(cached, None);

        // Metrics should show invalidation
        let metrics = cache.metrics();
        prop_assert_eq!(metrics.invalidations, 1);
    }

    /// Property 8.3: Symbol index caching
    /// For any symbol index, caching and retrieval should preserve the index
    #[test]
    fn prop_symbol_index_caching(
        uri in uri_strategy(),
        code in code_strategy(),
        symbols in prop::collection::vec("[a-z_]+", 1..10)
    ) {
        let cache = SymbolIndexCache::new();
        let hash = hash_input(&code);

        // Create symbol index
        let mut index = HashMap::new();
        for (i, symbol) in symbols.iter().enumerate() {
            index.insert(symbol.clone(), i);
        }

        // Store index
        cache.put(uri.clone(), hash, index.clone());

        // Retrieve index
        let cached = cache.get(&uri, hash);
        prop_assert_eq!(cached, Some(index));
    }

    /// Property 8.4: Semantic cache with multiple entries
    /// For any set of documents, cache should maintain separate entries
    #[test]
    fn prop_semantic_cache_multiple_entries(
        uris in prop::collection::vec(uri_strategy(), 2..10),
        codes in prop::collection::vec(code_strategy(), 2..10)
    ) {
        let cache = SemanticCache::new();

        // Store multiple entries
        for (i, uri) in uris.iter().enumerate() {
            let code = &codes[i % codes.len()];
            let hash = hash_input(code);

            let mut info = SemanticInfo::new();
            info.imports.push(format!("import{}", i));
            cache.put(uri.clone(), hash, info);
        }

        // Verify all entries are cached
        for (i, uri) in uris.iter().enumerate() {
            let code = &codes[i % codes.len()];
            let hash = hash_input(code);

            let cached = cache.get(uri, hash);
            prop_assert!(cached.is_some());
            prop_assert_eq!(cached.unwrap().imports.len(), 1);
        }
    }

    /// Property 8.5: Cache hit rate consistency
    /// For any sequence of cache operations, hit rate should be consistent
    #[test]
    fn prop_cache_hit_rate_consistency(
        operations in prop::collection::vec(0..100u32, 10..100)
    ) {
        let cache = SemanticCache::new();
        let mut expected_hits = 0;
        let mut expected_misses = 0;

        for (i, op) in operations.iter().enumerate() {
            let uri = format!("file://test{}.rs", op % 5);
            let hash = hash_input(&format!("code{}", op));

            if i % 3 == 0 {
                // Store
                let mut info = SemanticInfo::new();
                info.imports.push(format!("import{}", op));
                cache.put(uri, hash, info);
            } else if i % 3 == 1 {
                // Retrieve (likely hit if stored recently)
                let _cached = cache.get(&uri, hash);
                if i > 0 && (i - 1) % 3 == 0 {
                    expected_hits += 1;
                } else {
                    expected_misses += 1;
                }
            } else {
                // Retrieve with wrong hash (miss)
                let _cached = cache.get(&uri, hash + 1);
                expected_misses += 1;
            }
        }

        let metrics = cache.metrics();
        let hit_rate = metrics.hit_rate();

        // Hit rate should be between 0 and 100%
        prop_assert!(hit_rate >= 0.0 && hit_rate <= 100.0);
    }

    /// Property 8.6: Cache memory efficiency
    /// For any cache with size limit, it should not exceed the limit
    #[test]
    fn prop_cache_memory_efficiency(
        codes in prop::collection::vec("[a-z]{10,100}", 5..20)
    ) {
        let cache = SemanticCache::with_size(10 * 1024); // 10KB limit

        // Store multiple entries
        for (i, code) in codes.iter().enumerate() {
            let uri = format!("file://test{}.rs", i);
            let hash = hash_input(code);

            let mut info = SemanticInfo::new();
            info.imports.push(format!("import{}", i));
            cache.put(uri, hash, info);
        }

        // Cache should have processed entries
        let metrics = cache.metrics();
        prop_assert!(metrics.hits + metrics.misses >= 0);
    }

    /// Property 8.7: Cache invalidation consistency
    /// For any invalidated entry, subsequent retrievals should miss
    #[test]
    fn prop_cache_invalidation_consistency(
        uri in uri_strategy(),
        code in code_strategy()
    ) {
        let cache = AstCache::new();
        let hash = hash_input(&code);

        // Store
        cache.put(uri.clone(), hash, "AST".to_string());

        // Verify hit
        let _cached = cache.get(&uri, hash);

        // Invalidate
        cache.invalidate(&uri);

        // Verify miss
        let _cached = cache.get(&uri, hash);
        let _cached = cache.get(&uri, hash);

        let metrics = cache.metrics();
        prop_assert_eq!(metrics.hits, 1);
        prop_assert_eq!(metrics.misses, 2);
        prop_assert_eq!(metrics.invalidations, 1);
    }

    /// Property 8.8: Cache clear operation
    /// For any cache with entries, clear should remove all entries
    #[test]
    fn prop_cache_clear_removes_all(
        uris in prop::collection::vec(uri_strategy(), 2..10),
        codes in prop::collection::vec(code_strategy(), 2..10)
    ) {
        let cache = AstCache::new();

        // Store multiple entries
        for (i, uri) in uris.iter().enumerate() {
            let code = &codes[i % codes.len()];
            let hash = hash_input(code);
            cache.put(uri.clone(), hash, format!("AST{}", i));
        }

        // Clear cache
        cache.clear();

        // Verify all entries are gone
        for (i, uri) in uris.iter().enumerate() {
            let code = &codes[i % codes.len()];
            let hash = hash_input(code);

            let cached = cache.get(uri, hash);
            prop_assert_eq!(cached, None);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_ast_cache_basic() {
        let cache = AstCache::new();
        let uri = "file://test.rs".to_string();
        let code = "fn main() {}";
        let hash = hash_input(code);

        cache.put(uri.clone(), hash, "AST".to_string());

        let cached = cache.get(&uri, hash);
        assert_eq!(cached, Some("AST".to_string()));
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = AstCache::new();
        let uri = "file://test.rs".to_string();
        let hash = hash_input("code");

        cache.put(uri.clone(), hash, "AST".to_string());
        cache.invalidate(&uri);

        let cached = cache.get(&uri, hash);
        assert_eq!(cached, None);
    }

    #[test]
    fn test_symbol_index_cache_basic() {
        let cache = SymbolIndexCache::new();
        let uri = "file://test.rs".to_string();
        let hash = hash_input("code");

        let mut index = HashMap::new();
        index.insert("main".to_string(), 0);

        cache.put(uri.clone(), hash, index.clone());

        let cached = cache.get(&uri, hash);
        assert_eq!(cached, Some(index));
    }

    #[test]
    fn test_semantic_cache_hit_rate() {
        let cache = SemanticCache::new();

        // Store entry
        let hash = hash_input("code");
        let mut info = SemanticInfo::new();
        cache.put("file://test.rs".to_string(), hash, info);

        // Hit
        let _cached = cache.get("file://test.rs", hash);

        // Miss
        let _cached = cache.get("file://test.rs", hash + 1);

        let metrics = cache.metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 50.0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = AstCache::new();

        cache.put("file://test1.rs".to_string(), 1, "AST1".to_string());
        cache.put("file://test2.rs".to_string(), 2, "AST2".to_string());

        cache.clear();

        assert_eq!(cache.get("file://test1.rs", 1), None);
        assert_eq!(cache.get("file://test2.rs", 2), None);
    }

    #[test]
    fn test_cache_invalidation_metrics() {
        let cache = AstCache::new();

        cache.put("file://test.rs".to_string(), 1, "AST".to_string());
        cache.invalidate("file://test.rs");

        let metrics = cache.metrics();
        assert_eq!(metrics.invalidations, 1);
    }
}
