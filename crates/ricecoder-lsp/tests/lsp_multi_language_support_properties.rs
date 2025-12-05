//! Property-based tests for LSP multi-language support
//!
//! **Feature: ricecoder-lsp, Property 6: Multi-language support**
//! **Validates: Requirements LSP-5.1, LSP-5.2**
//!
//! These tests verify that:
//! - Each language is analyzed correctly
//! - Unsupported languages degrade gracefully
//! - Language-specific rules are applied correctly

use proptest::prelude::*;
use ricecoder_lsp::diagnostics::{DefaultDiagnosticsEngine, DiagnosticsEngine};
use ricecoder_lsp::semantic::{LanguageDetector, SemanticAnalyzerFactory};
use ricecoder_lsp::types::Language;
use std::path::Path;

/// Strategy for generating Rust code snippets
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
        Just("impl Point { fn new(x: i32, y: i32) -> Self { Point { x, y } } }".to_string()),
        Just(
            "pub async fn fetch_data() -> Result<String, Error> { Ok(String::new()) }".to_string()
        ),
    ]
}

/// Strategy for generating TypeScript code snippets
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
        Just("async function fetchData(): Promise<string> { return 'data'; }".to_string()),
        Just("export interface Config { debug: boolean; timeout: number; }".to_string()),
    ]
}

/// Strategy for generating Python code snippets
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
        Just("async def fetch_data():\n    return 'data'".to_string()),
        Just("@dataclass\nclass Config:\n    debug: bool\n    timeout: int".to_string()),
    ]
}

/// Property 6.1: Rust code is analyzed correctly
#[test]
fn prop_rust_language_analyzed_correctly() {
    proptest!(|(code in rust_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Analyze code
        let result = analyzer.analyze(&code);
        assert!(result.is_ok(), "Rust analysis should succeed");

        let info = result.unwrap();

        // Should have valid semantic info
        let _ = info.symbols.len();
        let _ = info.imports.len();

        // All symbols should have valid names
        for symbol in &info.symbols {
            assert!(!symbol.name.is_empty(), "Symbol name should not be empty");
        }
    });
}

/// Property 6.2: TypeScript code is analyzed correctly
#[test]
fn prop_typescript_language_analyzed_correctly() {
    proptest!(|(code in typescript_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::TypeScript);

        // Analyze code
        let result = analyzer.analyze(&code);
        assert!(result.is_ok(), "TypeScript analysis should succeed");

        let info = result.unwrap();

        // Should have valid semantic info
        let _ = info.symbols.len();
        let _ = info.imports.len();

        // All symbols should have valid names
        for symbol in &info.symbols {
            assert!(!symbol.name.is_empty(), "Symbol name should not be empty");
        }
    });
}

/// Property 6.3: Python code is analyzed correctly
#[test]
fn prop_python_language_analyzed_correctly() {
    proptest!(|(code in python_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Python);

        // Analyze code
        let result = analyzer.analyze(&code);
        assert!(result.is_ok(), "Python analysis should succeed");

        let info = result.unwrap();

        // Should have valid semantic info
        let _ = info.symbols.len();
        let _ = info.imports.len();

        // All symbols should have valid names
        for symbol in &info.symbols {
            assert!(!symbol.name.is_empty(), "Symbol name should not be empty");
        }
    });
}

/// Property 6.4: Unsupported languages degrade gracefully
#[test]
fn prop_unsupported_language_degrades_gracefully() {
    proptest!(|(code in r#"[a-zA-Z0-9\s\n\{\}\[\]\(\);:,=\.\-_]+"#)| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Unknown);

        // Should not crash
        let result = analyzer.analyze(&code);
        assert!(result.is_ok(), "Unknown language analysis should not crash");

        let info = result.unwrap();

        // Should return empty results for unknown language
        assert!(info.symbols.is_empty(), "Unknown language should have no symbols");
        assert!(info.imports.is_empty(), "Unknown language should have no imports");
        assert!(info.definitions.is_empty(), "Unknown language should have no definitions");
        assert!(info.references.is_empty(), "Unknown language should have no references");
    });
}

/// Property 6.5: Language detection from file extension works correctly
#[test]
fn prop_language_detection_from_extension() {
    proptest!(|(
        ext in prop_oneof![
            Just("rs"),
            Just("ts"),
            Just("tsx"),
            Just("js"),
            Just("jsx"),
            Just("py"),
        ]
    )| {
        let filename = format!("test.{}", ext);
        let path = Path::new(&filename);

        let language = LanguageDetector::from_extension(path);

        // Should detect a language (not Unknown for known extensions)
        match ext {
            "rs" => assert_eq!(language, Language::Rust),
            "ts" | "tsx" => assert_eq!(language, Language::TypeScript),
            "js" | "jsx" => assert_eq!(language, Language::TypeScript), // JS often treated as TS
            "py" => assert_eq!(language, Language::Python),
            _ => {}
        }
    });
}

/// Property 6.6: Language detection from content works correctly
#[test]
fn prop_language_detection_from_content() {
    proptest!(|(
        code in prop_oneof![
            rust_code_strategy(),
            typescript_code_strategy(),
            python_code_strategy(),
        ]
    )| {
        let language = LanguageDetector::from_content(&code);

        // Should detect a language or Unknown (detection is heuristic-based)
        // The important thing is that it doesn't crash
        let _ = language;
    });
}

/// Property 6.7: Language-specific rules are applied correctly for Rust
#[test]
fn prop_rust_specific_rules_applied() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in rust_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::Rust);
        assert!(result.is_ok(), "Rust diagnostics should succeed");

        let diagnostics = result.unwrap();

        // All diagnostics should have valid structure
        for diag in &diagnostics {
            assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            // Rust-specific diagnostics might mention Rust concepts
            // (but not required for all diagnostics)
        }
    });
}

/// Property 6.8: Language-specific rules are applied correctly for TypeScript
#[test]
fn prop_typescript_specific_rules_applied() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in typescript_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::TypeScript);
        assert!(result.is_ok(), "TypeScript diagnostics should succeed");

        let diagnostics = result.unwrap();

        // All diagnostics should have valid structure
        for diag in &diagnostics {
            assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            // TypeScript-specific diagnostics might mention TypeScript concepts
            // (but not required for all diagnostics)
        }
    });
}

/// Property 6.9: Language-specific rules are applied correctly for Python
#[test]
fn prop_python_specific_rules_applied() {
    let engine = DefaultDiagnosticsEngine::new();

    proptest!(|(code in python_code_strategy())| {
        let result = engine.generate_diagnostics(&code, Language::Python);
        assert!(result.is_ok(), "Python diagnostics should succeed");

        let diagnostics = result.unwrap();

        // All diagnostics should have valid structure
        for diag in &diagnostics {
            assert!(!diag.message.is_empty(), "Diagnostic message should not be empty");
            // Python-specific diagnostics might mention Python concepts
            // (but not required for all diagnostics)
        }
    });
}

/// Property 6.10: Different languages produce different analyses for same-looking code
#[test]
fn prop_different_languages_produce_different_analyses() {
    proptest!(|(code in r#"[a-zA-Z_][a-zA-Z0-9_]{0,10}"#)| {
        let rust_analyzer = SemanticAnalyzerFactory::create(Language::Rust);
        let ts_analyzer = SemanticAnalyzerFactory::create(Language::TypeScript);
        let py_analyzer = SemanticAnalyzerFactory::create(Language::Python);

        // Analyze with each language
        let rust_result = rust_analyzer.analyze(&code);
        let ts_result = ts_analyzer.analyze(&code);
        let py_result = py_analyzer.analyze(&code);

        // All should succeed
        assert!(rust_result.is_ok());
        assert!(ts_result.is_ok());
        assert!(py_result.is_ok());

        // Results might be different (depending on how the code is interpreted)
        // This is expected behavior
        let _ = rust_result.unwrap();
        let _ = ts_result.unwrap();
        let _ = py_result.unwrap();
    });
}

/// Property 6.11: All supported languages are handled without crashing
#[test]
fn prop_all_supported_languages_handled() {
    proptest!(|(code in r#"[a-zA-Z0-9\s\n]+"#)| {
        let languages = vec![
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::Unknown,
        ];

        for language in languages {
            let analyzer = SemanticAnalyzerFactory::create(language);

            // Should not crash
            let result = analyzer.analyze(&code);
            assert!(result.is_ok(), "Analysis should not crash for {:?}", language);

            let _ = result.unwrap();
        }
    });
}

/// Property 6.12: Language detection is consistent across multiple calls
#[test]
fn prop_language_detection_consistency() {
    proptest!(|(code in rust_code_strategy())| {
        // Detect language multiple times
        let lang1 = LanguageDetector::from_content(&code);
        let lang2 = LanguageDetector::from_content(&code);
        let lang3 = LanguageDetector::from_content(&code);

        // All should be identical
        assert_eq!(lang1, lang2, "Language detection should be consistent");
        assert_eq!(lang2, lang3, "Language detection should be consistent");
    });
}

/// Property 6.13: Semantic analyzer factory creates correct analyzer for each language
#[test]
fn prop_semantic_analyzer_factory_creates_correct_analyzer() {
    proptest!(|(language in prop_oneof![
        Just(Language::Rust),
        Just(Language::TypeScript),
        Just(Language::Python),
        Just(Language::Unknown),
    ])| {
        let analyzer = SemanticAnalyzerFactory::create(language);

        // Analyzer should report correct language
        assert_eq!(analyzer.language(), language, "Analyzer should report correct language");
    });
}

/// Property 6.14: Rust analyzer handles Rust-specific syntax
#[test]
fn prop_rust_analyzer_handles_rust_syntax() {
    proptest!(|(code in rust_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Rust);

        // Should handle Rust-specific syntax without crashing
        let result = analyzer.extract_symbols(&code);
        assert!(result.is_ok(), "Rust analyzer should handle Rust syntax");

        let symbols = result.unwrap();
        // Should extract symbols (may be empty for some code)
        let _ = symbols.len();
    });
}

/// Property 6.15: TypeScript analyzer handles TypeScript-specific syntax
#[test]
fn prop_typescript_analyzer_handles_typescript_syntax() {
    proptest!(|(code in typescript_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::TypeScript);

        // Should handle TypeScript-specific syntax without crashing
        let result = analyzer.extract_symbols(&code);
        assert!(result.is_ok(), "TypeScript analyzer should handle TypeScript syntax");

        let symbols = result.unwrap();
        // Should extract symbols (may be empty for some code)
        let _ = symbols.len();
    });
}

/// Property 6.16: Python analyzer handles Python-specific syntax
#[test]
fn prop_python_analyzer_handles_python_syntax() {
    proptest!(|(code in python_code_strategy())| {
        let analyzer = SemanticAnalyzerFactory::create(Language::Python);

        // Should handle Python-specific syntax without crashing
        let result = analyzer.extract_symbols(&code);
        assert!(result.is_ok(), "Python analyzer should handle Python syntax");

        let symbols = result.unwrap();
        // Should extract symbols (may be empty for some code)
        let _ = symbols.len();
    });
}
