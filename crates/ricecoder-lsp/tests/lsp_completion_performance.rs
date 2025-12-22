//! Performance tests for LSP completion functionality
//!
//! Tests performance targets:
//! - Completion generation time: < 100ms
//! - Ghost text update time: < 50ms
//! - Prefix filtering time: < 50ms

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
use ricecoder_completion::{
    BasicCompletionRanker, CompletionContext, CompletionGenerator, CompletionItem,
    CompletionItemKind, CompletionResult, GenericCompletionEngine, Position as CompletionPosition,
    ProviderRegistry, PythonCompletionProvider, RustCompletionProvider, TreeSitterContextAnalyzer,
    TypeScriptCompletionProvider,
};
use ricecoder_lsp::{config::CompletionConfig, server::LspServer, types::ServerState};
use serde_json::json;

/// Mock completion generator for performance testing
struct PerformanceTestGenerator;

#[async_trait]
impl CompletionGenerator for PerformanceTestGenerator {
    async fn generate_completions(
        &self,
        _code: &str,
        _position: CompletionPosition,
        _context: &CompletionContext,
    ) -> CompletionResult<Vec<CompletionItem>> {
        // Generate a reasonable number of completions for testing
        let mut completions = Vec::new();

        for i in 0..50 {
            completions.push(
                CompletionItem::new(
                    format!("completion_{}", i),
                    CompletionItemKind::Variable,
                    format!("completion_{}", i),
                )
                .with_detail(format!("Type {}", i))
                .with_score(0.5 - (i as f32 * 0.01)),
            );
        }

        Ok(completions)
    }
}

/// Helper to create a test server with completion engine
fn create_performance_test_server() -> LspServer {
    let mut server = LspServer::new();

    // Create completion engine with providers
    let mut registry = ProviderRegistry::new();
    registry.register(Arc::new(RustCompletionProvider));
    registry.register(Arc::new(TypeScriptCompletionProvider));
    registry.register(Arc::new(PythonCompletionProvider));

    let engine = Arc::new(GenericCompletionEngine::new(
        Arc::new(TreeSitterContextAnalyzer),
        Arc::new(PerformanceTestGenerator),
        Arc::new(BasicCompletionRanker::default_weights()),
        registry,
    ));

    server.register_completion_engine(engine);
    server.set_state(ServerState::Initialized);

    server
}

#[tokio::test]
async fn test_completion_generation_time_target() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";

    server.set_document(uri.to_string(), code.to_string());

    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });

    // Measure completion generation time
    let start = Instant::now();
    let result = server.handle_completion(params).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Completion should succeed");

    // Target: < 100ms
    let target_ms = 100;
    let elapsed_ms = elapsed.as_millis() as u64;

    println!(
        "Completion generation time: {}ms (target: <{}ms)",
        elapsed_ms, target_ms
    );
    assert!(
        elapsed_ms < target_ms,
        "Completion generation should be < {}ms, but took {}ms",
        target_ms,
        elapsed_ms
    );
}

#[tokio::test]
async fn test_completion_generation_time_average() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";

    server.set_document(uri.to_string(), code.to_string());

    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });

    // Measure average completion generation time over multiple runs
    let mut total_ms = 0u64;
    let iterations = 10;

    for _ in 0..iterations {
        let start = Instant::now();
        let result = server.handle_completion(params.clone()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        total_ms += elapsed.as_millis() as u64;
    }

    let average_ms = total_ms / iterations as u64;
    let target_ms = 100;

    println!(
        "Average completion generation time: {}ms (target: <{}ms)",
        average_ms, target_ms
    );
    assert!(
        average_ms < target_ms,
        "Average completion generation should be < {}ms, but was {}ms",
        target_ms,
        average_ms
    );
}

#[tokio::test]
async fn test_prefix_filtering_performance() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = comp";

    server.set_document(uri.to_string(), code.to_string());

    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 16 }
    });

    // Measure prefix filtering time
    let start = Instant::now();
    let result = server.handle_completion(params).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());

    // Target: < 50ms
    let target_ms = 50;
    let elapsed_ms = elapsed.as_millis() as u64;

    println!(
        "Prefix filtering time: {}ms (target: <{}ms)",
        elapsed_ms, target_ms
    );
    assert!(
        elapsed_ms < target_ms,
        "Prefix filtering should be < {}ms, but took {}ms",
        target_ms,
        elapsed_ms
    );
}

#[tokio::test]
async fn test_completion_with_large_code() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";

    // Generate a large code file (10KB)
    let mut code = String::new();
    for i in 0..1000 {
        code.push_str(&format!("let var_{} = {};\n", i, i));
    }
    code.push_str("let x = ");

    server.set_document(uri.to_string(), code.clone());

    let line_count = code.lines().count() as u32;
    let last_line_len = code.lines().last().unwrap_or("").len() as u32;

    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line_count - 1, "character": last_line_len }
    });

    // Measure completion time with large code
    let start = Instant::now();
    let result = server.handle_completion(params).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok());

    // Target: < 100ms even with large code
    let target_ms = 100;
    let elapsed_ms = elapsed.as_millis() as u64;

    println!(
        "Completion with large code ({}KB): {}ms (target: <{}ms)",
        code.len() / 1024,
        elapsed_ms,
        target_ms
    );
    assert!(
        elapsed_ms < target_ms,
        "Completion with large code should be < {}ms, but took {}ms",
        target_ms,
        elapsed_ms
    );
}

#[tokio::test]
async fn test_rapid_completion_requests_performance() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";

    server.set_document(uri.to_string(), code.to_string());

    // Measure time for rapid completion requests
    let start = Instant::now();

    for i in 0..20 {
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": 1, "character": 12 + i }
        });

        let result = server.handle_completion(params).await;
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    let total_ms = elapsed.as_millis() as u64;
    let average_ms = total_ms / 20;

    // Target: < 100ms per request on average
    let target_ms = 100;

    println!(
        "Rapid requests average: {}ms per request (target: <{}ms)",
        average_ms, target_ms
    );
    assert!(
        average_ms < target_ms,
        "Rapid requests should average < {}ms per request, but averaged {}ms",
        target_ms,
        average_ms
    );
}

#[tokio::test]
async fn test_completion_configuration_performance_impact() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";

    server.set_document(uri.to_string(), code.to_string());

    // Test with default config
    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });

    let start = Instant::now();
    let result = server.handle_completion(params.clone()).await;
    let elapsed_default = start.elapsed();

    assert!(result.is_ok());

    // Test with custom config (reduced max completions)
    let mut custom_config = CompletionConfig::default();
    custom_config.max_completions = 10;
    server.set_completion_config(custom_config);

    let start = Instant::now();
    let result = server.handle_completion(params).await;
    let elapsed_custom = start.elapsed();

    assert!(result.is_ok());

    println!("Default config: {}ms", elapsed_default.as_millis());
    println!("Custom config (max 10): {}ms", elapsed_custom.as_millis());

    // Custom config should not be significantly slower
    assert!(
        elapsed_custom.as_millis() <= elapsed_default.as_millis() * 2,
        "Custom config should not be significantly slower"
    );
}

#[tokio::test]
async fn test_completion_memory_efficiency() {
    let mut server = create_performance_test_server();

    let uri = "file://test.rs";
    let code = "fn main() {\n    let x = ";

    server.set_document(uri.to_string(), code.to_string());

    let params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": 1, "character": 12 }
    });

    // Perform many completions to test memory efficiency
    for _ in 0..100 {
        let result = server.handle_completion(params.clone()).await;
        assert!(result.is_ok());
    }

    // If we got here without running out of memory, the test passes
    println!("Successfully completed 100 completion requests");
}

#[tokio::test]
async fn test_completion_response_size() {
    let mut server = create_performance_test_server();

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
    let response_json = serde_json::to_string(&completions).unwrap();
    let response_size = response_json.len();

    println!("Completion response size: {} bytes", response_size);

    // Response should be reasonably sized (< 100KB for 50 completions)
    assert!(
        response_size < 100_000,
        "Response size should be < 100KB, but was {} bytes",
        response_size
    );
}
