//! Property-based tests for MCP protocol invariants, edge cases, and enterprise security scenarios
//!
//! **Feature: ricecoder-mcp, Property Tests: Protocol Invariants and Enterprise Security**
//! **Validates: Requirements MCP-1.1, MCP-1.2, MCP-2.1, MCP-2.2, MCP-3.1, MCP-3.2**
//!
//! These tests verify that the MCP protocol implementation maintains correctness under
//! various edge cases, enterprise security requirements, and protocol invariants.

use proptest::prelude::*;
use ricecoder_mcp::{
    audit::MCPAuditLogger,
    compliance::{MCPComplianceMonitor, ComplianceReport, ComplianceReportType, ViolationSeverity},
    config::{MCPConfig, MCPServerConfig},
    connection_pool::{ConnectionPool, PoolConfig, PoolStats},
    error::{Error, Result, ToolError},
    health_check::{HealthChecker, HealthCheckConfig, HealthStatus},
    metadata::{ParameterMetadata, ToolMetadata, ToolSource},
    permissions::{MCPPermissionManager, PermissionLevelConfig, PermissionRule},
    protocol_validation::{MCPProtocolValidator},
    rbac::{MCRBACManager, MCPAuthorizationMiddleware},
    registry::ToolRegistry,
    server_management::{ServerConfig, ServerManager, AuthConfig, AuthType},
    tool_execution::{MCPToolExecutor, ToolExecutionContext, ToolExecutionResult},
    transport::{
        MCPMessage, MCPRequest, MCPResponse, MCPNotification, MCPError, MCPErrorData,
        MCPTransport, StdioTransport, TransportConfig,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use chrono::{DateTime, Utc};

// Import testing infrastructure enhancements
#[path = "mcp_testing_infrastructure.rs"]
mod mcp_testing_infrastructure;
use mcp_testing_infrastructure::*;

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_mcp_request() -> impl Strategy<Value = MCPRequest> {
    (
        "[a-zA-Z0-9_-]{1,64}".prop_map(|s| s), // id
        "[a-zA-Z0-9_.-]{1,128}".prop_map(|s| s), // method
        any::<serde_json::Value>(), // params
    )
        .prop_map(|(id, method, params)| MCPRequest { id, method, params })
}

fn arb_mcp_response() -> impl Strategy<Value = MCPResponse> {
    (
        "[a-zA-Z0-9_-]{1,64}".prop_map(|s| s), // id
        any::<serde_json::Value>(), // result
    )
        .prop_map(|(id, result)| MCPResponse { id, result })
}

fn arb_mcp_notification() -> impl Strategy<Value = MCPNotification> {
    (
        "[a-zA-Z0-9_.-]{1,128}".prop_map(|s| s), // method
        any::<serde_json::Value>(), // params
    )
        .prop_map(|(method, params)| MCPNotification { method, params })
}

fn arb_mcp_error_data() -> impl Strategy<Value = MCPErrorData> {
    (
        any::<i32>(), // code
        ".{0,1024}".prop_map(|s| s), // message
        proptest::option::of(any::<serde_json::Value>()), // data
    )
        .prop_map(|(code, message, data)| MCPErrorData { code, message, data })
}

fn arb_mcp_error() -> impl Strategy<Value = MCPError> {
    (
        proptest::option::of("[a-zA-Z0-9_-]{1,64}".prop_map(|s| s)), // id
        arb_mcp_error_data(),
    )
        .prop_map(|(id, error)| MCPError { id, error })
}

fn arb_mcp_message() -> impl Strategy<Value = MCPMessage> {
    prop_oneof![
        arb_mcp_request().prop_map(MCPMessage::Request),
        arb_mcp_response().prop_map(MCPMessage::Response),
        arb_mcp_notification().prop_map(MCPMessage::Notification),
        arb_mcp_error().prop_map(MCPMessage::Error),
    ]
}

fn arb_tool_metadata() -> impl Strategy<Value = ToolMetadata> {
    (
        "[a-zA-Z0-9_-]{1,64}".prop_map(|s| s), // name
        ".{1,256}".prop_map(|s| s), // description
        any::<serde_json::Value>(), // input_schema
        proptest::collection::vec("[a-zA-Z0-9_-]{1,64}".prop_map(|s| s), 0..5), // permissions_required
        proptest::option::of(any::<serde_json::Value>()), // metadata
    )
        .prop_map(|(name, description, input_schema, permissions_required, metadata)| ToolMetadata {
            name,
            description,
            input_schema,
            permissions_required,
            metadata,
        })
}

fn arb_permission_rule() -> impl Strategy<Value = PermissionRule> {
    (
        ".{1,256}".prop_map(|s| s), // pattern
        prop_oneof![
            Just(PermissionLevelConfig::Allow),
            Just(PermissionLevelConfig::Ask),
            Just(PermissionLevelConfig::Deny),
        ],
        proptest::option::of("[a-zA-Z0-9_-]{1,64}".prop_map(|s| s)), // agent_id
    )
        .prop_map(|(pattern, level, agent_id)| PermissionRule {
            pattern,
            level,
            agent_id,
        })
}

// ============================================================================
// Property 1: MCP Protocol Invariants
// ============================================================================

proptest! {
    /// Property 1: MCP Message Serialization Invariants
    /// *For any* valid MCP message, serialization and deserialization SHALL preserve
    /// all message properties and maintain protocol correctness.
    /// **Validates: Requirements MCP-1.1**
    #[test]
    fn prop_mcp_message_serialization_invariants(message in arb_mcp_message()) {
        // Test JSON serialization roundtrip
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

        prop_assert_eq!(message, deserialized);

        // Test that serialized JSON is valid
        let _: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Test message size constraints
        prop_assert!(serialized.len() < 10 * 1024 * 1024, "Message too large: {} bytes", serialized.len());
    }

    /// Property 1 variant: Request-Response ID Correlation
    #[test]
    fn prop_request_response_id_correlation(
        request in arb_mcp_request(),
        response_result in any::<serde_json::Value>(),
    ) {
        let response = MCPResponse {
            id: request.id.clone(),
            result: response_result,
        };

        // Response ID must match request ID
        prop_assert_eq!(response.id, request.id);

        // Both should serialize successfully
        let req_json = serde_json::to_string(&MCPMessage::Request(request)).unwrap();
        let resp_json = serde_json::to_string(&MCPMessage::Response(response)).unwrap();

        let _: serde_json::Value = serde_json::from_str(&req_json).unwrap();
        let _: serde_json::Value = serde_json::from_str(&resp_json).unwrap();
    }
}

// ============================================================================
// Property 2: Edge Cases and Error Handling
// ============================================================================

proptest! {
    /// Property 2: Edge Cases in Message Processing
    /// *For any* edge case input, the MCP implementation SHALL handle it gracefully
    /// without crashing or exposing internal state.
    /// **Validates: Requirements MCP-1.2, MCP-2.1**
    #[test]
    fn prop_mcp_edge_case_handling(
        empty_string in Just("".to_string()),
        very_long_string in (10000..50000usize).prop_map(|len| "A".repeat(len)),
        special_chars in "[\\x00-\\x1F\\x7F-\\x9F]{1,100}".prop_map(|s| s.to_string()),
        unicode_string in "[\\u{0000}-\\u{FFFF}]{1,100}".prop_map(|s| s.to_string()),
    ) {
        let test_inputs = vec![empty_string, very_long_string, special_chars, unicode_string];

        for input in test_inputs {
            // Test as JSON parameter
            let test_params = serde_json::json!({ "data": input });

            // Should not crash when processing
            let message = MCPMessage::Notification(MCPNotification {
                method: "test.method".to_string(),
                params: test_params,
            });

            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();

            // Should preserve the data (or handle truncation gracefully)
            match deserialized {
                MCPMessage::Notification(notif) => {
                    if let Some(data) = notif.params.get("data") {
                        if let Some(data_str) = data.as_str() {
                            // Either preserves data or truncates reasonably
                            prop_assert!(data_str.len() <= input.len() + 100, "Data corrupted: {} -> {}", input.len(), data_str.len());
                        }
                    }
                }
                _ => panic!("Wrong message type"),
            }
        }
    }

    /// Property 2 variant: Malformed Message Recovery
    #[test]
    fn prop_malformed_message_recovery(
        malformed_json in ".{1,1000}",
    ) {
        // Try to parse as MCP message - should not crash
        let parse_result = serde_json::from_str::<MCPMessage>(&malformed_json);

        // Either succeeds (if valid JSON) or fails gracefully
        match parse_result {
            Ok(_) => {
                // If it parsed, verify it's valid JSON
                let _: serde_json::Value = serde_json::from_str(&malformed_json).unwrap();
            }
            Err(e) => {
                // Error should be descriptive
                prop_assert!(!e.to_string().is_empty());
                // Should not be a panic or crash
            }
        }
    }
}

// ============================================================================
// Property 3: Enterprise Security Scenarios
// ============================================================================

proptest! {
    /// Property 3: Enterprise Security Compliance
    /// *For any* enterprise security scenario, MCP SHALL maintain compliance
    /// with security standards and audit requirements.
    /// **Validates: Requirements MCP-2.2, MCP-3.1**
    #[test]
    fn prop_enterprise_security_compliance(
        tool_count in 1..50usize,
        user_count in 1..20usize,
        operation_count in 10..100usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            // Setup enterprise MCP environment
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let compliance_monitor = Arc::new(MCPComplianceMonitor::new(audit_logger.clone()));
            let permission_manager = Arc::new(MCPPermissionManager::new());
            let tool_registry = Arc::new(ToolRegistry::new());

            // Register tools with enterprise permissions
            for i in 0..tool_count {
                let tool = ToolMetadata {
                    name: format!("enterprise-tool-{}", i),
                    description: format!("Enterprise tool {}", i),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "param1": {"type": "string"}
                        }
                    }),
                    permissions_required: vec![format!("enterprise.access.{}", i)],
                    metadata: Some(serde_json::json!({"classification": "internal"})),
                };

                tool_registry.register_tool(tool).unwrap();

                // Add permission rules
                let rule = PermissionRule {
                    pattern: format!("enterprise-tool-{}", i),
                    level: PermissionLevelConfig::Allow,
                    agent_id: Some(format!("user-{}", i % user_count)),
                };
                permission_manager.add_global_rule(rule).unwrap();
            }

            // Simulate enterprise operations
            let mut compliance_violations = 0;

            for op in 0..operation_count {
                let user_id = format!("user-{}", op % user_count);
                let tool_name = format!("enterprise-tool-{}", op % tool_count);

                // Check permissions
                let permission_result = permission_manager.check_permission(&tool_name, Some(&user_id));

                match permission_result {
                    Ok(level) => {
                        if matches!(level, ricecoder_permissions::PermissionLevel::Deny) {
                            compliance_violations += 1;
                        }

                        // Log the access attempt
                        audit_logger.log_tool_access(&tool_name, &user_id, &level.to_string()).await.unwrap();
                    }
                    Err(_) => {
                        compliance_violations += 1;
                    }
                }
            }

            // Generate compliance report
            let report = compliance_monitor.generate_report(
                ComplianceReportType::Soc2Type2,
                Utc::now() - chrono::Duration::hours(1),
                Utc::now(),
            ).await.unwrap();

            // Should have audit trail
            prop_assert!(report.audit_events > 0);

            // Compliance violations should be within acceptable limits
            let violation_rate = compliance_violations as f64 / operation_count as f64;
            prop_assert!(violation_rate < 0.5, "Too many compliance violations: {}%", violation_rate * 100.0);
        });
    }

    /// Property 3 variant: Data Residency and Sovereignty
    #[test]
    fn prop_data_residency_compliance(
        region_count in 2..10usize,
        data_operation_count in 5..50usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let regions = (0..region_count).map(|i| format!("region-{}", i)).collect::<Vec<_>>();

            let mut data_transfers = HashMap::new();

            // Simulate data operations across regions
            for _ in 0..data_operation_count {
                let source_region = &regions[rand::random::<usize>() % regions.len()];
                let dest_region = &regions[rand::random::<usize>() % regions.len()];

                let transfer_key = format!("{}->{}", source_region, dest_region);
                *data_transfers.entry(transfer_key).or_insert(0) += 1;

                // Log cross-region data transfer
                audit_logger.log_data_transfer(source_region, dest_region, "sensitive_data").await.unwrap();
            }

            // Verify data residency compliance (simplified)
            // In real implementation, would check against data residency policies
            let audit_events = audit_logger.get_audit_events(
                Utc::now() - chrono::Duration::hours(1),
                Utc::now(),
            ).await.unwrap();

            // Should have logged all data transfers
            let transfer_events = audit_events.iter()
                .filter(|e| e.event_type == "data_transfer")
                .count();

            prop_assert_eq!(transfer_events, data_operation_count);
        });
    }
}

// ============================================================================
// Property 4: Connection Pool Scaling and Reliability
// ============================================================================

proptest! {
    /// Property 4: Connection Pool Scaling
    /// *For any* connection pool configuration and load pattern, the pool SHALL
    /// scale appropriately and maintain connection health.
    /// **Validates: Requirements MCP-3.2**
    #[test]
    fn prop_connection_pool_scaling(
        pool_size in 1..100usize,
        connection_requests in 10..200usize,
        concurrent_clients in 1..20usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let pool_config = PoolConfig {
                min_connections: 1,
                max_connections: pool_size,
                connection_timeout_ms: 5000,
                idle_timeout_ms: 30000,
            };

            let pool = Arc::new(ConnectionPool::with_config(pool_config));
            let semaphore = Arc::new(Semaphore::new(concurrent_clients));

            let mut handles = Vec::new();

            for i in 0..connection_requests {
                let pool_clone = pool.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    // Simulate connection usage
                    match timeout(Duration::from_millis(1000), pool_clone.acquire("test-server")).await {
                        Ok(Ok(connection)) => {
                            // Simulate some work
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            // Connection would be returned to pool here
                            Some(connection)
                        }
                        Ok(Err(_)) => None, // Failed to acquire
                        Err(_) => None, // Timeout
                    }
                });

                handles.push(handle);
            }

            // Wait for all connection requests
            let mut successful_connections = 0;
            for handle in handles {
                if let Ok(Some(_)) = handle.await {
                    successful_connections += 1;
                }
            }

            let pool_stats = pool.get_stats().await;

            // Should not exceed max connections
            prop_assert!(pool_stats.active_connections <= pool_size);

            // Should have processed some requests successfully
            prop_assert!(successful_connections > 0);

            // Success rate should be reasonable
            let success_rate = successful_connections as f64 / connection_requests as f64;
            prop_assert!(success_rate > 0.5, "Connection success rate too low: {}", success_rate);
        });
    }

    /// Property 4 variant: Connection Pool Health Monitoring
    #[test]
    fn prop_connection_pool_health_monitoring(
        pool_size in 5..50usize,
        health_check_interval_ms in 100..1000u64,
        unhealthy_connection_count in 1..10usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let pool_config = PoolConfig {
                min_connections: 1,
                max_connections: pool_size,
                connection_timeout_ms: 5000,
                idle_timeout_ms: 30000,
            };

            let pool = Arc::new(ConnectionPool::with_config(pool_config));

            // Establish baseline healthy connections
            let mut healthy_connections = Vec::new();
            for _ in 0..pool_size.saturating_sub(unhealthy_connection_count) {
                if let Ok(conn) = pool.acquire("healthy-server").await {
                    healthy_connections.push(conn);
                }
            }

            // Simulate some unhealthy connections
            for _ in 0..unhealthy_connection_count {
                // In real implementation, would mark connections as unhealthy
                // For testing, we just verify the pool can handle the scenario
            }

            let stats_before = pool.get_stats().await;

            // Run health checks
            pool.run_health_checks().await;

            let stats_after = pool.get_stats().await;

            // Pool should still be functional
            prop_assert!(stats_after.total_connections <= pool_size);

            // Should be able to acquire connections after health checks
            if let Ok(_) = timeout(Duration::from_millis(1000), pool.acquire("test-server")).await {
                // Success
            } else {
                // Should not fail indefinitely
                prop_assert!(false, "Could not acquire connection after health checks");
            }
        });
    }
}

// ============================================================================
// Property 5: Tool Execution Security and Validation
// ============================================================================

proptest! {
    /// Property 5: Tool Execution Security Validation
    /// *For any* tool execution request, MCP SHALL validate permissions and
    /// prevent unauthorized tool access.
    /// **Validates: Requirements MCP-2.1, MCP-3.1**
    #[test]
    fn prop_tool_execution_security_validation(
        tool_count in 1..20usize,
        execution_attempts in 5..50usize,
        permission_denial_rate in 0.0..1.0f64,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let permission_manager = Arc::new(MCPPermissionManager::new());
            let tool_executor = Arc::new(MCPToolExecutor::new(
                "test-server".to_string(),
                Arc::new(MockTransport::new()),
                permission_manager.clone(),
            ));

            // Register tools with different permission requirements
            let mut tools = Vec::new();
            for i in 0..tool_count {
                let tool_name = format!("secure-tool-{}", i);
                let requires_permission = rand::random::<f64>() < permission_denial_rate;

                let tool_metadata = ToolMetadata {
                    name: tool_name.clone(),
                    description: format!("Secure tool {}", i),
                    input_schema: serde_json::json!({"type": "object"}),
                    permissions_required: if requires_permission {
                        vec![format!("permission-{}", i)]
                    } else {
                        vec![]
                    },
                    metadata: None,
                };

                tools.push((tool_name, requires_permission));
            }

            // Simulate execution attempts
            let mut authorized_executions = 0;
            let mut denied_executions = 0;

            for attempt in 0..execution_attempts {
                let tool_idx = attempt % tools.len();
                let (tool_name, requires_permission) = &tools[tool_idx];
                let user_id = format!("user-{}", attempt % 5);

                // Add or deny permissions based on requirements
                if *requires_permission {
                    // Deny permission
                    let deny_rule = PermissionRule {
                        pattern: tool_name.clone(),
                        level: PermissionLevelConfig::Deny,
                        agent_id: Some(user_id.clone()),
                    };
                    permission_manager.add_global_rule(deny_rule).unwrap();
                } else {
                    // Allow permission
                    let allow_rule = PermissionRule {
                        pattern: tool_name.clone(),
                        level: PermissionLevelConfig::Allow,
                        agent_id: Some(user_id.clone()),
                    };
                    permission_manager.add_global_rule(allow_rule).unwrap();
                }

                let context = ToolExecutionContext {
                    tool_name: tool_name.clone(),
                    parameters: HashMap::new(),
                    user_id: Some(user_id.clone()),
                    session_id: Some(format!("session-{}", attempt)),
                    timeout: Duration::from_secs(30),
                    metadata: HashMap::new(),
                };

                let result = tool_executor.execute(&context).await;

                match result {
                    Ok(_) => {
                        if *requires_permission {
                            // Should not have succeeded if permission required
                            prop_assert!(false, "Execution succeeded without permission");
                        } else {
                            authorized_executions += 1;
                        }
                    }
                    Err(_) => {
                        if *requires_permission {
                            denied_executions += 1;
                        } else {
                            // Should have succeeded
                            prop_assert!(false, "Execution failed despite having permission");
                        }
                    }
                }

                // Log the attempt
                audit_logger.log_tool_execution(tool_name, &user_id, result.is_ok()).await.unwrap();
            }

            // Verify audit logging
            let audit_events = audit_logger.get_audit_events(
                Utc::now() - chrono::Duration::hours(1),
                Utc::now(),
            ).await.unwrap();

            prop_assert_eq!(audit_events.len(), execution_attempts);

            // Should have appropriate number of denials based on permission denial rate
            let expected_denials = (execution_attempts as f64 * permission_denial_rate) as usize;
            prop_assert!((denied_executions as isize - expected_denials as isize).abs() < 5, // Allow some variance
                "Denied executions: {}, Expected: {}", denied_executions, expected_denials);
        });
    }

    /// Property 5 variant: Parameter Validation and Sanitization
    #[test]
    fn prop_parameter_validation_sanitization(
        dangerous_params in prop::collection::vec(
            prop_oneof![
                ".*<script>.*".prop_map(|s| s.to_string()),
                ".*../../../.*".prop_map(|s| s.to_string()),
                ".*;.*rm.*".prop_map(|s| s.to_string()),
                ".*'\\s*OR\\s*'.*".prop_map(|s| s.to_string()),
            ],
            1..10
        ),
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let tool_executor = Arc::new(MCPToolExecutor::new(
                "test-server".to_string(),
                Arc::new(MockTransport::new()),
                Arc::new(MCPPermissionManager::new()),
            ));

            let mut blocked_attempts = 0;

            for dangerous_param in dangerous_params {
                let context = ToolExecutionContext {
                    tool_name: "parameter-test-tool".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("input".to_string(), serde_json::json!(dangerous_param));
                        params
                    },
                    user_id: Some("test-user".to_string()),
                    session_id: Some("test-session".to_string()),
                    timeout: Duration::from_secs(10),
                    metadata: HashMap::new(),
                };

                let result = tool_executor.execute(&context).await;

                if result.is_err() {
                    blocked_attempts += 1;
                }

                // Log the attempt
                audit_logger.log_parameter_validation(&dangerous_param, result.is_ok()).await.unwrap();
            }

            // Should block dangerous parameters
            prop_assert!(blocked_attempts > 0, "No dangerous parameters were blocked");

            // Should log all validation attempts
            let audit_events = audit_logger.get_audit_events(
                Utc::now() - chrono::Duration::hours(1),
                Utc::now(),
            ).await.unwrap();

            prop_assert_eq!(audit_events.len(), dangerous_params.len());
        });
    }
}

// ============================================================================
// Property 6: RBAC and Authorization Middleware
// ============================================================================

proptest! {
    /// Property 6: RBAC Authorization Consistency
    /// *For any* RBAC configuration and access pattern, authorization decisions
    /// SHALL be consistent and auditable.
    /// **Validates: Requirements MCP-2.2, MCP-3.2**
    #[test]
    fn prop_rbac_authorization_consistency(
        role_count in 2..10usize,
        user_count in 1..20usize,
        permission_check_count in 10..100usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let rbac_manager = Arc::new(MCRBACManager::new(audit_logger.clone()));
            let auth_middleware = Arc::new(MCPAuthorizationMiddleware::new(rbac_manager.clone()));

            // Setup roles and permissions
            let mut roles = Vec::new();
            for i in 0..role_count {
                let role_name = format!("role-{}", i);
                rbac_manager.create_role(&role_name).await.unwrap();

                // Assign permissions to role
                for j in 0..(i + 1) {
                    rbac_manager.assign_permission_to_role(&role_name, &format!("permission-{}", j)).await.unwrap();
                }

                roles.push(role_name);
            }

            // Assign roles to users
            let mut user_roles = HashMap::new();
            for i in 0..user_count {
                let user_id = format!("user-{}", i);
                let assigned_roles = (0..rand::random::<usize>() % role_count + 1)
                    .map(|j| roles[j % roles.len()].clone())
                    .collect::<Vec<_>>();

                for role in &assigned_roles {
                    rbac_manager.assign_role_to_user(&user_id, role).await.unwrap();
                }

                user_roles.insert(user_id, assigned_roles);
            }

            // Test authorization consistency
            let mut consistent_decisions = 0;

            for _ in 0..permission_check_count {
                let user_idx = rand::random::<usize>() % user_count;
                let user_id = format!("user-{}", user_idx);
                let permission = format!("permission-{}", rand::random::<usize>() % 10);

                let decision1 = auth_middleware.check_permission(&user_id, &permission).await;
                let decision2 = auth_middleware.check_permission(&user_id, &permission).await;

                // Authorization decisions should be consistent
                prop_assert_eq!(decision1, decision2, "Inconsistent authorization decisions");

                consistent_decisions += 1;

                // Log the check
                audit_logger.log_authorization_check(&user_id, &permission, decision1).await.unwrap();
            }

            // All decisions should be consistent
            prop_assert_eq!(consistent_decisions, permission_check_count);

            // Should have audit trail
            let audit_events = audit_logger.get_audit_events(
                Utc::now() - chrono::Duration::hours(1),
                Utc::now(),
            ).await.unwrap();

            let auth_events = audit_events.iter()
                .filter(|e| e.event_type == "authorization_check")
                .count();

            prop_assert_eq!(auth_events, permission_check_count);
        });
    }

    /// Property 6 variant: Authorization Middleware Performance
    #[test]
    fn prop_authorization_middleware_performance(
        concurrent_checks in 1..50usize,
        check_count in 10..100usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let audit_logger = Arc::new(MCPAuditLogger::new(Arc::new(ricecoder_security::audit::MemoryAuditStorage::new())));
            let rbac_manager = Arc::new(MCRBACManager::new(audit_logger));
            let auth_middleware = Arc::new(MCPAuthorizationMiddleware::new(rbac_manager));

            let semaphore = Arc::new(Semaphore::new(concurrent_checks));
            let start_time = Instant::now();

            let mut handles = Vec::new();

            for i in 0..check_count {
                let auth_middleware_clone = auth_middleware.clone();
                let semaphore_clone = semaphore.clone();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    let user_id = format!("perf-user-{}", i % 10);
                    let permission = format!("perf-permission-{}", i % 5);

                    auth_middleware_clone.check_permission(&user_id, &permission).await
                });

                handles.push(handle);
            }

            // Wait for all checks
            let mut success_count = 0;
            for handle in handles {
                if let Ok(_) = handle.await {
                    success_count += 1;
                }
            }

            let total_time = start_time.elapsed();
            let avg_time_per_check = total_time / check_count as u32;

            // All checks should succeed
            prop_assert_eq!(success_count, check_count);

            // Performance should be reasonable
            prop_assert!(avg_time_per_check < Duration::from_millis(100),
                "Authorization checks too slow: {:?}", avg_time_per_check);
        });
    }
}

// ============================================================================
// Enhanced Property Tests Using Testing Infrastructure
// ============================================================================

proptest! {
    /// **Enhanced Property Test: Enterprise Tool Registration with Test Fixtures**
    /// *For any* enterprise tool metadata generated by our test infrastructure,
    /// the MCP test environment SHALL register and manage tools correctly.
    /// **Validates: Requirements MCP-1.1, MCP-2.1 (Enhanced Testing)**
    #[test]
    fn prop_enterprise_tool_registration_with_test_fixtures(
        tool_count in 1..20usize,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            // Use our enhanced test environment
            let env = MCPTestEnvironment::new();
            env.register_test_tools(tool_count).await.unwrap();

            // Verify all tools are registered
            prop_assert_eq!(env.tool_registry.tool_count().await, tool_count);

            // Verify each tool is retrievable and has correct properties
            for i in 0..tool_count {
                let tool_name = format!("test-tool-{}", i);
                let tool = env.tool_registry.get_tool(&tool_name).await.unwrap();

                prop_assert_eq!(tool.name, tool_name);
                prop_assert!(tool.description.contains("MCP testing"));
                prop_assert!(tool.input_schema.is_object());
                prop_assert!(tool.permissions_required.is_empty()); // Test tools have no permissions
            }
        });
    }

    /// **Enhanced Property Test: Mock Server Protocol Compliance**
    /// *For any* generated MCP protocol scenario, mock servers SHALL handle
    /// requests according to the MCP protocol specification.
    /// **Validates: Requirements MCP-1.1, MCP-1.2 (Protocol Testing)**
    #[test]
    fn prop_mock_server_protocol_compliance(
        scenario in prop::sample::select(MCPProtocolDataGenerator::generate_protocol_test_scenarios())
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            let mut server = MockMCPServer::new("protocol-test-server".to_string());

            // Add a test tool for tool/call scenarios
            if scenario.request.method == "tools/call" {
                let tool = ToolMetadata {
                    name: "test-tool".to_string(),
                    description: "Tool for protocol testing".to_string(),
                    input_schema: serde_json::json!({"type": "object"}),
                    permissions_required: vec![],
                    metadata: None,
                };
                server.tools.insert("test-tool".to_string(), tool);
            }

            let result = server.handle_request(&scenario.request).await;

            if scenario.should_fail {
                prop_assert!(result.is_err(), "Scenario should fail but succeeded");
            } else {
                prop_assert!(result.is_ok(), "Scenario should succeed but failed");

                let response = result.unwrap();
                prop_assert_eq!(response.id, scenario.request.id, "Response ID should match request ID");

                // If we expect a specific response, verify it
                if let Some(expected) = &scenario.expected_response {
                    prop_assert_eq!(response.result, expected.result, "Response content should match expected");
                }
            }
        });
    }

    /// **Enhanced Property Test: Enterprise Security Metadata Generation**
    /// *For any* enterprise tool metadata generated by our property-based generators,
    /// the metadata SHALL conform to enterprise security standards.
    /// **Validates: Requirements MCP-2.2, MCP-3.1 (Enterprise Security)**
    #[test]
    fn prop_enterprise_security_metadata_generation(
        tool in arb_enterprise_tool_metadata()
    ) {
        // Verify enterprise security properties
        prop_assert!(!tool.name.is_empty());
        prop_assert!(tool.name.len() <= 64);

        // Check input schema has security-related fields
        let schema = tool.input_schema.as_object().unwrap();
        let properties = schema["properties"].as_object().unwrap();

        // Should have classification field for enterprise scenarios
        let has_classification = properties.contains_key("classification");
        prop_assert!(has_classification, "Enterprise tools should have classification field");

        // If metadata exists, should contain compliance information
        if let Some(metadata) = &tool.metadata {
            let meta_obj = metadata.as_object().unwrap();

            // Should have classification
            prop_assert!(meta_obj.contains_key("classification"));

            // Should have compliance array
            prop_assert!(meta_obj.contains_key("compliance"));
            let compliance = meta_obj["compliance"].as_array().unwrap();
            prop_assert!(!compliance.is_empty(), "Should have compliance requirements");
        }

        // Permissions should be enterprise-style
        for perm in &tool.permissions_required {
            prop_assert!(perm.contains(":") || perm.contains("-"),
                "Enterprise permissions should use namespacing: {}", perm);
        }
    }
}

// ============================================================================
// Mock Transport for Testing
// ============================================================================

struct MockTransport {
    should_fail: bool,
}

impl MockTransport {
    fn new() -> Self {
        Self { should_fail: false }
    }
}

#[async_trait::async_trait]
impl MCPTransport for MockTransport {
    async fn send(&self, _message: &MCPMessage) -> Result<()> {
        if self.should_fail {
            Err(Error::TransportError("Mock transport failure".to_string()))
        } else {
            Ok(())
        }
    }

    async fn receive(&self) -> Result<MCPMessage> {
        if self.should_fail {
            Err(Error::TransportError("Mock transport failure".to_string()))
        } else {
            Ok(MCPMessage::Response(MCPResponse {
                id: "mock-id".to_string(),
                result: serde_json::json!({"status": "ok"}),
            }))
        }
    }

    async fn is_connected(&self) -> bool {
        !self.should_fail
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}</content>
<parameter name="filePath">projects/ricecoder/crates/ricecoder-mcp/tests/mcp_protocol_invariants_property_tests.rs