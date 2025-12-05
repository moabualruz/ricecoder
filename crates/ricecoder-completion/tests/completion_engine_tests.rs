use async_trait::async_trait;
/// Integration tests for the completion engine core
use ricecoder_completion::{
    CompletionContext, CompletionEngine, CompletionItem, CompletionItemKind,
    GenericCompletionEngine, Position, ProviderRegistry, Range, Scope, ScopeKind, Symbol,
    SymbolKind, Type,
};
use std::sync::Arc;

/// Mock context analyzer for testing
struct MockContextAnalyzer;

#[async_trait]
impl ricecoder_completion::ContextAnalyzer for MockContextAnalyzer {
    async fn analyze_context(
        &self,
        _code: &str,
        position: Position,
        language: &str,
    ) -> ricecoder_completion::CompletionResult<CompletionContext> {
        let mut context = CompletionContext::new(language.to_string(), position, "".to_string());

        // Add some test symbols
        context.available_symbols.push(Symbol {
            name: "test_var".to_string(),
            kind: SymbolKind::Variable,
            scope: Scope {
                kind: ScopeKind::Function,
                name: Some("main".to_string()),
                range: Range::new(Position::new(0, 0), Position::new(10, 0)),
            },
            type_info: Some("i32".to_string()),
            documentation: Some("A test variable".to_string()),
        });

        context.available_symbols.push(Symbol {
            name: "test_fn".to_string(),
            kind: SymbolKind::Function,
            scope: Scope {
                kind: ScopeKind::Global,
                name: None,
                range: Range::new(Position::new(0, 0), Position::new(100, 0)),
            },
            type_info: Some("fn() -> String".to_string()),
            documentation: Some("A test function".to_string()),
        });

        Ok(context)
    }

    fn get_available_symbols(&self, context: &CompletionContext, _code: &str) -> Vec<Symbol> {
        context.available_symbols.clone()
    }

    fn infer_expected_type(&self, _context: &CompletionContext) -> Option<Type> {
        Some(Type::new("String".to_string()))
    }
}

/// Mock completion generator for testing
struct MockCompletionGenerator;

#[async_trait]
impl ricecoder_completion::CompletionGenerator for MockCompletionGenerator {
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
                CompletionItemKind::Variable,
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

/// Mock completion ranker for testing
struct MockCompletionRanker;

impl ricecoder_completion::CompletionRanker for MockCompletionRanker {
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
async fn test_completion_item_creation() {
    let item = CompletionItem::new(
        "test".to_string(),
        CompletionItemKind::Variable,
        "test".to_string(),
    );

    assert_eq!(item.label, "test");
    assert_eq!(item.kind, CompletionItemKind::Variable);
    assert_eq!(item.insert_text, "test");
    assert_eq!(item.score, 0.0);
}

#[tokio::test]
async fn test_completion_item_with_details() {
    let item = CompletionItem::new(
        "test".to_string(),
        CompletionItemKind::Function,
        "test()".to_string(),
    )
    .with_detail("fn() -> String".to_string())
    .with_documentation("A test function".to_string())
    .with_score(0.9);

    assert_eq!(item.label, "test");
    assert_eq!(item.detail, Some("fn() -> String".to_string()));
    assert_eq!(item.documentation, Some("A test function".to_string()));
    assert_eq!(item.score, 0.9);
}

#[tokio::test]
async fn test_completion_context_creation() {
    let context =
        CompletionContext::new("rust".to_string(), Position::new(5, 10), "test".to_string());

    assert_eq!(context.language, "rust");
    assert_eq!(context.position.line, 5);
    assert_eq!(context.position.character, 10);
    assert_eq!(context.prefix, "test");
    assert!(context.available_symbols.is_empty());
}

#[tokio::test]
async fn test_completion_context_with_symbols() {
    let mut context =
        CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let symbol = Symbol {
        name: "my_var".to_string(),
        kind: SymbolKind::Variable,
        scope: Scope {
            kind: ScopeKind::Global,
            name: None,
            range: Range::new(Position::new(0, 0), Position::new(10, 0)),
        },
        type_info: Some("i32".to_string()),
        documentation: None,
    };

    context.available_symbols.push(symbol);

    assert_eq!(context.available_symbols.len(), 1);
    assert_eq!(context.available_symbols[0].name, "my_var");
}

#[tokio::test]
async fn test_generic_completion_engine_generation() {
    let engine = GenericCompletionEngine::new(
        Arc::new(MockContextAnalyzer),
        Arc::new(MockCompletionGenerator),
        Arc::new(MockCompletionRanker),
        ProviderRegistry::new(),
    );

    let code = "fn main() { let x = ";
    let position = Position::new(0, 19);
    let completions = engine
        .generate_completions(code, position, "rust")
        .await
        .expect("Failed to generate completions");

    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "test_var"));
    assert!(completions.iter().any(|c| c.label == "test_fn"));
}

#[tokio::test]
async fn test_completion_item_serialization() {
    let item = CompletionItem::new(
        "test".to_string(),
        CompletionItemKind::Variable,
        "test".to_string(),
    )
    .with_detail("i32".to_string());

    let json = serde_json::to_value(&item).expect("Failed to serialize");
    assert_eq!(json["label"], "test");
    assert_eq!(json["detail"], "i32");
}

#[tokio::test]
async fn test_completion_context_serialization() {
    let context =
        CompletionContext::new("rust".to_string(), Position::new(5, 10), "test".to_string());

    let json = serde_json::to_value(&context).expect("Failed to serialize");
    assert_eq!(json["language"], "rust");
    assert_eq!(json["prefix"], "test");
}

#[tokio::test]
async fn test_completion_item_invalid_input() {
    // Test with empty label
    let item = CompletionItem::new("".to_string(), CompletionItemKind::Variable, "".to_string());

    assert_eq!(item.label, "");
    assert_eq!(item.insert_text, "");
}

#[tokio::test]
async fn test_completion_context_invalid_position() {
    let context = CompletionContext::new(
        "rust".to_string(),
        Position::new(u32::MAX, u32::MAX),
        "test".to_string(),
    );

    assert_eq!(context.position.line, u32::MAX);
    assert_eq!(context.position.character, u32::MAX);
}

#[tokio::test]
async fn test_completion_item_error_handling() {
    // Test that completion items can be created even with unusual inputs
    let item = CompletionItem::new(
        "test\nwith\nnewlines".to_string(),
        CompletionItemKind::Keyword,
        "test".to_string(),
    );

    assert!(item.label.contains('\n'));
}

#[tokio::test]
async fn test_completion_engine_with_empty_code() {
    let engine = GenericCompletionEngine::new(
        Arc::new(MockContextAnalyzer),
        Arc::new(MockCompletionGenerator),
        Arc::new(MockCompletionRanker),
        ProviderRegistry::new(),
    );

    let completions = engine
        .generate_completions("", Position::new(0, 0), "rust")
        .await
        .expect("Failed to generate completions");

    // Should still generate completions from context
    assert!(!completions.is_empty());
}

#[tokio::test]
async fn test_completion_item_resolve() {
    let engine = GenericCompletionEngine::new(
        Arc::new(MockContextAnalyzer),
        Arc::new(MockCompletionGenerator),
        Arc::new(MockCompletionRanker),
        ProviderRegistry::new(),
    );

    let item = CompletionItem::new(
        "test".to_string(),
        CompletionItemKind::Variable,
        "test".to_string(),
    );

    let resolved = engine
        .resolve_completion(&item)
        .await
        .expect("Failed to resolve completion");

    assert_eq!(resolved.label, item.label);
}
