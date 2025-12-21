//! Unit tests for tool invokers

use ricecoder_agents::tool_invokers::*;
use ricecoder_agents::ToolInvoker;
use serde_json::json;

#[tokio::test]
async fn test_webfetch_invoker() {
    let invoker = WebfetchToolInvoker;
    let input = json!({
        "url": "https://example.com"
    });

    let result = invoker.invoke(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output["success"], true);
    assert_eq!(output["url"], "https://example.com");
}

#[tokio::test]
async fn test_webfetch_missing_url() {
    let invoker = WebfetchToolInvoker;
    let input = json!({});

    let result = invoker.invoke(input).await;
    assert!(result.is_err());
}

#[test]
fn test_webfetch_metadata() {
    let invoker = WebfetchToolInvoker;
    let metadata = invoker.metadata();

    assert_eq!(metadata.id, "webfetch");
    assert_eq!(metadata.name, "Webfetch");
    assert!(metadata.available);
}

#[tokio::test]
async fn test_patch_invoker() {
    let invoker = PatchToolInvoker;
    let input = json!({
        "file_path": "src/main.rs",
        "patch": "@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"Hello, world!\");\n     // existing code\n }"
    });

    let result = invoker.invoke(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output["success"], true);
}

#[tokio::test]
async fn test_patch_invalid_format() {
    let invoker = PatchToolInvoker;
    let input = json!({
        "file_path": "src/main.rs",
        "patch": "invalid patch format"
    });

    let result = invoker.invoke(input).await;
    assert!(result.is_err());
}

#[test]
fn test_patch_metadata() {
    let invoker = PatchToolInvoker;
    let metadata = invoker.metadata();

    assert_eq!(metadata.id, "patch");
    assert_eq!(metadata.name, "Patch");
    assert!(metadata.available);
}

#[tokio::test]
async fn test_todowrite_invoker() {
    let invoker = TodowriteToolInvoker;
    let input = json!({
        "content": "Test todo item",
        "status": "pending",
        "priority": "high"
    });

    let result = invoker.invoke(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output["success"], true);
}

#[tokio::test]
async fn test_todowrite_missing_content() {
    let invoker = TodowriteToolInvoker;
    let input = json!({});

    let result = invoker.invoke(input).await;
    assert!(result.is_err());
}

#[test]
fn test_todowrite_metadata() {
    let invoker = TodowriteToolInvoker;
    let metadata = invoker.metadata();

    assert_eq!(metadata.id, "todowrite");
    assert_eq!(metadata.name, "Todowrite");
    assert!(metadata.available);
}

#[tokio::test]
async fn test_todoread_invoker() {
    let invoker = TodoreadToolInvoker;
    let input = json!({});

    let result = invoker.invoke(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output["success"], true);
}

#[test]
fn test_todoread_metadata() {
    let invoker = TodoreadToolInvoker;
    let metadata = invoker.metadata();

    assert_eq!(metadata.id, "todoread");
    assert_eq!(metadata.name, "Todoread");
    assert!(metadata.available);
}

#[tokio::test]
async fn test_websearch_invoker() {
    let invoker = WebsearchToolInvoker;
    let input = json!({
        "query": "rust programming"
    });

    let result = invoker.invoke(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert_eq!(output["success"], true);
}

#[tokio::test]
async fn test_websearch_missing_query() {
    let invoker = WebsearchToolInvoker;
    let input = json!({});

    let result = invoker.invoke(input).await;
    assert!(result.is_err());
}

#[test]
fn test_websearch_metadata() {
    let invoker = WebsearchToolInvoker;
    let metadata = invoker.metadata();

    assert_eq!(metadata.id, "websearch");
    assert_eq!(metadata.name, "Websearch");
    assert!(metadata.available);
}
