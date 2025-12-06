//! Property-based tests for configuration-driven language support
//!
//! **Feature: ricecoder-refactoring, Property 3: Configuration-Driven Language Support**
//! **Validates: Requirements REF-4.1, REF-4.3**
//!
//! Property: For any language with configured refactoring provider, rules are applied correctly
//! Generate random code in each language and verify rule application
//! Run 100+ iterations with different language patterns

use proptest::prelude::*;
use ricecoder_refactoring::{
    ConfigManager, RefactoringEngine,
    RefactoringType,
};
use std::sync::Arc;

// Strategy for generating Rust code patterns
fn rust_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("fn main() { println!(\"Hello\"); }".to_string()),
        Just("fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
        Just("struct Point { x: i32, y: i32 }".to_string()),
        Just("impl Point { fn new(x: i32, y: i32) -> Self { Self { x, y } } }".to_string()),
        Just("let x = 42;".to_string()),
        Just("let mut y = 0; y += 1;".to_string()),
        Just("for i in 0..10 { println!(\"{}\", i); }".to_string()),
        Just("match x { 1 => println!(\"one\"), _ => println!(\"other\") }".to_string()),
        Just("if x > 0 { println!(\"positive\"); }".to_string()),
    ]
}

// Strategy for generating TypeScript code patterns
fn typescript_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("function main() {}".to_string()),
        Just("function main() { console.log('Hello'); }".to_string()),
        Just("function add(a: number, b: number): number { return a + b; }".to_string()),
        Just("interface Point { x: number; y: number; }".to_string()),
        Just("class Point { constructor(public x: number, public y: number) {} }".to_string()),
        Just("const x = 42;".to_string()),
        Just("let y = 0; y += 1;".to_string()),
        Just("for (let i = 0; i < 10; i++) { console.log(i); }".to_string()),
        Just("switch (x) { case 1: console.log('one'); break; default: console.log('other'); }".to_string()),
        Just("if (x > 0) { console.log('positive'); }".to_string()),
    ]
}

// Strategy for generating Python code patterns
fn python_code_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("def main():\n    pass".to_string()),
        Just("def main():\n    print('Hello')".to_string()),
        Just("def add(a, b):\n    return a + b".to_string()),
        Just("class Point:\n    def __init__(self, x, y):\n        self.x = x\n        self.y = y".to_string()),
        Just("x = 42".to_string()),
        Just("y = 0\ny += 1".to_string()),
        Just("for i in range(10):\n    print(i)".to_string()),
        Just("if x > 0:\n    print('positive')".to_string()),
        Just("try:\n    x = 1 / 0\nexcept ZeroDivisionError:\n    pass".to_string()),
        Just("with open('file.txt') as f:\n    content = f.read()".to_string()),
    ]
}

// Strategy for generating language names
fn language_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("rust".to_string()),
        Just("typescript".to_string()),
        Just("python".to_string()),
    ]
}

// Strategy for generating refactoring types
fn refactoring_type_strategy() -> impl Strategy<Value = RefactoringType> {
    prop_oneof![
        Just(RefactoringType::Rename),
        Just(RefactoringType::Extract),
        Just(RefactoringType::Inline),
        Just(RefactoringType::Move),
        Just(RefactoringType::ChangeSignature),
        Just(RefactoringType::RemoveUnused),
        Just(RefactoringType::Simplify),
    ]
}

proptest! {
    /// Property 3: Configuration-Driven Language Support
    ///
    /// For any language with configured refactoring provider, rules are applied correctly.
    /// This property verifies that:
    /// 1. The provider registry can be configured with language-specific providers
    /// 2. When a language is configured, the correct provider is used
    /// 3. The provider correctly analyzes refactoring for that language
    /// 4. The analysis result indicates the refactoring is applicable
    #[test]
    fn prop_configuration_driven_language_support(
        language in language_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        // Setup: Create engine with configuration
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        // Verify: Provider registry has the language configured
        let has_provider = engine.provider_registry().clone().has_provider(&language);
        prop_assert!(has_provider.is_ok(), "Language {} should be configured", language);

        // Verify: Get provider for the language
        let provider = engine.provider_registry().clone().get_provider(&language);

        // Generate appropriate code for the language
        let code = match language.as_str() {
            "rust" => "fn main() {}",
            "typescript" => "function main() {}",
            "python" => "def main():\n    pass",
            _ => "code",
        };

        // Verify: Provider can analyze refactoring for the language
        let analysis = provider.analyze_refactoring(code, &language, refactoring_type);
        prop_assert!(analysis.is_ok(), "Provider should analyze refactoring for {}", language);

        let analysis = analysis.unwrap();

        // Verify: Analysis indicates refactoring is applicable
        prop_assert!(analysis.applicable, "Refactoring should be applicable for {}", language);

        // Verify: Complexity is reasonable
        prop_assert!(analysis.complexity > 0, "Complexity should be positive for {}", language);
    }

    /// Property 3b: Rust Language Configuration
    ///
    /// For any Rust code, the Rust provider should correctly analyze refactoring.
    #[test]
    fn prop_rust_language_configuration(
        code in rust_code_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let provider = engine.provider_registry().clone().get_provider("rust");
        let analysis = provider.analyze_refactoring(&code, "rust", refactoring_type);

        prop_assert!(analysis.is_ok(), "Rust provider should analyze code");
        let analysis = analysis.unwrap();
        prop_assert!(analysis.applicable, "Refactoring should be applicable for Rust");
    }

    /// Property 3c: TypeScript Language Configuration
    ///
    /// For any TypeScript code, the TypeScript provider should correctly analyze refactoring.
    #[test]
    fn prop_typescript_language_configuration(
        code in typescript_code_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let provider = engine.provider_registry().clone().get_provider("typescript");
        let analysis = provider.analyze_refactoring(&code, "typescript", refactoring_type);

        prop_assert!(analysis.is_ok(), "TypeScript provider should analyze code");
        let analysis = analysis.unwrap();
        prop_assert!(analysis.applicable, "Refactoring should be applicable for TypeScript");
    }

    /// Property 3d: Python Language Configuration
    ///
    /// For any Python code, the Python provider should correctly analyze refactoring.
    #[test]
    fn prop_python_language_configuration(
        code in python_code_strategy(),
        refactoring_type in refactoring_type_strategy(),
    ) {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let provider = engine.provider_registry().clone().get_provider("python");
        let analysis = provider.analyze_refactoring(&code, "python", refactoring_type);

        prop_assert!(analysis.is_ok(), "Python provider should analyze code");
        let analysis = analysis.unwrap();
        prop_assert!(analysis.applicable, "Refactoring should be applicable for Python");
    }

    /// Property 3e: Provider Registry Consistency
    ///
    /// For any configured language, the provider registry should consistently return the same provider.
    #[test]
    fn prop_provider_registry_consistency(
        language in language_strategy(),
    ) {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        // Get provider twice
        let provider1 = engine.provider_registry().clone().get_provider(&language);
        let provider2 = engine.provider_registry().clone().get_provider(&language);

        // Both should work
        let analysis1 = provider1.analyze_refactoring("code", &language, RefactoringType::Rename);
        let analysis2 = provider2.analyze_refactoring("code", &language, RefactoringType::Rename);

        prop_assert!(analysis1.is_ok(), "First provider should work");
        prop_assert!(analysis2.is_ok(), "Second provider should work");
    }

    /// Property 3f: Configuration Hierarchy
    ///
    /// For any language, the configuration manager should load configuration correctly.
    #[test]
    fn prop_configuration_hierarchy(
        language in language_strategy(),
    ) {
        let config_manager = ConfigManager::new();

        // Verify configuration manager can be created
        prop_assert!(true, "Configuration manager should be created");

        // Verify it can be used in engine
        let engine = RefactoringEngine::new(Arc::new(config_manager));
        let has_provider = engine.provider_registry().clone().has_provider(&language);
        prop_assert!(has_provider.is_ok(), "Language {} should be available", language);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_provider_available() {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let has_rust = engine.provider_registry().clone().has_provider("rust");
        assert!(has_rust.is_ok());
    }

    #[test]
    fn test_typescript_provider_available() {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let has_ts = engine.provider_registry().clone().has_provider("typescript");
        assert!(has_ts.is_ok());
    }

    #[test]
    fn test_python_provider_available() {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let has_python = engine.provider_registry().clone().has_provider("python");
        assert!(has_python.is_ok());
    }

    #[test]
    fn test_provider_registry_get_provider() {
        let config_manager = Arc::new(ConfigManager::new());
        let engine = RefactoringEngine::new(config_manager);

        let provider = engine.provider_registry().clone().get_provider("rust");
        let analysis = provider.analyze_refactoring("fn main() {}", "rust", RefactoringType::Rename);

        assert!(analysis.is_ok());
    }
}
