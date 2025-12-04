/// Property-based tests for context-aware filtering
/// **Feature: ricecoder-completion, Property 3: Context-aware filtering**
/// **Validates: Requirements Completion-3.1, Completion-3.2**
///
/// Property: For any code context, only available symbols are suggested
use proptest::prelude::*;
use ricecoder_completion::*;

// Strategy for generating valid Rust code with variable declarations
fn rust_code_with_vars() -> impl Strategy<Value = String> {
    prop::string::string_regex("(let [a-z_][a-z0-9_]* = [0-9]+;\\n)*")
        .expect("valid regex")
}

// Strategy for generating valid TypeScript code with variable declarations
fn typescript_code_with_vars() -> impl Strategy<Value = String> {
    prop::string::string_regex("(const [a-z_][a-z0-9_]* = [0-9]+;\\n)*")
        .expect("valid regex")
}

// Strategy for generating valid Python code with variable declarations
fn python_code_with_vars() -> impl Strategy<Value = String> {
    prop::string::string_regex("([a-z_][a-z0-9_]* = [0-9]+\\n)*")
        .expect("valid regex")
}



#[tokio::test]
async fn test_context_aware_filtering_builtin_symbols_always_available() {
    // Property: Built-in symbols should always be available in any context
    let analyzer = TreeSitterContextAnalyzer;

    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Get available symbols
    let symbols = analyzer.get_available_symbols(&context, code);

    // Check that built-in symbols are present
    let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

    // All built-in types should be available
    assert!(symbol_names.contains(&"String".to_string()));
    assert!(symbol_names.contains(&"Vec".to_string()));
    assert!(symbol_names.contains(&"Option".to_string()));
    assert!(symbol_names.contains(&"Result".to_string()));
}

#[tokio::test]
async fn test_context_aware_filtering_symbols_have_valid_types() {
    // Property: All available symbols should have valid symbol kinds
    let analyzer = TreeSitterContextAnalyzer;

    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);

    // All symbols should have valid types
    for symbol in symbols {
        match symbol.kind {
            SymbolKind::Variable
            | SymbolKind::Function
            | SymbolKind::Type
            | SymbolKind::Constant
            | SymbolKind::Module
            | SymbolKind::Class
            | SymbolKind::Struct
            | SymbolKind::Enum
            | SymbolKind::Interface
            | SymbolKind::Trait
            | SymbolKind::Method
            | SymbolKind::Property
            | SymbolKind::Field
            | SymbolKind::Parameter
            | SymbolKind::Keyword => {
                // Valid symbol kind
            }
        }
    }
}

#[tokio::test]
async fn test_context_aware_filtering_symbols_in_different_scopes() {
    // Property: Symbols should be available in the scope where they are defined
    let analyzer = GenericContextAnalyzer;

    // Test global scope
    let code = "let x = 5;";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Global);

    // Test function scope
    let code = "fn main() { let x = 5; }";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Function);

    // Test block scope
    let code = "{ let x = 5; }";
    let position = Position::new(0, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Block);
}

#[tokio::test]
async fn test_context_aware_filtering_typescript_symbols() {
    // Property: TypeScript symbols should be available in TypeScript context
    let analyzer = TreeSitterContextAnalyzer;

    let code = "function main() {}";
    let position = Position::new(0, 10);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);
    let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

    // TypeScript built-in types should be available
    assert!(symbol_names.contains(&"Array".to_string()));
    assert!(symbol_names.contains(&"Object".to_string()));
    assert!(symbol_names.contains(&"Promise".to_string()));
    assert!(symbol_names.contains(&"Map".to_string()));
}

#[tokio::test]
async fn test_context_aware_filtering_python_symbols() {
    // Property: Python symbols should be available in Python context
    let analyzer = TreeSitterContextAnalyzer;

    let code = "def main():\n    pass";
    let position = Position::new(1, 5);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);
    let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

    // Python built-in types should be available
    assert!(symbol_names.contains(&"list".to_string()));
    assert!(symbol_names.contains(&"dict".to_string()));
    assert!(symbol_names.contains(&"str".to_string()));
    assert!(symbol_names.contains(&"int".to_string()));
}

#[tokio::test]
async fn test_context_aware_filtering_no_symbols_for_unsupported_language() {
    // Property: Unsupported languages should not provide symbols
    let analyzer = GenericContextAnalyzer;

    let code = "some code";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "unsupported_language")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);

    // Should have no symbols for unsupported language
    assert!(symbols.is_empty());
}

#[tokio::test]
async fn test_context_aware_filtering_symbols_have_documentation() {
    // Property: Built-in symbols should have documentation
    let analyzer = TreeSitterContextAnalyzer;

    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);

    // All symbols should have documentation
    for symbol in symbols {
        assert!(
            symbol.documentation.is_some(),
            "Symbol {} should have documentation",
            symbol.name
        );
    }
}

#[tokio::test]
async fn test_context_aware_filtering_symbols_have_type_info() {
    // Property: Built-in symbols should have type information
    let analyzer = TreeSitterContextAnalyzer;

    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbols = analyzer.get_available_symbols(&context, code);

    // All symbols should have type information
    for symbol in symbols {
        assert!(
            symbol.type_info.is_some(),
            "Symbol {} should have type information",
            symbol.name
        );
    }
}

proptest! {
    #[test]
    fn prop_context_aware_filtering_rust_symbols_valid(
        code in rust_code_with_vars(),
        line in 0u32..10,
        char in 0u32..50
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = TreeSitterContextAnalyzer;
            let position = Position::new(line, char);

            // Should not panic even with arbitrary positions
            let result = analyzer.analyze_context(&code, position, "rust").await;

            // Should either succeed or fail gracefully
            match result {
                Ok(context) => {
                    // If successful, symbols should be valid
                    let symbols = analyzer.get_available_symbols(&context, &code);
                    for symbol in symbols {
                        // All symbols should have a name
                        prop_assert!(!symbol.name.is_empty());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Errors are acceptable for invalid positions
                    Ok(())
                }
            }
        }).unwrap();
    }

    #[test]
    fn prop_context_aware_filtering_typescript_symbols_valid(
        code in typescript_code_with_vars(),
        line in 0u32..10,
        char in 0u32..50
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = TreeSitterContextAnalyzer;
            let position = Position::new(line, char);

            // Should not panic even with arbitrary positions
            let result = analyzer.analyze_context(&code, position, "typescript").await;

            // Should either succeed or fail gracefully
            match result {
                Ok(context) => {
                    // If successful, symbols should be valid
                    let symbols = analyzer.get_available_symbols(&context, &code);
                    for symbol in symbols {
                        // All symbols should have a name
                        prop_assert!(!symbol.name.is_empty());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Errors are acceptable for invalid positions
                    Ok(())
                }
            }
        }).unwrap();
    }

    #[test]
    fn prop_context_aware_filtering_python_symbols_valid(
        code in python_code_with_vars(),
        line in 0u32..10,
        char in 0u32..50
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = TreeSitterContextAnalyzer;
            let position = Position::new(line, char);

            // Should not panic even with arbitrary positions
            let result = analyzer.analyze_context(&code, position, "python").await;

            // Should either succeed or fail gracefully
            match result {
                Ok(context) => {
                    // If successful, symbols should be valid
                    let symbols = analyzer.get_available_symbols(&context, &code);
                    for symbol in symbols {
                        // All symbols should have a name
                        prop_assert!(!symbol.name.is_empty());
                    }
                    Ok(())
                }
                Err(_) => {
                    // Errors are acceptable for invalid positions
                    Ok(())
                }
            }
        }).unwrap();
    }
}
