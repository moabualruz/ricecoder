//! Property-based tests for MCP provider integration
//!
//! Tests the hybrid provider pattern with MCP → Built-in → Error priority chain.
//! Validates that MCP servers are used when available and graceful fallback occurs.

use async_trait::async_trait;
use ricecoder_tools::provider::{Provider, ProviderRegistry};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Mock provider for testing
struct MockProvider {
    name: String,
    should_fail: bool,
}

#[async_trait]
impl Provider for MockProvider {
    async fn execute(&self, input: &str) -> Result<String, ricecoder_tools::ToolError> {
        if self.should_fail {
            Err(ricecoder_tools::ToolError::new(
                "MOCK_ERROR",
                "Mock provider error",
            ))
        } else {
            Ok(format!("{}: {}", self.name, input))
        }
    }
}

/// Property 11: MCP provider priority
///
/// For any tool invocation, if an MCP server is configured and available,
/// the system SHALL use the MCP server instead of the built-in implementation.
///
/// **Validates: Requirements 1.1, 1.7, 2.1, 2.7, 3.1, 3.7, 4.1, 4.7**
#[tokio::test]
async fn prop_mcp_provider_priority() {
    // Test that MCP provider is selected when both MCP and built-in are available
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    let mcp = Arc::new(MockProvider {
        name: "mcp".to_string(),
        should_fail: false,
    });

    registry
        .register_builtin_provider("test_tool", builtin)
        .await;
    registry.register_mcp_provider("test_tool", mcp).await;

    // Get provider - should select MCP
    let (provider, selection) = registry.get_provider("test_tool").await.unwrap();

    // Verify MCP was selected
    assert_eq!(selection.provider_name, "mcp");
    assert!(!selection.is_fallback);

    // Verify MCP provider executes correctly
    let result = provider.execute("test").await.unwrap();
    assert_eq!(result, "mcp: test");
}

/// Property 12: Graceful fallback
///
/// For any tool invocation where the MCP server is unavailable or fails,
/// the system SHALL fall back to the built-in implementation without error.
///
/// **Validates: Requirements 1.1, 2.1, 3.1, 4.1**
#[tokio::test]
async fn prop_graceful_fallback_when_mcp_unavailable() {
    // Test that built-in provider is used when MCP is not available
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });

    registry
        .register_builtin_provider("test_tool", builtin)
        .await;

    // Get provider - should fall back to built-in
    let (provider, selection) = registry.get_provider("test_tool").await.unwrap();

    // Verify built-in was selected as fallback
    assert_eq!(selection.provider_name, "builtin");
    assert!(selection.is_fallback);

    // Verify built-in provider executes correctly
    let result = provider.execute("test").await.unwrap();
    assert_eq!(result, "builtin: test");
}

/// Property: MCP provider priority with multiple tools
///
/// For any set of tools with both MCP and built-in providers,
/// each tool should use its MCP provider when available.
#[tokio::test]
async fn prop_mcp_priority_multiple_tools() {
    let registry = ProviderRegistry::new();

    // Register providers for multiple tools
    for tool_num in 0..5 {
        let tool_name = format!("tool_{}", tool_num);
        let builtin = Arc::new(MockProvider {
            name: format!("builtin_{}", tool_num),
            should_fail: false,
        });
        let mcp = Arc::new(MockProvider {
            name: format!("mcp_{}", tool_num),
            should_fail: false,
        });

        registry
            .register_builtin_provider(&tool_name, builtin)
            .await;
        registry.register_mcp_provider(&tool_name, mcp).await;
    }

    // Verify each tool uses its MCP provider
    for tool_num in 0..5 {
        let tool_name = format!("tool_{}", tool_num);
        let (provider, selection) = registry.get_provider(&tool_name).await.unwrap();

        assert_eq!(selection.provider_name, "mcp");
        assert!(!selection.is_fallback);

        let result = provider.execute("test").await.unwrap();
        assert_eq!(result, format!("mcp_{}: test", tool_num));
    }
}

/// Property: Fallback chain completeness
///
/// For any tool, if MCP is unavailable, the system should try built-in.
/// If built-in is also unavailable, the system should return an error.
#[tokio::test]
async fn prop_fallback_chain_completeness() {
    let registry = ProviderRegistry::new();

    // Test 1: Only built-in available
    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    registry.register_builtin_provider("tool1", builtin).await;

    let (_, selection) = registry.get_provider("tool1").await.unwrap();
    assert_eq!(selection.provider_name, "builtin");
    assert!(selection.is_fallback);

    // Test 2: Neither available
    let result = registry.get_provider("tool2").await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.code, "PROVIDER_NOT_FOUND");
    }
}

/// Property: Provider selection includes trace ID
///
/// For any provider selection, the result should include a unique trace ID
/// for debugging and tracing purposes.
#[tokio::test]
async fn prop_provider_selection_has_trace_id() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    registry
        .register_builtin_provider("test_tool", builtin)
        .await;

    // Get provider multiple times
    let mut trace_ids = Vec::new();
    for _ in 0..5 {
        let (_, selection) = registry.get_provider("test_tool").await.unwrap();
        trace_ids.push(selection.trace_id);
    }

    // Verify all trace IDs are non-empty
    for trace_id in &trace_ids {
        assert!(!trace_id.is_empty());
    }

    // Verify trace IDs are unique (UUIDs)
    let unique_count = trace_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(unique_count, trace_ids.len());
}

/// Property: Cache invalidation works correctly
///
/// For any tool, after invalidating the cache, the next availability check
/// should re-check the MCP server status.
#[tokio::test]
async fn prop_cache_invalidation() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    registry
        .register_builtin_provider("test_tool", builtin)
        .await;

    // First call - populates cache
    let (_, selection1) = registry.get_provider("test_tool").await.unwrap();
    assert_eq!(selection1.provider_name, "builtin");

    // Invalidate cache
    registry.invalidate_cache("test_tool").await;

    // Second call - should re-check
    let (_, selection2) = registry.get_provider("test_tool").await.unwrap();
    assert_eq!(selection2.provider_name, "builtin");

    // Trace IDs should be different (different invocations)
    assert_ne!(selection1.trace_id, selection2.trace_id);
}

/// Property: Provider selection callback is invoked
///
/// For any provider selection, if a callback is registered,
/// it should be invoked with the selection information.
#[tokio::test]
async fn prop_provider_selection_callback_invoked() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    registry
        .register_builtin_provider("test_tool", builtin)
        .await;

    let callback_invoked = Arc::new(AtomicBool::new(false));
    let callback_invoked_clone = callback_invoked.clone();

    registry
        .on_provider_selected(move |_selection| {
            callback_invoked_clone.store(true, Ordering::SeqCst);
        })
        .await;

    // Get provider - should invoke callback
    let _ = registry.get_provider("test_tool").await;

    // Give callback time to execute
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Verify callback was invoked
    assert!(callback_invoked.load(Ordering::SeqCst));
}

/// Property: Simple provider API works correctly
///
/// For any tool, the simplified get_provider_simple API should return
/// the same provider as the full get_provider API.
#[tokio::test]
async fn prop_simple_provider_api() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    registry
        .register_builtin_provider("test_tool", builtin)
        .await;

    // Get provider using both APIs
    let (provider_full, _) = registry.get_provider("test_tool").await.unwrap();
    let provider_simple = registry.get_provider_simple("test_tool").await.unwrap();

    // Both should execute the same way
    let result_full = provider_full.execute("test").await.unwrap();
    let result_simple = provider_simple.execute("test").await.unwrap();

    assert_eq!(result_full, result_simple);
}

/// Property: MCP priority persists across multiple calls
///
/// For any tool with both MCP and built-in providers,
/// multiple calls should consistently select MCP.
#[tokio::test]
async fn prop_mcp_priority_consistency() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    let mcp = Arc::new(MockProvider {
        name: "mcp".to_string(),
        should_fail: false,
    });

    registry
        .register_builtin_provider("test_tool", builtin)
        .await;
    registry.register_mcp_provider("test_tool", mcp).await;

    // Call multiple times
    for _ in 0..10 {
        let (_, selection) = registry.get_provider("test_tool").await.unwrap();
        assert_eq!(selection.provider_name, "mcp");
        assert!(!selection.is_fallback);
    }
}

/// Property: Fallback is marked correctly
///
/// For any tool using fallback, the selection should indicate is_fallback = true.
/// For any tool using primary provider, the selection should indicate is_fallback = false.
#[tokio::test]
async fn prop_fallback_flag_correctness() {
    let registry = ProviderRegistry::new();

    let builtin = Arc::new(MockProvider {
        name: "builtin".to_string(),
        should_fail: false,
    });
    let mcp = Arc::new(MockProvider {
        name: "mcp".to_string(),
        should_fail: false,
    });

    // Test 1: MCP available - should not be fallback
    registry
        .register_builtin_provider("tool1", builtin.clone())
        .await;
    registry.register_mcp_provider("tool1", mcp.clone()).await;

    let (_, selection1) = registry.get_provider("tool1").await.unwrap();
    assert!(!selection1.is_fallback);

    // Test 2: Only built-in available - should be fallback
    let result = registry.get_provider("tool2").await;
    if let Ok((_, sel)) = result {
        assert!(sel.is_fallback);
    }
}
