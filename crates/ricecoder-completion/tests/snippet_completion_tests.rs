use async_trait::async_trait;
/// Tests for snippet completion suggestions
/// Tests that snippet completions include function templates, loop templates, and conditional templates
use ricecoder_completion::{
    CompletionContext, CompletionEngine, CompletionItem, CompletionItemKind, CompletionProvider,
    GenericCompletionEngine, Position, ProviderRegistry, PythonCompletionProvider, Range,
    RustCompletionProvider, Scope, ScopeKind, Symbol, SymbolKind, TypeScriptCompletionProvider,
};
use std::sync::Arc;

/// Mock context analyzer for snippet tests
struct SnippetTestContextAnalyzer;

#[async_trait]
impl ricecoder_completion::ContextAnalyzer for SnippetTestContextAnalyzer {
    async fn analyze_context(
        &self,
        _code: &str,
        position: Position,
        language: &str,
    ) -> ricecoder_completion::CompletionResult<CompletionContext> {
        Ok(CompletionContext::new(
            language.to_string(),
            position,
            "".to_string(),
        ))
    }

    fn get_available_symbols(&self, context: &CompletionContext, _code: &str) -> Vec<Symbol> {
        context.available_symbols.clone()
    }

    fn infer_expected_type(
        &self,
        _context: &CompletionContext,
    ) -> Option<ricecoder_completion::Type> {
        None
    }
}

/// Mock completion generator
struct SnippetTestCompletionGenerator;

#[async_trait]
impl ricecoder_completion::CompletionGenerator for SnippetTestCompletionGenerator {
    async fn generate_completions(
        &self,
        _code: &str,
        _position: Position,
        _context: &CompletionContext,
    ) -> ricecoder_completion::CompletionResult<Vec<CompletionItem>> {
        Ok(Vec::new())
    }
}

/// Mock completion ranker
struct SnippetTestCompletionRanker;

impl ricecoder_completion::CompletionRanker for SnippetTestCompletionRanker {
    fn rank_completions(
        &self,
        items: Vec<CompletionItem>,
        _context: &CompletionContext,
    ) -> Vec<CompletionItem> {
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
async fn test_rust_snippet_completions() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Find snippet completions
    let fn_snippet = completions
        .iter()
        .find(|c| c.label == "fn_snippet")
        .expect("Function snippet not found");

    assert_eq!(fn_snippet.kind, CompletionItemKind::Snippet);
    assert!(fn_snippet.insert_text.contains("fn"));
    assert!(fn_snippet.insert_text.contains("${1:name}"));

    let match_snippet = completions
        .iter()
        .find(|c| c.label == "match_snippet")
        .expect("Match snippet not found");

    assert_eq!(match_snippet.kind, CompletionItemKind::Snippet);
    assert!(match_snippet.insert_text.contains("match"));

    let for_snippet = completions
        .iter()
        .find(|c| c.label == "for_snippet")
        .expect("For snippet not found");

    assert_eq!(for_snippet.kind, CompletionItemKind::Snippet);
    assert!(for_snippet.insert_text.contains("for"));
}

#[tokio::test]
async fn test_typescript_snippet_completions() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new(
        "typescript".to_string(),
        Position::new(0, 0),
        "".to_string(),
    );

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Find snippet completions
    let fn_snippet = completions
        .iter()
        .find(|c| c.label == "fn_snippet")
        .expect("Function snippet not found");

    assert_eq!(fn_snippet.kind, CompletionItemKind::Snippet);
    assert!(fn_snippet.insert_text.contains("function"));

    let arrow_fn_snippet = completions
        .iter()
        .find(|c| c.label == "arrow_fn_snippet")
        .expect("Arrow function snippet not found");

    assert_eq!(arrow_fn_snippet.kind, CompletionItemKind::Snippet);
    assert!(arrow_fn_snippet.insert_text.contains("=>"));

    let class_snippet = completions
        .iter()
        .find(|c| c.label == "class_snippet")
        .expect("Class snippet not found");

    assert_eq!(class_snippet.kind, CompletionItemKind::Snippet);
    assert!(class_snippet.insert_text.contains("class"));
}

#[tokio::test]
async fn test_python_snippet_completions() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Find snippet completions
    let def_snippet = completions
        .iter()
        .find(|c| c.label == "def_snippet")
        .expect("Function definition snippet not found");

    assert_eq!(def_snippet.kind, CompletionItemKind::Snippet);
    assert!(def_snippet.insert_text.contains("def"));

    let class_snippet = completions
        .iter()
        .find(|c| c.label == "class_snippet")
        .expect("Class snippet not found");

    assert_eq!(class_snippet.kind, CompletionItemKind::Snippet);
    assert!(class_snippet.insert_text.contains("class"));

    let list_comp_snippet = completions
        .iter()
        .find(|c| c.label == "list_comp_snippet")
        .expect("List comprehension snippet not found");

    assert_eq!(list_comp_snippet.kind, CompletionItemKind::Snippet);
    assert!(list_comp_snippet.insert_text.contains("["));
}

#[tokio::test]
async fn test_rust_snippet_has_placeholders() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // All snippets should have placeholders
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    assert!(!snippets.is_empty());

    for snippet in snippets {
        // Snippets should have placeholder syntax
        assert!(
            snippet.insert_text.contains("${"),
            "Snippet {} missing placeholders",
            snippet.label
        );
    }
}

#[tokio::test]
async fn test_typescript_snippet_has_placeholders() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new(
        "typescript".to_string(),
        Position::new(0, 0),
        "".to_string(),
    );

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // All snippets should have placeholders
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    assert!(!snippets.is_empty());

    for snippet in snippets {
        // Snippets should have placeholder syntax
        assert!(
            snippet.insert_text.contains("${"),
            "Snippet {} missing placeholders",
            snippet.label
        );
    }
}

#[tokio::test]
async fn test_python_snippet_has_placeholders() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // All snippets should have placeholders
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    assert!(!snippets.is_empty());

    for snippet in snippets {
        // Snippets should have placeholder syntax
        assert!(
            snippet.insert_text.contains("${"),
            "Snippet {} missing placeholders",
            snippet.label
        );
    }
}

#[tokio::test]
async fn test_rust_loop_snippets() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have for loop snippet
    let for_snippet = completions
        .iter()
        .find(|c| c.label == "for_snippet")
        .expect("For snippet not found");

    assert!(for_snippet.insert_text.contains("for"));
    assert!(for_snippet.insert_text.contains("in"));

    // Should have while loop snippet
    let while_snippet = completions
        .iter()
        .find(|c| c.label == "while_snippet")
        .expect("While snippet not found");

    assert!(while_snippet.insert_text.contains("while"));
}

#[tokio::test]
async fn test_rust_conditional_snippets() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have if-else snippet
    let if_snippet = completions
        .iter()
        .find(|c| c.label == "if_snippet")
        .expect("If snippet not found");

    assert!(if_snippet.insert_text.contains("if"));
    assert!(if_snippet.insert_text.contains("else"));

    // Should have match snippet
    let match_snippet = completions
        .iter()
        .find(|c| c.label == "match_snippet")
        .expect("Match snippet not found");

    assert!(match_snippet.insert_text.contains("match"));
}

#[tokio::test]
async fn test_typescript_loop_snippets() {
    let provider = TypeScriptCompletionProvider;
    let context = CompletionContext::new(
        "typescript".to_string(),
        Position::new(0, 0),
        "".to_string(),
    );

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have for loop snippet
    let for_snippet = completions
        .iter()
        .find(|c| c.label == "for_snippet")
        .expect("For snippet not found");

    assert!(for_snippet.insert_text.contains("for"));

    // Should have for-of loop snippet
    let for_of_snippet = completions
        .iter()
        .find(|c| c.label == "for_of_snippet")
        .expect("For-of snippet not found");

    assert!(for_of_snippet.insert_text.contains("for"));
    assert!(for_of_snippet.insert_text.contains("of"));
}

#[tokio::test]
async fn test_python_loop_snippets() {
    let provider = PythonCompletionProvider;
    let context = CompletionContext::new("python".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // Should have for loop snippet
    let for_snippet = completions
        .iter()
        .find(|c| c.label == "for_snippet")
        .expect("For snippet not found");

    assert!(for_snippet.insert_text.contains("for"));
    assert!(for_snippet.insert_text.contains("in"));

    // Should have while loop snippet
    let while_snippet = completions
        .iter()
        .find(|c| c.label == "while_snippet")
        .expect("While snippet not found");

    assert!(while_snippet.insert_text.contains("while"));
}

#[tokio::test]
async fn test_snippet_completion_score() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // All snippets should have a score
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    for snippet in snippets {
        assert!(
            snippet.score > 0.0,
            "Snippet {} has no score",
            snippet.label
        );
    }
}

#[tokio::test]
async fn test_snippet_completion_has_description() {
    let provider = RustCompletionProvider;
    let context = CompletionContext::new("rust".to_string(), Position::new(0, 0), "".to_string());

    let completions = provider
        .generate_completions("", Position::new(0, 0), &context)
        .await
        .expect("Failed to generate completions");

    // All snippets should have a description
    let snippets: Vec<_> = completions
        .iter()
        .filter(|c| c.kind == CompletionItemKind::Snippet)
        .collect();

    for snippet in snippets {
        assert!(
            snippet.detail.is_some(),
            "Snippet {} has no description",
            snippet.label
        );
    }
}
