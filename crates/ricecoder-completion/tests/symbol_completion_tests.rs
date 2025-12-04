/// Tests for symbol completion suggestions
/// Tests that symbol completions include variables, functions, types with proper details
use ricecoder_completion::{
    CompletionContext, CompletionEngine, CompletionItem, CompletionItemKind, Position,
    GenericCompletionEngine, ProviderRegistry, Symbol, SymbolKind, Scope, ScopeKind,
    Range, Type, RustCompletionProvider, TypeScriptCompletionProvider, PythonCompletionProvider,
    GenericTextProvider, CompletionProvider,
};
use std::sync::Arc;
use async_trait::async_trait;

/// Mock context analyzer that provides test symbols
struct SymbolTestContextAnalyzer;

#[async_trait]
impl ricecoder_completion::ContextAnalyzer for SymbolTestContextAnalyzer {
    async fn analyze_context(
        &self,
        _code: &str,
        position: Position,
        language: &str,
    ) -> ricecoder_completion::CompletionResult<CompletionContext> {
        let mut context = CompletionContext::new(language.to_string(), position, "".to_string());
        
        // Add variable symbol with type
        context.available_symbols.push(Symbol {
            name: "my_var".to_string(),
            kind: SymbolKind::Variable,
            scope: Scope {
                kind: ScopeKind::Function,
                name: Some("main".to_string()),
                range: Range::new(Position::new(0, 0), Position::new(10, 0)),
            },
            type_info: Some("i32".to_string()),
            documentation: Some("A variable holding an integer".to_string()),
        });

        // Add function symbol with signature
        context.available_symbols.push(Symbol {
            name: "calculate".to_string(),
            kind: SymbolKind::Function,
            scope: Scope {
                kind: ScopeKind::Global,
                name: None,
                range: Range::new(Position::new(0, 0), Position::new(100, 0)),
            },
            type_info: Some("fn(i32, i32) -> i32".to_string()),
            documentation: Some("Calculates the sum of two numbers".to_string()),
        });

        // Add type symbol with documentation
        context.available_symbols.push(Symbol {
            name: "Point".to_string(),
            kind: SymbolKind::Struct,
            scope: Scope {
                kind: ScopeKind::Global,
                name: None,
                range: Range::new(Position::new(0, 0), Position::new(100, 0)),
            },
            type_info: Some("struct Point { x: i32, y: i32 }".to_string()),
            documentation: Some("A 2D point structure".to_string()),
        });

        // Add constant symbol
        context.available_symbols.push(Symbol {
            name: "MAX_SIZE".to_string(),
            kind: SymbolKind::Constant,
            scope: Scope {
                kind: ScopeKind::Global,
                name: None,
                range: Range::new(Position::new(0, 0), Position::new(100, 0)),
            },
            type_info: Some("const usize".to_string()),
            documentation: Some("Maximum size constant".to_string()),
        });

        Ok(context)
    }

    fn get_available_symbols(&self, context: &CompletionContext, _code: &str) -> Vec<Symbol> {
        context.available_symbols.clone()
    }

    fn infer_expected_type(&self, _context: &CompletionContext) -> Option<Type> {
        None
    }
}

/// Mock completion generator
struct SymbolTestCompletionGenerator;

#[async_trait]
impl ricecoder_completion::CompletionGenerator for SymbolTestCompletionGenerator {
    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        context: &CompletionContext,
    ) -> ricecoder_completion::CompletionResult<Vec<CompletionItem>> {
        let mut items = Vec::new();

        // Generate completions from available symbols
        for symbol in &context.available_symbols {
            let mut item = CompletionItem::new(
                symbol.name.clone(),
                match symbol.kind {
                    SymbolKind::Variable => CompletionItemKind::Variable,
                    SymbolKind::Function => CompletionItemKind::Function,
                    SymbolKind::Type => CompletionItemKind::Class,
                    SymbolKind::Constant => CompletionItemKind::Constant,
                    SymbolKind::Struct => CompletionItemKind::Struct,
                    _ => CompletionItemKind::Variable,
                },
                symbol.name.clone(),
            );

            if let Some(type_info) = &symbol.type_info {
                item = item.with_detail(type_info.clone());
            }

            if let Some(documentation) = &symbol.documentation {
                item = item.with_documentation(documentation.clone());
            }

            items.push(item);
        }

        Ok(items)
    }
}

/// Mock completion ranker
struct SymbolTestCompletionRanker;

impl ricecoder_completion::CompletionRanker for SymbolTestCompletionRanker {
    fn rank_completions(
        &self,
        mut items: Vec<CompletionItem>,
        _context: &CompletionContext,
    ) -> Vec<CompletionItem> {
        // Sort by label for deterministic ordering
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items
    }

    fn score_relevance(&self, _item: &CompletionItem, _context: &CompletionContext) -> f32 {
        0.5
    }

    fn score_frequency(&self, _item: &CompletionItem) -> f32 {
        0.3
    }
}

#[tokio::test]
async fn test_symbol_completion_variable_with_type() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Find the variable completion
    let var_completion = completions
        .iter()
        .find(|c| c.label == "my_var")
        .expect("Variable completion not found");

    assert_eq!(var_completion.kind, CompletionItemKind::Variable);
    assert_eq!(var_completion.detail, Some("i32".to_string()));
    assert_eq!(
        var_completion.documentation,
        Some("A variable holding an integer".to_string())
    );
}

#[tokio::test]
async fn test_symbol_completion_function_with_signature() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Find the function completion
    let fn_completion = completions
        .iter()
        .find(|c| c.label == "calculate")
        .expect("Function completion not found");

    assert_eq!(fn_completion.kind, CompletionItemKind::Function);
    assert_eq!(
        fn_completion.detail,
        Some("fn(i32, i32) -> i32".to_string())
    );
    assert_eq!(
        fn_completion.documentation,
        Some("Calculates the sum of two numbers".to_string())
    );
}

#[tokio::test]
async fn test_symbol_completion_type_with_documentation() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Find the type completion
    let type_completion = completions
        .iter()
        .find(|c| c.label == "Point")
        .expect("Type completion not found");

    assert_eq!(type_completion.kind, CompletionItemKind::Struct);
    assert_eq!(
        type_completion.detail,
        Some("struct Point { x: i32, y: i32 }".to_string())
    );
    assert_eq!(
        type_completion.documentation,
        Some("A 2D point structure".to_string())
    );
}

#[tokio::test]
async fn test_symbol_completion_constant() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Find the constant completion
    let const_completion = completions
        .iter()
        .find(|c| c.label == "MAX_SIZE")
        .expect("Constant completion not found");

    assert_eq!(const_completion.kind, CompletionItemKind::Constant);
    assert_eq!(const_completion.detail, Some("const usize".to_string()));
    assert_eq!(
        const_completion.documentation,
        Some("Maximum size constant".to_string())
    );
}

#[tokio::test]
async fn test_rust_provider_symbol_completions() {
    let provider = RustCompletionProvider;
    let mut context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());
    
    // Add test symbols
    context.available_symbols.push(Symbol {
        name: "test_var".to_string(),
        kind: SymbolKind::Variable,
        scope: Scope {
            kind: ScopeKind::Global,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
        },
        type_info: Some("String".to_string()),
        documentation: Some("A test variable".to_string()),
    });

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have both symbols and keywords
    assert!(!completions.is_empty());
    
    // Find the symbol completion
    let symbol_completion = completions
        .iter()
        .find(|c| c.label == "test_var")
        .expect("Symbol completion not found");

    assert_eq!(symbol_completion.kind, CompletionItemKind::Variable);
    assert_eq!(symbol_completion.detail, Some("String".to_string()));
    assert_eq!(
        symbol_completion.documentation,
        Some("A test variable".to_string())
    );

    // Should also have keywords
    let keyword_completion = completions
        .iter()
        .find(|c| c.label == "fn")
        .expect("Keyword completion not found");

    assert_eq!(keyword_completion.kind, CompletionItemKind::Keyword);
}

#[tokio::test]
async fn test_typescript_provider_symbol_completions() {
    let provider = TypeScriptCompletionProvider;
    let mut context = CompletionContext::new("typescript".to_string(), Position::new(0, 0), "".to_string());
    
    // Add test symbols
    context.available_symbols.push(Symbol {
        name: "myFunction".to_string(),
        kind: SymbolKind::Function,
        scope: Scope {
            kind: ScopeKind::Global,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
        },
        type_info: Some("(x: number) => string".to_string()),
        documentation: Some("A test function".to_string()),
    });

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have both symbols and keywords
    assert!(!completions.is_empty());
    
    // Find the symbol completion
    let symbol_completion = completions
        .iter()
        .find(|c| c.label == "myFunction")
        .expect("Symbol completion not found");

    assert_eq!(symbol_completion.kind, CompletionItemKind::Function);
    assert_eq!(
        symbol_completion.detail,
        Some("(x: number) => string".to_string())
    );
}

#[tokio::test]
async fn test_python_provider_symbol_completions() {
    let provider = PythonCompletionProvider;
    let mut context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());
    
    // Add test symbols
    context.available_symbols.push(Symbol {
        name: "my_class".to_string(),
        kind: SymbolKind::Class,
        scope: Scope {
            kind: ScopeKind::Global,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
        },
        type_info: Some("class MyClass".to_string()),
        documentation: Some("A test class".to_string()),
    });

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have both symbols and keywords
    assert!(!completions.is_empty());
    
    // Find the symbol completion
    let symbol_completion = completions
        .iter()
        .find(|c| c.label == "my_class")
        .expect("Symbol completion not found");

    assert_eq!(symbol_completion.kind, CompletionItemKind::Class);
    assert_eq!(symbol_completion.detail, Some("class MyClass".to_string()));
}

#[tokio::test]
async fn test_generic_provider_symbol_completions() {
    let provider = GenericTextProvider;
    let mut context = CompletionContext::new("unknown".to_string(), Position::new(0, 0), "".to_string());
    
    // Add test symbols
    context.available_symbols.push(Symbol {
        name: "generic_var".to_string(),
        kind: SymbolKind::Variable,
        scope: Scope {
            kind: ScopeKind::Global,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
        },
        type_info: Some("any".to_string()),
        documentation: Some("A generic variable".to_string()),
    });

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have symbol completions
    assert!(!completions.is_empty());
    
    // Find the symbol completion
    let symbol_completion = completions
        .iter()
        .find(|c| c.label == "generic_var")
        .expect("Symbol completion not found");

    assert_eq!(symbol_completion.kind, CompletionItemKind::Variable);
    assert_eq!(symbol_completion.detail, Some("any".to_string()));
}

#[tokio::test]
async fn test_symbol_completion_all_kinds() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Should have all symbol types
    assert!(completions.iter().any(|c| c.kind == CompletionItemKind::Variable));
    assert!(completions.iter().any(|c| c.kind == CompletionItemKind::Function));
    assert!(completions.iter().any(|c| c.kind == CompletionItemKind::Struct));
    assert!(completions.iter().any(|c| c.kind == CompletionItemKind::Constant));
}

#[tokio::test]
async fn test_symbol_completion_preserves_insert_text() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // All completions should have insert_text matching the label
    for completion in &completions {
        assert_eq!(completion.insert_text, completion.label);
    }
}

#[tokio::test]
async fn test_symbol_completion_multiple_symbols() {
    let engine = GenericCompletionEngine::new(
        Arc::new(SymbolTestContextAnalyzer),
        Arc::new(SymbolTestCompletionGenerator),
        Arc::new(SymbolTestCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Should have at least 4 symbols (variable, function, struct, constant)
    assert!(completions.len() >= 4);
}
