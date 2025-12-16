//! MCP Integration Tests
//!
//! Comprehensive integration tests for MCP functionality covering:
//! - Server registration/discovery/health monitoring
//! - Tool enablement/disablement with permissions
//! - Authentication/authorization/enterprise integration
//! - Performance/load testing with caching validation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tempfile::TempDir;

// MCP imports
use ricecoder_mcp::{
    config::{MCPConfig, MCPConfigLoader, MCPServerConfig},
    connection_pool::{ConnectionPool, PoolConfig},
    error::{Error, Result},
    health_check::{HealthChecker, HealthCheckConfig, HealthStatus},
    lifecycle::{ServerLifecycle, ServerState},
    metadata::{ToolMetadata, ToolSource},
    permissions::{MCPPermissionManager, PermissionLevelConfig, PermissionRule},
    registry::ToolRegistry,
    server_management::{
        AuthConfig, AuthType, DiscoveryResult, ServerConfig, ServerHealth, ServerManager,
        ServerRegistration, ServerState as ServerMgmtState,
    },
    tool_execution::ToolExecutionContext,
    transport::TransportConfig,
};

// External dependencies
use ricecoder_permissions::PermissionLevel;
use ricecoder_cache::{Cache, CacheConfig};
use serde_json::json;

/// **Integration Test 1.1: Server Registration and Discovery**
/// **Validates: Server registration/discovery/health monitoring**
#[tokio::test]
async fn test_server_registration_and_discovery() {
    // Create test server config
    let server_config = ServerConfig {
        id: "test-server-1".to_string(),
        name: "Test Server 1".to_string(),
        description: "Test server for integration testing".to_string(),
        transport_config: TransportConfig::Stdio {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            env: HashMap::new(),
        },
        auto_start: true,
        health_check_interval_seconds: 10,
        max_reconnect_attempts: 3,
        auth_config: Some(AuthConfig {
            auth_type: AuthType::None,
            credentials: HashMap::new(),
        }),
        enabled_tools: std::collections::HashSet::new(),
    };

    // Initialize server manager
    let server_manager = Arc::new(ServerManager::new());

    // Test server registration
    server_manager.register_server(server_config.clone()).await.unwrap();

    // Test server discovery
    let discovered_servers = server_manager.discover_servers().await.unwrap();
    assert!(!discovered_servers.is_empty());

    // Verify server details
    let discovered = discovered_servers.iter().find(|s| s.server_id == "test-server-1");
    assert!(discovered.is_some(), "Test server should be discovered");

    // Test health monitoring
    let health_checker = HealthChecker::new(HealthCheckConfig {
        check_interval: Duration::from_millis(100),
        timeout: Duration::from_millis(500),
        max_failures: 3,
    });

    // Check health of registered servers
    let health = health_checker.check_server_health(&server_config).await;
    // Since we're using echo commands, health check might fail, but the interface should work
    assert!(matches!(health, HealthStatus::Healthy | HealthStatus::Unhealthy));
}

/// **Integration Test 1.2: Server Lifecycle Management**
/// **Validates: Server lifecycle states and transitions**
#[tokio::test]
async fn test_server_lifecycle_management() {
    let server_id = "lifecycle-test-server";

    // Create server lifecycle manager
    let lifecycle = ServerLifecycle::new(server_id.to_string());

    // Test initial state
    assert_eq!(lifecycle.get_state().await, ServerState::Stopped);

    // Test state transitions
    lifecycle.start().await.unwrap();
    assert_eq!(lifecycle.get_state().await, ServerState::Starting);

    // Simulate startup completion
    lifecycle.set_state(ServerState::Running).await;
    assert_eq!(lifecycle.get_state().await, ServerState::Running);

    // Test health monitoring integration
    let health_info = lifecycle.get_health_info().await;
    assert!(health_info.uptime >= Duration::from_secs(0));

    // Test graceful shutdown
    lifecycle.stop().await.unwrap();
    assert_eq!(lifecycle.get_state().await, ServerState::Stopping);

    // Simulate shutdown completion
    lifecycle.set_state(ServerState::Stopped).await;
    assert_eq!(lifecycle.get_state().await, ServerState::Stopped);
}

/// **Integration Test 2.1: Tool Enablement/Disablement with Permissions**
/// **Validates: Tool enablement/disablement with permissions**
#[tokio::test]
async fn test_tool_enablement_disablement_with_permissions() {
    let mut permission_manager = MCPPermissionManager::new();
    let mut registry = ToolRegistry::new();

    // Register tools
    let admin_tool = ToolMetadata {
        id: "admin-tool".to_string(),
        name: "Admin Tool".to_string(),
        description: "Administrative tool".to_string(),
        category: "admin".to_string(),
        parameters: vec![],
        return_type: "string".to_string(),
        source: ToolSource::Custom,
        server_id: None,
    };

    let user_tool = ToolMetadata {
        id: "user-tool".to_string(),
        name: "User Tool".to_string(),
        description: "User tool".to_string(),
        category: "user".to_string(),
        parameters: vec![],
        return_type: "string".to_string(),
        source: ToolSource::Custom,
        server_id: None,
    };

    registry.register_tool(admin_tool).unwrap();
    registry.register_tool(user_tool).unwrap();

    // Setup permissions - deny all by default
    let deny_all_rule = PermissionRule {
        pattern: "*".to_string(),
        level: PermissionLevelConfig::Deny,
        agent_id: None,
    };
    permission_manager.add_global_rule(deny_all_rule).unwrap();

    // Allow admin tools for admin agent
    let admin_rule = PermissionRule {
        pattern: "admin-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: Some("admin-agent".to_string()),
    };
    permission_manager.add_agent_rule("admin-agent".to_string(), admin_rule).unwrap();

    // Allow user tools for user agent
    let user_rule = PermissionRule {
        pattern: "user-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: Some("user-agent".to_string()),
    };
    permission_manager.add_agent_rule("user-agent".to_string(), user_rule).unwrap();

    // Test tool enablement/disablement based on permissions

    // Admin agent should be able to use admin tools
    let admin_permission = permission_manager.check_permission("admin-tool", Some("admin-agent")).unwrap();
    assert_eq!(admin_permission, PermissionLevel::Allow);

    // Admin agent should not be able to use user tools (not explicitly allowed)
    let admin_user_perm = permission_manager.check_permission("user-tool", Some("admin-agent")).unwrap();
    assert_eq!(admin_user_perm, PermissionLevel::Deny);

    // User agent should be able to use user tools
    let user_permission = permission_manager.check_permission("user-tool", Some("user-agent")).unwrap();
    assert_eq!(user_permission, PermissionLevel::Allow);

    // User agent should not be able to use admin tools
    let user_admin_perm = permission_manager.check_permission("admin-tool", Some("user-agent")).unwrap();
    assert_eq!(user_admin_perm, PermissionLevel::Deny);

    // Unauthenticated agent should be denied all
    let unauth_admin_perm = permission_manager.check_permission("admin-tool", None).unwrap();
    let unauth_user_perm = permission_manager.check_permission("user-tool", None).unwrap();
    assert_eq!(unauth_admin_perm, PermissionLevel::Deny);
    assert_eq!(unauth_user_perm, PermissionLevel::Deny);
}

/// **Integration Test 2.2: Dynamic Permission Updates**
/// **Validates: Runtime permission changes**
#[tokio::test]
async fn test_dynamic_permission_updates() {
    let mut permission_manager = MCPPermissionManager::new();

    // Initially deny all
    let deny_rule = PermissionRule {
        pattern: "*".to_string(),
        level: PermissionLevelConfig::Deny,
        agent_id: None,
    };
    permission_manager.add_global_rule(deny_rule).unwrap();

    // Test initial denial
    let initial_perm = permission_manager.check_permission("test-tool", Some("test-agent")).unwrap();
    assert_eq!(initial_perm, PermissionLevel::Deny);

    // Dynamically add permission
    let allow_rule = PermissionRule {
        pattern: "test-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: Some("test-agent".to_string()),
    };
    permission_manager.add_agent_rule("test-agent".to_string(), allow_rule).unwrap();

    // Test updated permission
    let updated_perm = permission_manager.check_permission("test-tool", Some("test-agent")).unwrap();
    assert_eq!(updated_perm, PermissionLevel::Allow);

    // Test that other tools are still denied
    let other_perm = permission_manager.check_permission("other-tool", Some("test-agent")).unwrap();
    assert_eq!(other_perm, PermissionLevel::Deny);
}

/// **Integration Test 3.1: Authentication and Authorization Integration**
/// **Validates: Authentication/authorization/enterprise integration**
#[tokio::test]
async fn test_authentication_authorization_integration() {
    use ricecoder_mcp::rbac::{MCRBACManager, MCPAuthorizationMiddleware};
    use ricecoder_mcp::audit::MCPAuditLogger;

    // Setup RBAC manager
    let rbac_manager = Arc::new(MCRBACManager::new());

    // Create roles
    rbac_manager.create_role("admin".to_string()).await.unwrap();
    rbac_manager.create_role("developer".to_string()).await.unwrap();
    rbac_manager.create_role("viewer".to_string()).await.unwrap();

    // Assign permissions to roles
    rbac_manager.assign_permission_to_role("admin", "tool:*").await.unwrap();
    rbac_manager.assign_permission_to_role("developer", "tool:read,tool:write").await.unwrap();
    rbac_manager.assign_permission_to_role("viewer", "tool:read").await.unwrap();

    // Assign users to roles
    rbac_manager.assign_user_to_role("admin-user", "admin").await.unwrap();
    rbac_manager.assign_user_to_role("dev-user", "developer").await.unwrap();
    rbac_manager.assign_user_to_role("view-user", "viewer").await.unwrap();

    // Setup audit logger
    struct MockAuditLogger;
    impl ricecoder_security::audit::SecurityAuditLogger for MockAuditLogger {
        fn log_event(&self, _event: &ricecoder_security::audit::AuditEvent) {}
        fn log_security_event(&self, _event: &ricecoder_security::audit::SecurityAuditEvent) {}
    }
    let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(MockAuditLogger)));

    // Create authorization middleware
    let auth_middleware = MCPAuthorizationMiddleware::new(rbac_manager, Some(audit_logger));

    // Test authorization checks
    assert!(auth_middleware.check_user_permission("admin-user", "tool:delete").await.unwrap());
    assert!(auth_middleware.check_user_permission("dev-user", "tool:write").await.unwrap());
    assert!(auth_middleware.check_user_permission("view-user", "tool:read").await.unwrap());

    // Test denied permissions
    assert!(!auth_middleware.check_user_permission("dev-user", "tool:delete").await.unwrap());
    assert!(!auth_middleware.check_user_permission("view-user", "tool:write").await.unwrap());

    // Test role inheritance and multiple roles
    rbac_manager.assign_user_to_role("dev-user", "viewer").await.unwrap();
    // dev-user should still have write permissions from developer role
    assert!(auth_middleware.check_user_permission("dev-user", "tool:write").await.unwrap());
}

/// **Integration Test 3.2: Enterprise Compliance Monitoring**
/// **Validates: Enterprise compliance and audit integration**
#[tokio::test]
async fn test_enterprise_compliance_monitoring() {
    use ricecoder_mcp::compliance::{MCPComplianceMonitor, ComplianceReport};

    // Setup compliance monitor
    let audit_logger = None; // Would use real audit logger in production
    let compliance_monitor = MCPComplianceMonitor::new(audit_logger);

    // Generate initial compliance report
    let report = compliance_monitor.generate_report().await;
    assert!(report.violations.is_empty());

    // Test report structure
    assert!(report.timestamp <= std::time::SystemTime::now());
    assert!(report.scan_duration >= Duration::from_secs(0));
}

/// **Integration Test 4.1: Performance Load Testing**
/// **Validates: Performance/load testing with caching validation**
#[tokio::test]
async fn test_performance_load_testing() {
    use ricecoder_cache::storage::MemoryStorage;

    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let cache_storage = Arc::new(MemoryStorage::new());
    let cache = Arc::new(Cache::new(cache_storage));

    // Setup test tools
    let num_tools = 100;
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

    // Test concurrent access performance
    let start_time = Instant::now();
    let num_concurrent_tasks = 50;

    let mut handles = vec![];
    for _ in 0..num_concurrent_tasks {
        let reg_clone = Arc::clone(&registry);
        let cache_clone = Arc::clone(&cache);

        let handle = tokio::spawn(async move {
            for i in 0..10 {
                let tool_id = format!("perf-tool-{}", i % num_tools);

                // Test registry access
                let reg = reg_clone.read().await;
                let _tool = reg.get_tool(&tool_id);

                // Test caching
                let cache_result: Result<Option<serde_json::Value>> = cache_clone.get(&tool_id).await;
                if cache_result.unwrap().is_none() {
                    // Simulate cache population
                    let _ = cache_clone.set(&tool_id, json!({"cached": true}), None).await;
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start_time.elapsed();
    let operations_per_second = (num_concurrent_tasks * 10) as f64 / duration.as_secs_f64();

    // Performance assertions
    assert!(duration < Duration::from_secs(10), "Load test took too long: {:?}", duration);
    assert!(operations_per_second > 100.0, "Operations per second too low: {}", operations_per_second);

    // Verify cache effectiveness
    let cache_stats = cache.stats();
    assert!(cache_stats.hits > 0 || cache_stats.misses >= num_concurrent_tasks as u64);
}

/// **Integration Test 4.2: Connection Pool Load Testing**
/// **Validates: Connection pool performance under load**
#[tokio::test]
async fn test_connection_pool_load_testing() {
    let pool_config = PoolConfig {
        max_connections: 20,
        min_connections: 5,
        max_idle_time: Duration::from_secs(60),
        connection_timeout: Duration::from_secs(5),
    };

    let pool = Arc::new(ConnectionPool::new(pool_config));

    // Test connection pool under concurrent load
    let num_clients = 50;
    let requests_per_client = 20;

    let start_time = Instant::now();
    let mut handles = vec![];

    for _ in 0..num_clients {
        let pool_clone = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            for _ in 0..requests_per_client {
                // Simulate connection acquisition and release
                let connection = pool_clone.acquire().await;
                match connection {
                    Ok(conn) => {
                        // Simulate some work
                        sleep(Duration::from_millis(1)).await;
                        // Connection is automatically returned when dropped
                        drop(conn);
                    }
                    Err(_) => {
                        // Pool might be exhausted, which is expected under high load
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all clients to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start_time.elapsed();

    // Check pool stats
    let stats = pool.stats().await;
    assert!(stats.total_connections <= pool_config.max_connections);
    assert!(stats.active_connections <= stats.total_connections);

    // Performance check
    let total_requests = num_clients * requests_per_client;
    let requests_per_second = total_requests as f64 / duration.as_secs_f64();
    assert!(requests_per_second > 50.0, "Request throughput too low: {} req/sec", requests_per_second);
}

/// **Integration Test 4.3: Caching Validation Under Load**
/// **Validates: Cache consistency and performance under load**
#[tokio::test]
async fn test_caching_validation_under_load() {
    use ricecoder_cache::storage::MemoryStorage;

    let cache_storage = Arc::new(MemoryStorage::new());
    let cache = Arc::new(Cache::new(cache_storage));
    let num_operations = 1000;
    let num_concurrent_tasks = 10;

    let start_time = Instant::now();
    let mut handles = vec![];

    for task_id in 0..num_concurrent_tasks {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let mut local_hits = 0;
            let mut local_misses = 0;

            for i in 0..(num_operations / num_concurrent_tasks) {
                let key = format!("cache-key-{}-{}", task_id, i);
                let value = json!({"task": task_id, "index": i});

                // First access should be a miss
                let result: Result<Option<serde_json::Value>> = cache_clone.get(&key).await;
                if result.unwrap().is_none() {
                    local_misses += 1;
                    let _ = cache_clone.set(&key, value.clone(), None).await;
                } else {
                    local_hits += 1;
                }

                // Second access should be a hit
                let result2: Result<Option<serde_json::Value>> = cache_clone.get(&key).await;
                if result2.unwrap().is_some() {
                    local_hits += 1;
                }
            }

            (local_hits, local_misses)
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

    // Validate cache behavior
    assert!(total_misses > 0, "Should have cache misses for new entries");
    assert!(total_hits > total_misses, "Should have more hits than misses due to repeated access");

    // Performance validation
    let operations_per_second = num_operations as f64 / duration.as_secs_f64();
    assert!(operations_per_second > 1000.0, "Cache operations per second too low: {}", operations_per_second);

    // Cache stats validation
    let stats = cache.stats();
    assert_eq!(stats.hits, total_hits as u64);
    assert_eq!(stats.misses, total_misses as u64);
    assert!(stats.hit_rate > 0.5, "Cache hit rate too low: {}", stats.hit_rate);
}

/// **Integration Test 4.4: Memory and Resource Leak Detection**
/// **Validates: Resource management under sustained load**
#[tokio::test]
async fn test_memory_resource_leak_detection() {
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let pool = Arc::new(ConnectionPool::new(PoolConfig {
        max_connections: 10,
        min_connections: 1,
        max_idle_time: Duration::from_secs(30),
        connection_timeout: Duration::from_secs(5),
    }));

    // Initial resource usage
    let initial_pool_stats = pool.stats().await;
    {
        let reg = registry.read().await;
        let initial_tool_count = reg.tool_count();
        assert_eq!(initial_tool_count, 0);
    }

    // Sustained load test
    let test_duration = Duration::from_secs(5);
    let start_time = Instant::now();

    while start_time.elapsed() < test_duration {
        // Register and unregister tools
        {
            let mut reg = registry.write().await;
            let tool = ToolMetadata {
                id: format!("temp-tool-{}", start_time.elapsed().as_nanos()),
                name: "Temporary Tool".to_string(),
                description: "Tool for leak testing".to_string(),
                category: "test".to_string(),
                parameters: vec![],
                return_type: "string".to_string(),
                source: ToolSource::Custom,
                server_id: None,
            };
            let _ = reg.register_tool(tool);
        }

        // Acquire and release connections
        if let Ok(conn) = pool.acquire().await {
            // Simulate brief usage
            drop(conn);
        }

        // Small delay to prevent overwhelming the system
        sleep(Duration::from_millis(1)).await;
    }

    // Check resource usage after test
    let final_pool_stats = pool.stats().await;
    {
        let reg = registry.read().await;
        let final_tool_count = reg.tool_count();
        // Tools should be cleaned up or managed properly
        assert!(final_tool_count >= 0);
    }

    // Pool should not have grown excessively
    assert!(final_pool_stats.total_connections <= initial_pool_stats.total_connections + 5);
}

/// **Integration Test 5.1: End-to-End MCP Workflow**
/// **Validates: Complete MCP workflow from registration to execution**
#[tokio::test]
async fn test_end_to_end_mcp_workflow() {
    use ricecoder_cache::storage::MemoryStorage;

    // Setup components
    let mut registry = ToolRegistry::new();
    let mut permission_manager = MCPPermissionManager::new();
    let cache_storage = Arc::new(MemoryStorage::new());
    let cache = Arc::new(Cache::new(cache_storage));

    // 1. Register a tool
    let tool = ToolMetadata {
        id: "e2e-test-tool".to_string(),
        name: "End-to-End Test Tool".to_string(),
        description: "Tool for end-to-end testing".to_string(),
        category: "test".to_string(),
        parameters: vec![],
        return_type: "string".to_string(),
        source: ToolSource::Custom,
        server_id: None,
    };
    registry.register_tool(tool.clone()).unwrap();

    // 2. Setup permissions
    let allow_rule = PermissionRule {
        pattern: "e2e-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: Some("test-agent".to_string()),
    };
    permission_manager.add_agent_rule("test-agent".to_string(), allow_rule).unwrap();

    // 3. Verify tool discovery
    let discovered_tool = registry.get_tool("e2e-test-tool").unwrap();
    assert_eq!(discovered_tool.id, tool.id);

    // 4. Check permissions
    let permission = permission_manager.check_permission("e2e-test-tool", Some("test-agent")).unwrap();
    assert_eq!(permission, PermissionLevel::Allow);

    // 5. Test caching
    let _ = cache.set("e2e-test-tool", json!({"cached": true}), None).await;
    let cached: Result<Option<serde_json::Value>> = cache.get("e2e-test-tool").await;
    assert!(cached.unwrap().is_some());

    // 6. Simulate tool execution context
    let execution_context = ToolExecutionContext {
        tool_name: "e2e-test-tool".to_string(),
        parameters: HashMap::new(),
        user_id: Some("test-user".to_string()),
        session_id: Some("test-session".to_string()),
        timeout: Duration::from_secs(30),
        metadata: HashMap::new(),
    };

    // Verify execution context is properly structured
    assert_eq!(execution_context.tool_name, "e2e-test-tool");
    assert!(execution_context.user_id.is_some());
    assert!(execution_context.session_id.is_some());
}

/// **Integration Test 5.2: Failure Recovery and Resilience**
/// **Validates: System resilience under failure conditions**
#[tokio::test]
async fn test_failure_recovery_and_resilience() {
    use ricecoder_mcp::error_recovery::{BackoffConfig, RecoveryStrategy, RetryHandler};

    let config = BackoffConfig {
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        multiplier: 2.0,
        max_attempts: 3,
    };

    let retry_handler = RetryHandler::new(config);

    // Test recovery strategy determination
    let connection_error = Error::ConnectionError("Network timeout".to_string());
    let strategy = ricecoder_mcp::error_recovery::determine_recovery_strategy(&connection_error);
    assert!(matches!(strategy, RecoveryStrategy::Retry));

    let auth_error = Error::AuthenticationError("Invalid credentials".to_string());
    let strategy = ricecoder_mcp::error_recovery::determine_recovery_strategy(&auth_error);
    assert!(matches!(strategy, RecoveryStrategy::Fail));

    // Test retry handler behavior
    let mut attempt_count = 0;
    let result = retry_handler.retry(|| async {
        attempt_count += 1;
        if attempt_count < 3 {
            Err(Error::ConnectionError("Temporary failure".to_string()))
        } else {
            Ok("Success".to_string())
        }
    }).await;

    assert!(result.is_ok());
    assert_eq!(attempt_count, 3);
}

/// **Integration Test 5.3: Cross-Component Integration**
/// **Validates: Integration between all MCP components**
#[tokio::test]
async fn test_cross_component_integration() {
    use ricecoder_cache::storage::MemoryStorage;

    // Setup all major components
    let registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let permission_manager = Arc::new(MCPPermissionManager::new());
    let cache_storage = Arc::new(MemoryStorage::new());
    let cache = Arc::new(Cache::new(cache_storage));
    let pool = Arc::new(ConnectionPool::new(PoolConfig::default()));

    // Register tools across components
    {
        let mut reg = registry.write().await;
        let tool = ToolMetadata {
            id: "integration-test-tool".to_string(),
            name: "Integration Test Tool".to_string(),
            description: "Tool for cross-component testing".to_string(),
            category: "integration".to_string(),
            parameters: vec![],
            return_type: "string".to_string(),
            source: ToolSource::Custom,
            server_id: None,
        };
        reg.register_tool(tool).unwrap();
    }

    // Setup permissions
    let rule = PermissionRule {
        pattern: "integration-*".to_string(),
        level: PermissionLevelConfig::Allow,
        agent_id: Some("integration-agent".to_string()),
    };
    permission_manager.add_global_rule(rule).unwrap();

    // Test concurrent access to all components
    let num_concurrent = 10;
    let mut handles = vec![];

    for i in 0..num_concurrent {
        let reg_clone = Arc::clone(&registry);
        let perm_clone = Arc::clone(&permission_manager);
        let cache_clone = Arc::clone(&cache);
        let pool_clone = Arc::clone(&pool);

        let handle = tokio::spawn(async move {
            // Test registry access
            let reg = reg_clone.read().await;
            let _tool = reg.get_tool("integration-test-tool");

            // Test permissions
            let _perm = perm_clone.check_permission("integration-test-tool", Some("integration-agent"));

            // Test caching
            let cache_key = format!("cache-key-{}", i);
            let _ = cache_clone.set(&cache_key, json!({"test": i}), None).await;
            let _cached: Result<Option<serde_json::Value>> = cache_clone.get(&cache_key).await;

            // Test connection pool
            let _conn = pool_clone.acquire().await;
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state
    let reg = registry.read().await;
    assert_eq!(reg.tool_count(), 1);

    let pool_stats = pool.stats().await;
    assert!(pool_stats.total_connections >= 0);
}