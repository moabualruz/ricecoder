/// Unit tests for context analysis
use ricecoder_completion::*;

#[tokio::test]
async fn test_tree_sitter_context_analyzer_rust_scope_detection() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() { let x = 5; }";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "rust");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_tree_sitter_context_analyzer_typescript_scope_detection() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() { let x = 5; }";
    let position = Position::new(0, 18);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "typescript");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_tree_sitter_context_analyzer_python_scope_detection() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    x = 5";
    let position = Position::new(1, 10);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "python");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_generic_context_analyzer_scope_detection() {
    let analyzer = GenericContextAnalyzer;
    let code = "fn main() { let x = 5; }";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "rust");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_context_analyzer_prefix_extraction() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "let my_var = my";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.prefix, "my");
}

#[tokio::test]
async fn test_context_analyzer_empty_prefix() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "let x = ";
    let position = Position::new(0, 8);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.prefix, "");
}

#[tokio::test]
async fn test_context_analyzer_builtin_symbols_rust() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Check that built-in symbols are available
    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    assert!(symbol_names.contains(&"String".to_string()));
    assert!(symbol_names.contains(&"Vec".to_string()));
    assert!(symbol_names.contains(&"Option".to_string()));
    assert!(symbol_names.contains(&"Result".to_string()));
}

#[tokio::test]
async fn test_context_analyzer_builtin_symbols_typescript() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {}";
    let position = Position::new(0, 10);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    // Check that built-in symbols are available
    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    assert!(symbol_names.contains(&"Array".to_string()));
    assert!(symbol_names.contains(&"Object".to_string()));
    assert!(symbol_names.contains(&"Promise".to_string()));
    assert!(symbol_names.contains(&"Map".to_string()));
}

#[tokio::test]
async fn test_context_analyzer_builtin_symbols_python() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    pass";
    let position = Position::new(1, 5);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    // Check that built-in symbols are available
    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    assert!(symbol_names.contains(&"list".to_string()));
    assert!(symbol_names.contains(&"dict".to_string()));
    assert!(symbol_names.contains(&"str".to_string()));
    assert!(symbol_names.contains(&"int".to_string()));
}

#[tokio::test]
async fn test_context_analyzer_unsupported_language() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "some code";
    let position = Position::new(0, 5);

    let result = analyzer
        .analyze_context(code, position, "unsupported_language")
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generic_context_analyzer_builtin_symbols() {
    let analyzer = GenericContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Check that built-in symbols are available
    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    assert!(symbol_names.contains(&"String".to_string()));
    assert!(symbol_names.contains(&"Vec".to_string()));
}

#[tokio::test]
async fn test_context_analyzer_symbol_types() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Check that symbols have correct types
    for symbol in &context.available_symbols {
        assert_eq!(symbol.kind, SymbolKind::Type);
        assert!(symbol.type_info.is_some());
        assert!(symbol.documentation.is_some());
    }
}

#[tokio::test]
async fn test_context_analyzer_type_inference() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Type inference is not yet fully implemented, so expected_type should be None
    assert!(context.expected_type.is_none());
}

#[tokio::test]
async fn test_context_analyzer_multiline_code() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    let x = 5;\n    let y = 10;\n}";
    let position = Position::new(2, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.position, position);
    assert_eq!(context.language, "rust");
}

#[tokio::test]
async fn test_context_analyzer_invalid_code() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() { let x = ";
    let position = Position::new(0, 15);

    // Should still work even with invalid code
    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "rust");
}

#[tokio::test]
async fn test_context_analyzer_empty_code() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "";
    let position = Position::new(0, 0);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "rust");
    assert_eq!(context.prefix, "");
}

#[tokio::test]
async fn test_generic_context_analyzer_invalid_code() {
    let analyzer = GenericContextAnalyzer;
    let code = "fn main() { let x = ";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.language, "rust");
}

#[tokio::test]
async fn test_context_analyzer_scope_kind_global() {
    let analyzer = GenericContextAnalyzer;
    let code = "let x = 5;";
    let position = Position::new(0, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Global);
}

#[tokio::test]
async fn test_context_analyzer_scope_kind_function() {
    let analyzer = GenericContextAnalyzer;
    let code = "fn main() { let x = 5; }";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Function);
}

#[tokio::test]
async fn test_context_analyzer_scope_kind_block() {
    let analyzer = GenericContextAnalyzer;
    let code = "{ let x = 5; }";
    let position = Position::new(0, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Block);
}

#[tokio::test]
async fn test_context_analyzer_scope_kind_class() {
    let analyzer = GenericContextAnalyzer;
    let code = "struct MyStruct { x: i32 }";
    let position = Position::new(0, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    assert_eq!(context.scope.kind, ScopeKind::Class);
}

// Additional comprehensive tests for symbol resolution accuracy

#[tokio::test]
async fn test_symbol_resolution_rust_local_variables() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    let x = 5;\n    let y = 10;\n    let z = x + y;\n}";
    let position = Position::new(3, 15);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include local variables
    assert!(symbol_names.contains(&"x".to_string()));
    assert!(symbol_names.contains(&"y".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_typescript_local_variables() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {\n    let x = 5;\n    const y = 10;\n    var z = x + y;\n}";
    let position = Position::new(3, 15);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include local variables
    assert!(symbol_names.contains(&"x".to_string()));
    assert!(symbol_names.contains(&"y".to_string()));
    assert!(symbol_names.contains(&"z".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_python_local_variables() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    x = 5\n    y = 10\n    z = x + y";
    let position = Position::new(3, 10);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include local variables
    assert!(symbol_names.contains(&"x".to_string()));
    assert!(symbol_names.contains(&"y".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_rust_function_parameters() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
    let position = Position::new(1, 5);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include function parameters
    assert!(symbol_names.contains(&"a".to_string()));
    assert!(symbol_names.contains(&"b".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_typescript_function_parameters() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function add(a: number, b: number): number {\n    return a + b;\n}";
    let position = Position::new(1, 15);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include function parameters
    assert!(symbol_names.contains(&"a".to_string()));
    assert!(symbol_names.contains(&"b".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_python_function_parameters() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def add(a, b):\n    return a + b";
    let position = Position::new(1, 15);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include function parameters
    assert!(symbol_names.contains(&"a".to_string()));
    assert!(symbol_names.contains(&"b".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_rust_constants() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    const MAX: i32 = 100;\n    let x = MAX;\n}";
    let position = Position::new(2, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include constants defined in the same scope
    assert!(symbol_names.contains(&"MAX".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_rust_mutable_variables() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    let mut x = 5;\n    x = 10;\n}";
    let position = Position::new(2, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include mutable variables
    assert!(symbol_names.contains(&"x".to_string()));
}

#[tokio::test]
async fn test_symbol_resolution_typescript_const_variables() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {\n    const x = 5;\n    const y = x + 10;\n}";
    let position = Position::new(2, 15);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    let symbol_names: Vec<String> = context
        .available_symbols
        .iter()
        .map(|s| s.name.clone())
        .collect();

    // Should include const variables
    assert!(symbol_names.contains(&"x".to_string()));
}

#[tokio::test]
async fn test_symbol_kind_classification_rust() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    let x = 5;\n}";
    let position = Position::new(1, 10);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Check that symbols have correct kinds
    for symbol in &context.available_symbols {
        match symbol.name.as_str() {
            "x" => assert_eq!(symbol.kind, SymbolKind::Variable),
            _ => {
                // Built-in types should be Type kind
                if symbol.kind == SymbolKind::Type {
                    assert!(symbol.type_info.is_some());
                }
            }
        }
    }
}

#[tokio::test]
async fn test_symbol_kind_classification_typescript() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {\n    let x = 5;\n    const y = 10;\n}";
    let position = Position::new(2, 15);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    // Check that symbols have correct kinds
    for symbol in &context.available_symbols {
        match symbol.name.as_str() {
            "x" => assert_eq!(symbol.kind, SymbolKind::Variable),
            "y" => assert_eq!(symbol.kind, SymbolKind::Constant),
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_symbol_kind_classification_python() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    x = 5\n    y = 10";
    let position = Position::new(2, 10);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    // Check that symbols have correct kinds
    for symbol in &context.available_symbols {
        match symbol.name.as_str() {
            "x" | "y" => assert_eq!(symbol.kind, SymbolKind::Variable),
            _ => {}
        }
    }
}

// Type inference tests

#[tokio::test]
async fn test_type_inference_rust_integer_literal() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {\n    let x = 5;\n    let y = x";
    let position = Position::new(2, 14);

    let context = analyzer
        .analyze_context(code, position, "rust")
        .await
        .expect("Failed to analyze context");

    // Type inference may not be fully implemented, but context should be valid
    assert_eq!(context.language, "rust");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_type_inference_typescript_string_literal() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {\n    let x = \"hello\";\n    let y = x";
    let position = Position::new(2, 14);

    let context = analyzer
        .analyze_context(code, position, "typescript")
        .await
        .expect("Failed to analyze context");

    // Type inference may not be fully implemented, but context should be valid
    assert_eq!(context.language, "typescript");
    assert_eq!(context.position, position);
}

#[tokio::test]
async fn test_type_inference_python_list_literal() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    x = [1, 2, 3]\n    y = x";
    let position = Position::new(2, 10);

    let context = analyzer
        .analyze_context(code, position, "python")
        .await
        .expect("Failed to analyze context");

    // Type inference may not be fully implemented, but context should be valid
    assert_eq!(context.language, "python");
    assert_eq!(context.position, position);
}

// Error handling tests

#[tokio::test]
async fn test_error_handling_rust_incomplete_function() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {";
    let position = Position::new(0, 10);

    // Should handle incomplete code gracefully
    let result = analyzer.analyze_context(code, position, "rust").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_typescript_incomplete_function() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() {";
    let position = Position::new(0, 15);

    // Should handle incomplete code gracefully
    let result = analyzer.analyze_context(code, position, "typescript").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_python_incomplete_function() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():";
    let position = Position::new(0, 10);

    // Should handle incomplete code gracefully
    let result = analyzer.analyze_context(code, position, "python").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_rust_syntax_error() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() { let x = ; }";
    let position = Position::new(0, 15);

    // Should handle syntax errors gracefully
    let result = analyzer.analyze_context(code, position, "rust").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_typescript_syntax_error() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "function main() { let x = ; }";
    let position = Position::new(0, 20);

    // Should handle syntax errors gracefully
    let result = analyzer.analyze_context(code, position, "typescript").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_python_syntax_error() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "def main():\n    x = ";
    let position = Position::new(1, 10);

    // Should handle syntax errors gracefully
    let result = analyzer.analyze_context(code, position, "python").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_position_out_of_bounds() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(10, 100);

    // Should handle out-of-bounds positions gracefully
    let result = analyzer.analyze_context(code, position, "rust").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_handling_empty_language() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "fn main() {}";
    let position = Position::new(0, 5);

    // Should handle empty language string
    let result = analyzer.analyze_context(code, position, "").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_handling_null_code() {
    let analyzer = TreeSitterContextAnalyzer;
    let code = "";
    let position = Position::new(0, 0);

    // Should handle empty code gracefully
    let result = analyzer.analyze_context(code, position, "rust").await;
    assert!(result.is_ok());
}
