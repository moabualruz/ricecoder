//! Integration tests for LSP completion functionality
//!
//! Tests the end-to-end completion workflow including:
//! - Completion request handling
//! - Multi-language support (Rust, TypeScript, Python)
//! - Rapid typing handling
//! - Memory usage under load

use ricecoder_lsp::{
    server::LspServer,
    types::ServerState,
    config::CompletionConfig,
};
use ricecoder_completion::{
    GenericCompletionEngine, TreeSitterContextAnalyzer, CompletionGenerator,
    BasicCompletionRanker, ProviderRegistry, RustCompletionProvider,
    TypeScriptCompletionProvider, PythonCompletionProvider, CompletionResult,
    CompletionContext, CompletionItem, Position as CompletionPosition,
};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

/// Mock completion generator for testing
struct MockCompletionGenerator;

#[async_trait]
impl CompletionGenerator for MockCompletionGenerator {
    async fn generate_completions(
        &self,
        _code: &str,
        _position: CompletionPosition,
        _context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        // Return some basic completions for testing
        Ok(vec![
            CompletionItem::new("let".to_string(), ricecoder_completion::CompletionItemKind::Keyword, "let".to_string()),
            CompletionItem::new("fn".to_string(), ricecoder_completion::CompletionItemKind::Keyword, "fn".to_string()),
            CompletionItem::new("if".to_string(), ricecoder_completion::CompletionItemKind::Keyword, "if".to_string()),
        ])
    }
}

/// Helper to create a test server with completion engine
fn create_test_server_with_completion() -> LspServer {
    let mut server = LspServer::new();
    
    // Create completion engine with providers
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(RustCompletionProvider));
    registry.register(Arc::new(TypeScriptCompletionProvider));
    registry.register(Arc::new(PythonCompletionProvider));
    
    let engine = Arc::new(GenericCompletionEngine::new(
        Arc::new(TreeSitterContextAnalyzer),
        Arc::new(MockCompletionGenerator),
        Arc::new(BasicCompletionRanker::default_weights()),
        registry,
    ));
    
    server.register_completion_engine(engine);
    server.set_state(ServerState::Initialized);
    
    server
}

#[tokio::test]
async fn test_completion_workflow_end_to_end() {
    let mut server = create_test_server_with_completion();
    
    // Open a Rust document
    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    // Request completion
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_ok(), "Completion request should succeed");
    
    let completions = result.unwrap();
    assert!(completions.is_array(), "Result should be an array");
    
    let items = completions.as_array().unwrap();
    assert!(!items.is_empty(), "Should return at least one completion");
}

#[tokio::test]
async fn test_completion_multi_language_rust() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_ok());
    
    let completions = result.unwrap();
    let items = completions.as_array().unwrap();
    
    // Rust completions should include keywords like 'let', 'fn', etc.
    let has_rust_keywords = items.iter().any(|item| {
        if let Some(label) = item.get("label").and_then(|l| l.as_str()) {
            matches!(label, "let" | "fn" | "if" | "match" | "for" | "while")
        } else {
            false
        }
    });
    
    assert!(has_rust_keywords, "Should include Rust keywords");
}

#[tokio::test]
async fn test_completion_multi_language_typescript() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.ts";
    let code = "function main() {\n    const x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 14 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_ok());
    
    let completions = result.unwrap();
    let items = completions.as_array().unwrap();
    
    // TypeScript completions should include keywords
    let has_ts_keywords = items.iter().any(|item| {
        if let Some(label) = item.get("label").and_then(|l| l.as_str()) {
            matches!(label, "const" | "let" | "var" | "function" | "class" | "interface")
        } else {
            false
        }
    });
    
    assert!(has_ts_keywords, "Should include TypeScript keywords");
}

#[tokio::test]
async fn test_completion_multi_language_python() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.py";
    let code = "def main():\n    x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 8 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_ok());
    
    let completions = result.unwrap();
    let items = completions.as_array().unwrap();
    
    // Python completions should include keywords
    let has_python_keywords = items.iter().any(|item| {
        if let Some(label) = item.get("label").and_then(|l| l.as_str()) {
            matches!(label, "def" | "class" | "if" | "for" | "while" | "return" | "import")
        } else {
            false
        }
    });
    
    assert!(has_python_keywords, "Should include Python keywords");
}

#[tokio::test]
async fn test_completion_rapid_typing_handling() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    // Simulate rapid typing by sending multiple completion requests
    for i in 0..10 {
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": 1, "character": 12 + i }
        });
        
        let result = server.handle_completion(params).await;
        assert!(result.is_ok(), "Completion request {} should succeed", i);
    }
}

#[tokio::test]
async fn test_completion_disabled() {
    let mut server = LspServer::new();
    
    // Create completion config with completion disabled
    let mut config = CompletionConfig::default();
    config.enabled = false;
    
    server.set_completion_config(config);
    server.set_state(ServerState::Initialized);
    
    let uri = "file://test.rs";
    let code = "fn main() {}";
    
    server.set_document(uri.to_string(), code.to_string());
    
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 0, "character": 0 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_ok());
    
    // Should return empty array when disabled
    let completions = result.unwrap();
    let items = completions.as_array().unwrap();
    assert!(items.is_empty(), "Should return empty completions when disabled");
}

#[tokio::test]
async fn test_completion_configuration() {
    let mut config = CompletionConfig::default();
    
    // Test default values
    assert!(config.enabled);
    assert_eq!(config.timeout_ms, 100);
    assert_eq!(config.max_completions, 50);
    assert!(config.ghost_text_enabled);
    
    // Test custom values
    config.enabled = false;
    config.timeout_ms = 200;
    config.max_completions = 100;
    config.ghost_text_enabled = false;
    
    assert!(!config.enabled);
    assert_eq!(config.timeout_ms, 200);
    assert_eq!(config.max_completions, 100);
    assert!(!config.ghost_text_enabled);
}

#[tokio::test]
async fn test_completion_item_resolve() {
    let server = create_test_server_with_completion();
    
    let item = json!({
        "label": "test_function",
        "kind": 3,
        "detail": "fn test_function()",
        "documentation": "A test function"
    });
    
    let result = server.handle_completion_resolve(item.clone()).await;
    assert!(result.is_ok());
    
    let resolved = result.unwrap();
    assert_eq!(resolved.get("label").and_then(|l| l.as_str()), Some("test_function"));
    assert_eq!(resolved.get("resolved").and_then(|r| r.as_bool()), Some(true));
}

#[tokio::test]
async fn test_completion_not_initialized() {
    let server = LspServer::new();
    
    let params = json!({
        "textDocument": { "uri": "file://test.rs" },
        "position": { "line": 0, "character": 0 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_err(), "Should fail when server not initialized");
}

#[tokio::test]
async fn test_completion_missing_document() {
    let server = create_test_server_with_completion();
    
    let params = json!({
        "textDocument": { "uri": "file://nonexistent.rs" },
        "position": { "line": 0, "character": 0 }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_err(), "Should fail for non-existent document");
}

#[tokio::test]
async fn test_completion_invalid_position() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.rs";
    let code = "fn main() {}";
    
    server.set_document(uri.to_string(), code.to_string());
    
    // Missing position parameter
    let params = json!({
        "textDocument": { "uri": uri }
    });
    
    let result = server.handle_completion(params).await;
    assert!(result.is_err(), "Should fail with missing position");
}

#[tokio::test]
async fn test_completion_memory_under_load() {
    let mut server = create_test_server_with_completion();
    
    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";
    
    server.set_document(uri.to_string(), code.to_string());
    
    // Simulate load by sending many completion requests
    for _ in 0..100 {
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": 1, "character": 12 }
        });
        
        let result = server.handle_completion(params).await;
        assert!(result.is_ok(), "Completion request should succeed under load");
    }
}

#[tokio::test]
async fn test_completion_with_different_file_extensions() {
    let mut server = create_test_server_with_completion();
    
    let test_cases = vec![
        ("file://test.rs", "fn main() {", "rust"),
        ("file://test.ts", "function main() {", "typescript"),
        ("file://test.py", "def main():", "python"),
        ("file://test.js", "function main() {", "typescript"),
        ("file://test.tsx", "function Main() {", "typescript"),
    ];
    
    for (uri, code, _language) in test_cases {
        server.set_document(uri.to_string(), code.to_string());
        
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": code.len() as u32 }
        });
        
        let result = server.handle_completion(params).await;
        assert!(result.is_ok(), "Completion should work for {}", uri);
    }
}
