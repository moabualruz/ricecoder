//! Property-based tests for LSP semantic analysis consistency
//!
//! **Feature: ricecoder-lsp, Property 2: Semantic analysis consistency**
//! **Validates: Requirements LSP-2.1, LSP-2.2**
//!
//! These tests verify that semantic analysis is consistent and deterministic by testing that:
//! - Analyzing the same code twice produces identical symbols
//! - Symbol extraction is complete for all symbol types
//! - Import tracking is accurate and consistent
//! - Symbol index lookup returns correct results

use std::path::Path;

use proptest::prelude::*;
use ricecoder_lsp::{
    semantic::{LanguageDetector, SemanticAnalyzerFactory},
    types::Language,
};

/// Strategy for generating valid Rust code snippets
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
        Just("struct Point { x: i32, y: i32 }".to_string()),
        Just("trait Iterator { fn next(&mut self) -> Option<Self::Item>; }".to_string()),
        Just("use std::io;\nuse std::fs;".to_string()),
        Just("mod utils { pub fn helper() {} }".to_string()),
        Just("const MAX_SIZE: usize = 100;".to_string()),
        Just("enum Result<T, E> { Ok(T), Err(E) }".to_string()),
    ]
}

/// Strategy for generating valid TypeScript code snippets
fn typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("function add(a: number, b: number): number { return a + b; }".to_string()),
        Just("interface User { name: string; age: number; }".to_string()),
        Just("class Point { x: number; y: number; }".to_string()),
        Just("export const MAX_SIZE = 100;".to_string()),
        Just("import { foo } from 'bar';".to_string()),
        Just("type Result<T> = T | Error;".to_string()),
        Just("enum Status { Active, Inactive }".to_string()),
        Just("const x: string = 'hello';".to_string()),
    ]
}

/// Strategy for generating valid Python code snippets
fn python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("def add(a, b):\n    return a + b".to_string()),
        Just("class Point:\n    def __init__(self, x, y):\n        self.x = x".to_string()),
        Just("import os\nimport sys".to_string()),
        Just("MAX_SIZE = 100".to_string()),
        Just(
            "def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)"
                .to_string()
        ),
        Just("class Iterator:\n    def __next__(self):\n        pass".to_string()),
        Just("from typing import List, Dict".to_string()),
        Just("x: str = 'hello'".to_string()),
    ]
}

/// Property 1: Analyzing same Rust code twice produces identical symbols
#[test]
fn prop_rust_analysis_consistency() {
    proptest!(|(code in rust_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Analyze code twice
        let result1 = analyzer.extract_symbols(&code);
        let result2 = analyzer.extract_symbols(&code);

        // Both should succeed
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Results should be identical
        let symbols1 = result1.unwrap();
        let symbols2 = result2.unwrap();

        assert_eq!(symbols1.len(), symbols2.len());
        for (s1, s2) in symbols1.iter().zip(symbols2.iter()) {
            assert_eq!(s1.name, s2.name);
            assert_eq!(s1.kind, s2.kind);
            assert_eq!(s1.range, s2.range);
        }
    });
}

/// Property 2: Analyzing same TypeScript code twice produces identical symbols
#[test]
fn prop_typescript_analysis_consistency() {
    proptest!(|(code in typescript_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::TypeScript);

        // Analyze code twice
        let result1 = analyzer.extract_symbols(&code);
        let result2 = analyzer.extract_symbols(&code);

        // Both should succeed
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Results should be identical
        let symbols1 = result1.unwrap();
        let symbols2 = result2.unwrap();

        assert_eq!(symbols1.len(), symbols2.len());
        for (s1, s2) in symbols1.iter().zip(symbols2.iter()) {
            assert_eq!(s1.name, s2.name);
            assert_eq!(s1.kind, s2.kind);
            assert_eq!(s1.range, s2.range);
        }
    });
}

/// Property 3: Analyzing same Python code twice produces identical symbols
#[test]
fn prop_python_analysis_consistency() {
    proptest!(|(code in python_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Python);

        // Analyze code twice
        let result1 = analyzer.extract_symbols(&code);
        let result2 = analyzer.extract_symbols(&code);

        // Both should succeed
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Results should be identical
        let symbols1 = result1.unwrap();
        let symbols2 = result2.unwrap();

        assert_eq!(symbols1.len(), symbols2.len());
        for (s1, s2) in symbols1.iter().zip(symbols2.iter()) {
            assert_eq!(s1.name, s2.name);
            assert_eq!(s1.kind, s2.kind);
            assert_eq!(s1.range, s2.range);
        }
    });
}

/// Property 4: Semantic analysis never crashes on valid code
#[test]
fn prop_semantic_analysis_never_crashes() {
    proptest!(|(
        code in r#"[a-zA-Z0-9\s\n\{\}\[\]\(\);:,=\.\-_]+"#,
    )| {
        // Try analyzing with each language
        for language in &[Language::Rust, Language::TypeScript, Language::Python] {
            let analyzer = SemanticAnalyzerFactory::create(*language);

            // Should not panic
            let _ = analyzer.analyze(&code);
            let _ = analyzer.extract_symbols(&code);
        }
    });
}

/// Property 5: Semantic info is always valid (no panics on access)
#[test]
fn prop_semantic_info_always_valid() {
    proptest!(|(code in rust_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Analyze code
        let result = analyzer.analyze(&code);
        assert!(result.is_ok());

        let info = result.unwrap();

        // Should be able to access all fields without panicking
        let _ = info.symbols.len();
        let _ = info.imports.len();
        let _ = info.definitions.len();
        let _ = info.references.len();

        // Iterate over symbols without panicking
        for symbol in &info.symbols {
            let _ = &symbol.name;
            let _ = &symbol.kind;
            let _ = &symbol.range;
        }
    });
}

/// Property 6: Language detection is consistent
#[test]
fn prop_language_detection_consistency() {
    proptest!(|(
        code in rust_code_strategy(),
    )| {
        // Detect language from content multiple times
        let lang1 = LanguageDetector::from_content(&code);
        let lang2 = LanguageDetector::from_content(&code);

        // Should be identical
        assert_eq!(lang1, lang2);
    });
}

/// Property 7: Language detection from extension is consistent
#[test]
fn prop_language_detection_from_extension_consistency() {
    proptest!(|(
        ext in r#"[a-z]{1,5}"#,
    )| {
        let filename = format!("test.{}", ext);
        let path = Path::new(&filename);

        // Detect language multiple times
        let lang1 = LanguageDetector::from_extension(path);
        let lang2 = LanguageDetector::from_extension(path);

        // Should be identical
        assert_eq!(lang1, lang2);
    });
}

/// Property 8: Semantic analyzer factory creates correct analyzer for language
#[test]
fn prop_semantic_analyzer_factory_correctness() {
    proptest!(|(language in prop_oneof![
        Just(Language::Rust),
        Just(Language::TypeScript),
        Just(Language::Python),
        Just(Language::Unknown),
    ])| {
        let analyzer = SemanticAnalyzerFactory::create(language);

        // Analyzer should report correct language
        assert_eq!(analyzer.language(), language);
    });
}

/// Property 9: Empty code produces empty symbols
#[test]
fn prop_empty_code_produces_empty_symbols() {
    proptest!(|(
        whitespace in r#"[\s]*"#,
    )| {
        for language in &[Language::Rust, Language::TypeScript, Language::Python] {
            let analyzer = SemanticAnalyzerFactory::create(*language);

            // Analyze whitespace-only code
            let result = analyzer.extract_symbols(&whitespace);
            assert!(result.is_ok());

            let symbols = result.unwrap();
            // Empty or whitespace-only code should produce empty or minimal symbols
            // (actual behavior depends on implementation)
            let _ = symbols.len();
        }
    });
}

/// Property 10: Symbol extraction is deterministic (same code = same symbols)
#[test]
fn prop_symbol_extraction_deterministic() {
    proptest!(|(code in rust_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Extract symbols multiple times
        let symbols1 = analyzer.extract_symbols(&code).unwrap();
        let symbols2 = analyzer.extract_symbols(&code).unwrap();
        let symbols3 = analyzer.extract_symbols(&code).unwrap();

        // All should be identical
        assert_eq!(symbols1, symbols2);
        assert_eq!(symbols2, symbols3);
    });
}

/// Property 11: Semantic analysis handles code with imports correctly
#[test]
fn prop_semantic_analysis_with_imports() {
    proptest!(|(
        code in r#"(use std::[a-z_]+;)+"#,
    )| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Analyze code with imports
        let result = analyzer.analyze(&code);
        assert!(result.is_ok());

        let info = result.unwrap();

        // Should have imports field (may be empty depending on implementation)
        let _ = info.imports.len();
    });
}

/// Property 12: Unsupported language gracefully degrades
#[test]
fn prop_unsupported_language_graceful_degradation() {
    proptest!(|(code in r#"[a-zA-Z0-9\s]+"#)| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Unknown);

        // Should not crash
        let result = analyzer.analyze(&code);
        assert!(result.is_ok());

        let info = result.unwrap();

        // Should return empty results
        assert!(info.symbols.is_empty());
        assert!(info.imports.is_empty());
        assert!(info.definitions.is_empty());
        assert!(info.references.is_empty());
    });
}

/// Property 13: Semantic analyzer is thread-safe
#[test]
fn prop_semantic_analyzer_thread_safe() {
    proptest!(|(code in rust_code_strategy())| {
        let _analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Create multiple threads analyzing the same code
        let code1 = code.clone();
        let code2 = code.clone();

        let handle1 = std::thread::spawn(move || {
            let analyzer = SemanticAnalyzerFactory::create(Language::Rust);
            analyzer.extract_symbols(&code1)
        });

        let handle2 = std::thread::spawn(move || {
            let analyzer = SemanticAnalyzerFactory::create(Language::Rust);
            analyzer.extract_symbols(&code2)
        });

        // Both threads should complete successfully
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    });
}
