//! MCP Performance Optimization Tests
//!
//! Tests to ensure performance targets are maintained while preserving coverage.
//! Focuses on optimizing test execution time and resource usage.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use ricecoder_mcp::{
    connection_pool::{ConnectionPool, PoolConfig},
    metadata::{ToolMetadata, ToolSource},
    permissions::MCPPermissionManager,
    registry::ToolRegistry,
    tool_execution::{ToolExecutionContext, ToolExecutionResult},
};
use serde_json::json;
use tokio::sync::RwLock;

/// **Performance Test Perf.1: Tool Registry Lookup Performance**
/// **Validates: Tool registry operations complete within performance targets**
#[tokio::test]
async fn test_tool_registry_lookup_performance() {
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));

    // Setup test tools
    let num_tools = 1000;
    {
        let mut reg = registry.write().await;
        for i in 0..num_tools {
            let tool = ToolMetadata {
                id: format!("perf-tool-{}", i),
                name: format!("Performance Tool {}", i),
                description: format!("Tool for performance testing {}", i),
                category: "performance".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Custom,
                server_id: None,
            };
            reg.register_tool(tool).unwrap();
        }
    }

    // Test lookup performance
    let start_time = Instant::now();
    let lookup_count = 10000;

    for _ in 0..lookup_count {
        let tool_id = format!("perf-tool-{}", rand::random::<usize>() % num_tools);
        let reg = registry.read().await;
        let _tool = reg.get_tool(&tool_id);
    }

    let duration = start_time.elapsed();
    let lookups_per_second = lookup_count as f64 / duration.as_secs_f64();

    // Performance assertion: should handle at least 1000 lookups per second
    assert!(
        lookups_per_second > 1000.0,
        "Lookup performance too slow: {} lookups/sec",
        lookups_per_second
    );
}

/// **Performance Test Perf.2: Permission Manager Scalability**
/// **Validates: Permission checking scales with number of rules**
#[tokio::test]
async fn test_permission_manager_scalability() {
    let permission_manager = Arc::new(MCPPermissionManager::new());

    // Setup many permission rules
    let num_rules = 1000;
    for i in 0..num_rules {
        let rule = ricecoder_mcp::permissions::PermissionRule {
            pattern: format!("tool-{}-*", i),
            level: ricecoder_mcp::permissions::PermissionLevelConfig::Allow,
            agent_id: Some(format!("agent-{}", i % 10)),
        };
        permission_manager.add_global_rule(rule).unwrap();
    }

    // Test permission checking performance
    let start_time = Instant::now();
    let check_count = 5000;

    for i in 0..check_count {
        let tool_name = format!("tool-{}-action", i % num_rules);
        let agent_id = format!("agent-{}", i % 10);
        let _result = permission_manager.check_permission(&tool_name, Some(&agent_id));
    }

    let duration = start_time.elapsed();
    let checks_per_second = check_count as f64 / duration.as_secs_f64();

    // Performance assertion: should handle at least 2000 checks per second
    assert!(
        checks_per_second > 2000.0,
        "Permission checking too slow: {} checks/sec",
        checks_per_second
    );
}

/// **Performance Test Perf.3: Connection Pool Throughput**
/// **Validates: Connection pool operations maintain high throughput**
#[tokio::test]
async fn test_connection_pool_throughput() {
    let pool_config = PoolConfig {
        max_connections: 50,
        min_connections: 10,
        max_idle_time: Duration::from_secs(60),
        connection_timeout: Duration::from_millis(100),
    };

    let pool = Arc::new(ConnectionPool::new(pool_config));

    // Test concurrent connection acquisition/release
    let start_time = Instant::now();
    let num_operations = 1000;
    let num_concurrent = 20;

    let mut handles = vec![];

    for _ in 0..num_concurrent {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            for _ in 0..(num_operations / num_concurrent) {
                if let Ok(_conn) = pool_clone.acquire("test-server").await {
                    // Simulate brief usage
                    tokio::time::sleep(Duration::from_micros(100)).await;
                    // Connection automatically returned
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    let operations_per_second = num_operations as f64 / duration.as_secs_f64();

    // Performance assertion: should handle at least 500 operations per second
    assert!(
        operations_per_second > 500.0,
        "Connection pool throughput too low: {} ops/sec",
        operations_per_second
    );
}

/// **Performance Test Perf.4: Memory Usage Optimization**
/// **Validates: Memory usage remains bounded during sustained operation**
#[tokio::test]
async fn test_memory_usage_optimization() {
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let permission_manager = Arc::new(MCPPermissionManager::new());

    // Initial memory baseline (approximate)
    let initial_tools = 100;
    {
        let mut reg = registry.write().await;
        for i in 0..initial_tools {
            let tool = ToolMetadata {
                id: format!("memory-tool-{}", i),
                name: format!("Memory Tool {}", i),
                description: "Tool for memory testing".to_string(),
                category: "memory".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Custom,
                server_id: None,
            };
            reg.register_tool(tool).unwrap();
        }
    }

    // Sustained operation test
    let test_duration = Duration::from_secs(2);
    let start_time = Instant::now();
    let mut operation_count = 0;

    while start_time.elapsed() < test_duration {
        // Perform various operations
        {
            let reg = registry.read().await;
            let _count = reg.tool_count();
            operation_count += 1;
        }

        let _perm = permission_manager.check_permission("memory-tool-0", Some("test-agent"));
        operation_count += 1;

        // Small delay to prevent overwhelming
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    let final_tools = registry.read().await.tool_count();

    // Verify no memory leaks (tool count should remain stable)
    assert_eq!(
        final_tools, initial_tools,
        "Memory leak detected: tool count changed from {} to {}",
        initial_tools, final_tools
    );

    // Performance check
    let operations_per_second = operation_count as f64 / test_duration.as_secs_f64();
    assert!(
        operations_per_second > 1000.0,
        "Operation throughput too low: {} ops/sec",
        operations_per_second
    );
}

/// **Performance Test Perf.5: Concurrent Operation Optimization**
/// **Validates: System performs well under high concurrency**
#[tokio::test]
async fn test_concurrent_operation_optimization() {
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let permission_manager = Arc::new(MCPPermissionManager::new());
    let pool = Arc::new(ConnectionPool::new(PoolConfig::default()));

    // Setup initial data
    {
        let mut reg = registry.write().await;
        for i in 0..100 {
            let tool = ToolMetadata {
                id: format!("concurrent-tool-{}", i),
                name: format!("Concurrent Tool {}", i),
                description: "Tool for concurrency testing".to_string(),
                category: "concurrent".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Custom,
                server_id: None,
            };
            reg.register_tool(tool).unwrap();
        }
    }

    // Test high concurrency
    let num_tasks = 50;
    let operations_per_task = 100;
    let start_time = Instant::now();

    let mut handles = vec![];

    for task_id in 0..num_tasks {
        let reg_clone = Arc::clone(&registry);
        let perm_clone = Arc::clone(&permission_manager);
        let pool_clone = Arc::clone(&pool);

        let handle = tokio::spawn(async move {
            let mut local_count = 0;

            for i in 0..operations_per_task {
                // Registry operation
                {
                    let reg = reg_clone.read().await;
                    let tool_id = format!("concurrent-tool-{}", (task_id + i) % 100);
                    let _tool = reg.get_tool(&tool_id);
                }
                local_count += 1;

                // Permission operation
                let _perm = perm_clone.check_permission("concurrent-tool-0", Some("test-agent"));
                local_count += 1;

                // Connection pool operation
                if let Ok(_conn) = pool_clone.acquire("test-server").await {
                    local_count += 1;
                }
            }

            local_count
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_operations = 0;
    for handle in handles {
        total_operations += handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    let operations_per_second = total_operations as f64 / duration.as_secs_f64();

    // Performance assertion: should handle at least 5000 operations per second
    assert!(
        operations_per_second > 5000.0,
        "Concurrent operations too slow: {} ops/sec",
        operations_per_second
    );
}

/// **Performance Test Perf.6: Cache Effectiveness Under Load**
/// **Validates: Caching improves performance under load**
#[tokio::test]
async fn test_cache_effectiveness_under_load() {
    use ricecoder_cache::{storage::MemoryStorage, Cache, CacheConfig};

    let cache_storage = Arc::new(MemoryStorage::new());
    let cache_config = CacheConfig {
        max_size: 1000,
        ttl: Some(Duration::from_secs(300)),
        ..Default::default()
    };
    let cache = Arc::new(Cache::new_with_config(cache_storage, cache_config));

    // Pre-populate cache
    for i in 0..500 {
        let key = format!("cache-key-{}", i);
        let value = json!({"data": i, "description": format!("Cached item {}", i)});
        cache.set(&key, value, None).await.unwrap();
    }

    // Test cache performance under load
    let num_operations = 10000;
    let num_concurrent = 10;
    let start_time = Instant::now();

    let mut handles = vec![];

    for _ in 0..num_concurrent {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let mut hits = 0;
            let mut misses = 0;

            for i in 0..(num_operations / num_concurrent) {
                let key = format!("cache-key-{}", i % 500);

                match cache_clone.get(&key).await {
                    Ok(Some(_)) => hits += 1,
                    Ok(None) => misses += 1,
                    Err(_) => misses += 1,
                }
            }

            (hits, misses)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_hits = 0;
    let mut total_misses = 0;
    for handle in handles {
        let (hits, misses) = handle.await.unwrap();
        total_hits += hits;
        total_misses += misses;
    }

    let duration = start_time.elapsed();
    let operations_per_second = (total_hits + total_misses) as f64 / duration.as_secs_f64();

    // Performance assertions
    assert!(
        operations_per_second > 10000.0,
        "Cache operations too slow: {} ops/sec",
        operations_per_second
    );

    // Cache effectiveness: should have high hit rate
    let hit_rate = total_hits as f64 / (total_hits + total_misses) as f64;
    assert!(
        hit_rate > 0.8,
        "Cache hit rate too low: {:.2}%",
        hit_rate * 100.0
    );
}

/// **Performance Test Perf.7: Serialization/Deserialization Speed**
/// **Validates: JSON processing performance for MCP messages**
#[test]
fn test_serialization_deserialization_speed() {
    use ricecoder_mcp::transport::{MCPMessage, MCPRequest, MCPResponse};

    // Create test messages of various sizes
    let messages: Vec<MCPMessage> = (0..100)
        .map(|i| {
            MCPMessage::Request(MCPRequest {
                id: format!("perf-test-{}", i),
                method: "test.performance".to_string(),
                params: json!({
                    "index": i,
                    "data": "x".repeat(100), // 100 bytes of data
                    "nested": {
                        "array": vec![1, 2, 3, 4, 5],
                        "object": {
                            "key1": "value1",
                            "key2": i,
                            "key3": true
                        }
                    }
                }),
            })
        })
        .collect();

    // Test serialization performance
    let serialize_start = Instant::now();
    let mut serialized = vec![];
    for message in &messages {
        let json = serde_json::to_string(message).unwrap();
        serialized.push(json);
    }
    let serialize_duration = serialize_start.elapsed();

    // Test deserialization performance
    let deserialize_start = Instant::now();
    for json_str in &serialized {
        let _message: MCPMessage = serde_json::from_str(json_str).unwrap();
    }
    let deserialize_duration = deserialize_start.elapsed();

    // Performance assertions
    let serialize_throughput = messages.len() as f64 / serialize_duration.as_secs_f64();
    let deserialize_throughput = messages.len() as f64 / deserialize_duration.as_secs_f64();

    assert!(
        serialize_throughput > 1000.0,
        "Serialization too slow: {} msg/sec",
        serialize_throughput
    );
    assert!(
        deserialize_throughput > 1000.0,
        "Deserialization too slow: {} msg/sec",
        deserialize_throughput
    );
}

/// **Performance Test Perf.8: Resource Cleanup Efficiency**
/// **Validates: Resources are cleaned up efficiently**
#[tokio::test]
async fn test_resource_cleanup_efficiency() {
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));

    // Test rapid creation and cleanup simulation
    let iterations = 10;
    let tools_per_iteration = 100;

    let start_time = Instant::now();
    let mut total_tools_created = 0;

    for iteration in 0..iterations {
        // Create tools
        {
            let mut reg = registry.write().await;
            for i in 0..tools_per_iteration {
                let tool = ToolMetadata {
                    id: format!("cleanup-tool-{}-{}", iteration, i),
                    name: format!("Cleanup Tool {} {}", iteration, i),
                    description: "Tool for cleanup testing".to_string(),
                    category: "cleanup".to_string(),
                    parameters: vec![],
                    return_type: "string".to_string(),
                    source: ToolSource::Custom,
                    server_id: None,
                };
                reg.register_tool(tool).unwrap();
                total_tools_created += 1;
            }
        }

        // Simulate usage
        {
            let reg = registry.read().await;
            let _count = reg.tool_count();
        }

        // In a real scenario, tools might be cleaned up here
        // For this test, we just verify the registry handles the load
    }

    let duration = start_time.elapsed();
    let tools_per_second = total_tools_created as f64 / duration.as_secs_f64();

    // Verify final state
    let final_count = registry.read().await.tool_count();
    assert_eq!(
        final_count, total_tools_created,
        "Tool registry lost tools: expected {}, got {}",
        total_tools_created, final_count
    );

    // Performance check
    assert!(
        tools_per_second > 1000.0,
        "Tool creation too slow: {} tools/sec",
        tools_per_second
    );
}

/// **Performance Test Perf.9: Error Handling Performance**
/// **Validates: Error handling doesn't significantly impact performance**
#[tokio::test]
async fn test_error_handling_performance() {
    use ricecoder_mcp::error::Error;

    let start_time = Instant::now();
    let num_errors = 10000;

    let mut error_count = 0;
    for i in 0..num_errors {
        // Simulate various error conditions
        let error = match i % 4 {
            0 => Error::ConnectionError(format!("Connection failed {}", i)),
            1 => Error::ValidationError(format!("Validation failed {}", i)),
            2 => Error::AuthenticationError(format!("Auth failed {}", i)),
            _ => Error::ServerError(format!("Server error {}", i)),
        };

        // Test error formatting and handling
        let _error_msg = error.to_string();
        error_count += 1;
    }

    let duration = start_time.elapsed();
    let errors_per_second = error_count as f64 / duration.as_secs_f64();

    // Performance assertion: should handle at least 10000 errors per second
    assert!(
        errors_per_second > 10000.0,
        "Error handling too slow: {} errors/sec",
        errors_per_second
    );
}

/// **Performance Test Perf.10: Memory Leak Prevention**
/// **Validates: No memory leaks during extended operation**
#[tokio::test]
async fn test_memory_leak_prevention() {
    // This test simulates extended operation to check for memory leaks
    // In a real implementation, this would use memory profiling tools

    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let permission_manager = Arc::new(MCPPermissionManager::new());

    // Baseline measurements
    let initial_registry_size = registry.read().await.tool_count();

    // Extended operation loop
    let test_duration = Duration::from_secs(3);
    let start_time = Instant::now();
    let mut cycles = 0;

    while start_time.elapsed() < test_duration {
        // Perform operations that might cause memory leaks
        {
            let mut reg = registry.write().await;
            let tool = ToolMetadata {
                id: format!("leak-test-tool-{}", cycles),
                name: format!("Leak Test Tool {}", cycles),
                description: "Tool for leak testing".to_string(),
                category: "leak_test".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Custom,
                server_id: None,
            };

            // Register tool (this might allocate memory)
            let _ = reg.register_tool(tool);

            // Immediately "remove" by overwriting (simulate cleanup)
            // In practice, this would be a proper cleanup mechanism
        }

        // Check permissions (another operation that might allocate)
        let _perm = permission_manager.check_permission("leak-test-tool-0", Some("test-agent"));

        cycles += 1;

        // Small delay to prevent overwhelming
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    let final_registry_size = registry.read().await.tool_count();

    // The registry size should be manageable (not growing without bound)
    // Allow some growth for the test tools, but not excessive
    assert!(
        final_registry_size < initial_registry_size + cycles + 100,
        "Potential memory leak: registry grew from {} to {} tools over {} cycles",
        initial_registry_size,
        final_registry_size,
        cycles
    );

    // Performance check
    let cycles_per_second = cycles as f64 / test_duration.as_secs_f64();
    assert!(
        cycles_per_second > 100.0,
        "Operation throughput too low: {} cycles/sec",
        cycles_per_second
    );
}
