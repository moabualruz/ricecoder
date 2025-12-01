//! Integration tests for permissions system with agent execution and storage

use ricecoder_permissions::{
    AgentExecutor, AgentExecutionResult, AuditLogger, InMemoryPermissionRepository,
    PermissionConfig, PermissionLevel, PermissionRepository, ToolPermission,
};
use std::sync::Arc;

#[test]
fn test_agent_execution_with_permission_checking() {
    // Setup
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "allowed_tool".to_string(),
        PermissionLevel::Allow,
    ));
    config.add_permission(ToolPermission::new(
        "denied_tool".to_string(),
        PermissionLevel::Deny,
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());
    let executor = AgentExecutor::new(config, logger.clone());

    // Test allowed tool execution
    let (result, output) = executor
        .execute_with_permission("allowed_tool", None, None, || Ok(42))
        .unwrap();

    assert_eq!(result, AgentExecutionResult::Allowed);
    assert_eq!(output, Some(42));

    // Test denied tool execution
    let (result, output) = executor
        .execute_with_permission("denied_tool", None, None, || Ok(99))
        .unwrap();

    assert_eq!(result, AgentExecutionResult::Denied);
    assert_eq!(output, None);

    // Verify audit logs
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_agent_execution_with_per_agent_override() {
    // Setup
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "test_tool".to_string(),
        PermissionLevel::Allow,
    ));
    config.add_permission(ToolPermission::with_agent(
        "test_tool".to_string(),
        PermissionLevel::Deny,
        "restricted_agent".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test with unrestricted agent
    let executor = AgentExecutor::with_agent(
        config.clone(),
        logger.clone(),
        "normal_agent".to_string(),
    );
    let (result, _) = executor
        .execute_with_permission("test_tool", None, None, || Ok(42))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Allowed);

    // Test with restricted agent
    let executor = AgentExecutor::with_agent(
        config,
        logger.clone(),
        "restricted_agent".to_string(),
    );
    let (result, _) = executor
        .execute_with_permission("test_tool", None, None, || Ok(42))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Denied);

    // Verify audit logs
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_storage_integration_load_and_save_config() {
    // Setup
    let repo = InMemoryPermissionRepository::new();

    // Create and save config
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "test_tool".to_string(),
        PermissionLevel::Allow,
    ));
    config.add_permission(ToolPermission::with_agent(
        "restricted_tool".to_string(),
        PermissionLevel::Deny,
        "agent1".to_string(),
    ));

    repo.save_config(&config).unwrap();

    // Load config
    let loaded_config = repo.load_config().unwrap();

    // Verify
    assert_eq!(loaded_config.get_permissions().len(), 2);
    assert_eq!(loaded_config.get_permissions()[0].tool_pattern, "test_tool");
    assert_eq!(loaded_config.get_permissions()[1].tool_pattern, "restricted_tool");
    assert_eq!(
        loaded_config.get_permissions()[1].agent,
        Some("agent1".to_string())
    );
}

#[test]
fn test_storage_integration_audit_logs() {
    // Setup
    let repo = InMemoryPermissionRepository::new();
    let logger = Arc::new(AuditLogger::new());

    // Log some events
    logger
        .log_execution("tool1".to_string(), Some("agent1".to_string()), None)
        .unwrap();
    logger
        .log_denial("tool2".to_string(), Some("agent1".to_string()), None)
        .unwrap();
    logger
        .log_prompt("tool3".to_string(), Some("agent2".to_string()), None)
        .unwrap();

    // Get logs from logger
    let entries = logger.entries().unwrap();

    // Save to storage
    repo.save_audit_logs(&entries).unwrap();

    // Load from storage
    let loaded_logs = repo.load_audit_logs().unwrap();

    // Verify
    assert_eq!(loaded_logs.len(), 3);
    assert_eq!(loaded_logs[0].tool, "tool1");
    assert_eq!(loaded_logs[0].agent, Some("agent1".to_string()));
    assert_eq!(loaded_logs[1].tool, "tool2");
    assert_eq!(loaded_logs[2].tool, "tool3");
    assert_eq!(loaded_logs[2].agent, Some("agent2".to_string()));
}

#[test]
fn test_full_workflow_with_storage() {
    // Setup storage
    let repo = InMemoryPermissionRepository::new();

    // Create initial config
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "safe_tool".to_string(),
        PermissionLevel::Allow,
    ));
    config.add_permission(ToolPermission::new(
        "dangerous_tool".to_string(),
        PermissionLevel::Deny,
    ));

    // Save config
    repo.save_config(&config).unwrap();

    // Load config and create executor
    let loaded_config = repo.load_config().unwrap();
    let config = Arc::new(loaded_config);
    let logger = Arc::new(AuditLogger::new());
    let executor = AgentExecutor::with_agent(
        config,
        logger.clone(),
        "test_agent".to_string(),
    );

    // Execute tools
    let (result1, _) = executor
        .execute_with_permission("safe_tool", None, None, || Ok(42))
        .unwrap();
    let (result2, _) = executor
        .execute_with_permission("dangerous_tool", None, None, || Ok(99))
        .unwrap();

    // Verify execution results
    assert_eq!(result1, AgentExecutionResult::Allowed);
    assert_eq!(result2, AgentExecutionResult::Denied);

    // Get audit logs
    let entries = logger.entries().unwrap();

    // Save audit logs to storage
    repo.save_audit_logs(&entries).unwrap();

    // Load audit logs from storage
    let loaded_logs = repo.load_audit_logs().unwrap();

    // Verify audit logs
    assert_eq!(loaded_logs.len(), 2);
    assert_eq!(loaded_logs[0].tool, "safe_tool");
    assert_eq!(loaded_logs[1].tool, "dangerous_tool");
    assert_eq!(loaded_logs[0].agent, Some("test_agent".to_string()));
    assert_eq!(loaded_logs[1].agent, Some("test_agent".to_string()));
}

#[test]
fn test_permission_checking_in_agent_context() {
    // Setup
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "public_tool".to_string(),
        PermissionLevel::Allow,
    ));
    config.add_permission(ToolPermission::new(
        "admin_tool".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::with_agent(
        "admin_tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test as regular user
    let user_executor = AgentExecutor::with_agent(
        config.clone(),
        logger.clone(),
        "user".to_string(),
    );

    let (result, _) = user_executor
        .execute_with_permission("public_tool", None, None, || Ok(1))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Allowed);

    let (result, _) = user_executor
        .execute_with_permission("admin_tool", None, None, || Ok(2))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Denied);

    // Test as admin
    let admin_executor = AgentExecutor::with_agent(
        config,
        logger.clone(),
        "admin".to_string(),
    );

    let (result, _) = admin_executor
        .execute_with_permission("public_tool", None, None, || Ok(3))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Allowed);

    // For admin_tool, there's both a global deny and an agent-specific allow
    // Agent-specific permissions override global, so it should be allowed
    let (result, _) = admin_executor
        .execute_with_permission("admin_tool", None, None, || Ok(4))
        .unwrap();
    assert_eq!(result, AgentExecutionResult::Allowed);

    // Verify audit logs
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 4);
}

#[test]
fn test_storage_persistence_across_operations() {
    // Setup
    let repo = InMemoryPermissionRepository::new();

    // Initial config
    let mut config1 = PermissionConfig::new();
    config1.add_permission(ToolPermission::new(
        "tool1".to_string(),
        PermissionLevel::Allow,
    ));
    repo.save_config(&config1).unwrap();

    // Load and modify
    let mut config2 = repo.load_config().unwrap();
    config2.add_permission(ToolPermission::new(
        "tool2".to_string(),
        PermissionLevel::Deny,
    ));
    repo.save_config(&config2).unwrap();

    // Load again and verify
    let config3 = repo.load_config().unwrap();
    assert_eq!(config3.get_permissions().len(), 2);
    assert_eq!(config3.get_permissions()[0].tool_pattern, "tool1");
    assert_eq!(config3.get_permissions()[1].tool_pattern, "tool2");
}
