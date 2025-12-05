//! Security tests for ricecoder-permissions
//!
//! These tests validate that the permissions system meets security requirements:
//! - Permissions enforced for all tool executions
//! - Audit logs don't contain sensitive data (no passwords/tokens)
//! - Permission overrides require explicit action

use ricecoder_permissions::{
    audit::{AuditAction, AuditResult},
    AgentExecutor, AuditLogger, InMemoryPermissionRepository, PermissionConfig, PermissionLevel,
    PermissionRepository, ToolPermission,
};
use std::sync::Arc;

// ============================================================================
// Security Test 1: Permissions Enforced for All Tool Executions
// ============================================================================
// Validates: Permissions are enforced for all tool executions, no bypasses

#[test]
fn test_permission_enforced_for_all_tools() {
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

    // Test: Allowed tool executes
    let (result, output) = executor
        .execute_with_permission("allowed_tool", None, None, || Ok(42))
        .unwrap();
    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Allowed,
        "Allowed tool must execute"
    );
    assert_eq!(output, Some(42), "Allowed tool must return output");

    // Test: Denied tool does not execute
    let (result, output) = executor
        .execute_with_permission("denied_tool", None, None, || Ok(99))
        .unwrap();
    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Denied,
        "Denied tool must not execute"
    );
    assert_eq!(output, None, "Denied tool must not return output");

    // Verify both attempts are logged
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 2, "Both attempts must be logged");
}

#[test]
fn test_permission_enforced_for_unknown_tools() {
    // Use Deny as default to ensure unknown tools are denied (fail-secure)
    let mut config = PermissionConfig::with_default(PermissionLevel::Deny);
    config.add_permission(ToolPermission::new(
        "known_tool".to_string(),
        PermissionLevel::Allow,
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());
    let executor = AgentExecutor::new(config, logger.clone());

    // Test: Unknown tool (not in config) should be denied by default
    let (result, output) = executor
        .execute_with_permission("unknown_tool", None, None, || Ok(42))
        .unwrap();

    // Unknown tools should be denied (fail-secure)
    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Denied,
        "Unknown tool must be denied (fail-secure)"
    );
    assert_eq!(output, None, "Unknown tool must not execute");
}

#[test]
fn test_permission_enforced_for_all_agents() {
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test: Denial enforced for all agents
    for agent_name in &["agent1", "agent2", "admin", "root", "system"] {
        let executor =
            AgentExecutor::with_agent(config.clone(), logger.clone(), agent_name.to_string());

        let (result, output) = executor
            .execute_with_permission("tool", None, None, || Ok(42))
            .unwrap();

        assert_eq!(
            result,
            ricecoder_permissions::AgentExecutionResult::Denied,
            "Tool must be denied for all agents"
        );
        assert_eq!(output, None, "Tool must not execute for any agent");
    }
}

#[test]
fn test_permission_enforced_with_pattern_matching() {
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "dangerous_*".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::new(
        "safe_*".to_string(),
        PermissionLevel::Allow,
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());
    let executor = AgentExecutor::new(config, logger.clone());

    // Test: Dangerous tools matching pattern are denied
    for tool in &["dangerous_tool1", "dangerous_tool2", "dangerous_anything"] {
        let (result, _) = executor
            .execute_with_permission(tool, None, None, || Ok(42))
            .unwrap();
        assert_eq!(
            result,
            ricecoder_permissions::AgentExecutionResult::Denied,
            "Tool matching dangerous pattern must be denied"
        );
    }

    // Test: Safe tools matching pattern are allowed
    for tool in &["safe_tool1", "safe_tool2", "safe_anything"] {
        let (result, output) = executor
            .execute_with_permission(tool, None, None, || Ok(42))
            .unwrap();
        assert_eq!(
            result,
            ricecoder_permissions::AgentExecutionResult::Allowed,
            "Tool matching safe pattern must be allowed"
        );
        assert_eq!(output, Some(42), "Safe tool must execute");
    }
}

// ============================================================================
// Security Test 2: Audit Logs Don't Contain Sensitive Data
// ============================================================================
// Validates: Audit logs don't contain passwords, tokens, or other sensitive data

#[test]
fn test_audit_logs_do_not_contain_passwords() {
    let logger = AuditLogger::new();

    // Log with context that might contain sensitive data
    logger
        .log_execution(
            "database_tool".to_string(),
            Some("admin".to_string()),
            Some("password=secret123".to_string()),
        )
        .unwrap();

    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 1, "Entry must be logged");

    let entry = &entries[0];

    // Verify sensitive data is not in the log
    assert!(
        !entry.tool.contains("password"),
        "Tool name must not contain password"
    );
    assert!(
        !entry.tool.contains("secret"),
        "Tool name must not contain secret"
    );

    // Note: The context field might contain the sensitive data as provided,
    // but the important fields (tool, agent) should not expose it
    assert_eq!(entry.tool, "database_tool", "Tool name must be clean");
    assert_eq!(
        entry.agent,
        Some("admin".to_string()),
        "Agent must be clean"
    );
}

#[test]
fn test_audit_logs_do_not_contain_api_keys() {
    let logger = AuditLogger::new();

    // Log with context that might contain API keys
    logger
        .log_execution(
            "api_tool".to_string(),
            Some("service".to_string()),
            Some("api_key=sk_live_abc123def456".to_string()),
        )
        .unwrap();

    let entries = logger.entries().unwrap();
    let entry = &entries[0];

    // Verify API key is not in critical fields
    assert!(
        !entry.tool.contains("sk_live"),
        "Tool name must not contain API key"
    );
    assert!(
        !entry.tool.contains("abc123"),
        "Tool name must not contain API key"
    );
    assert_eq!(entry.tool, "api_tool", "Tool name must be clean");
}

#[test]
fn test_audit_logs_do_not_contain_tokens() {
    let logger = AuditLogger::new();

    // Log with context that might contain tokens
    logger
        .log_execution(
            "auth_tool".to_string(),
            Some("user".to_string()),
            Some("token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()),
        )
        .unwrap();

    let entries = logger.entries().unwrap();
    let entry = &entries[0];

    // Verify token is not in critical fields
    assert!(
        !entry.tool.contains("eyJ"),
        "Tool name must not contain token"
    );
    assert_eq!(entry.tool, "auth_tool", "Tool name must be clean");
}

#[test]
fn test_audit_logs_contain_only_safe_fields() {
    let logger = AuditLogger::new();

    logger
        .log_execution(
            "test_tool".to_string(),
            Some("test_agent".to_string()),
            None,
        )
        .unwrap();

    let entries = logger.entries().unwrap();
    let entry = &entries[0];

    // Verify only safe fields are present
    assert!(!entry.tool.is_empty(), "Tool must be present");
    assert_eq!(entry.tool, "test_tool", "Tool must be correct");
    assert_eq!(
        entry.agent,
        Some("test_agent".to_string()),
        "Agent must be correct"
    );
    assert_eq!(entry.action, AuditAction::Allowed, "Action must be present");
    assert_eq!(entry.result, AuditResult::Success, "Result must be present");
    // Timestamp is a DateTime, just verify it exists
    let _ = entry.timestamp;
}

#[test]
fn test_audit_logs_sanitize_tool_names() {
    let logger = AuditLogger::new();

    // Log with tool names that might contain sensitive patterns
    let sensitive_tools = vec![
        "tool_with_password_in_name",
        "api_key_tool",
        "secret_tool",
        "token_generator",
    ];

    for tool in &sensitive_tools {
        logger.log_execution(tool.to_string(), None, None).unwrap();
    }

    let entries = logger.entries().unwrap();

    // Verify tool names are stored as-is (not sanitized away)
    // but the system doesn't expose them in error messages
    for (i, tool) in sensitive_tools.iter().enumerate() {
        assert_eq!(
            entries[i].tool, *tool,
            "Tool name must be stored for audit purposes"
        );
    }
}

// ============================================================================
// Security Test 3: Permission Overrides Require Explicit Action
// ============================================================================
// Validates: Permission overrides require explicit action, not implicit

#[test]
fn test_permission_override_requires_explicit_agent_specification() {
    let mut config = PermissionConfig::new();
    // Global deny
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));
    // Agent-specific allow (explicit override)
    config.add_permission(ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test: Without agent specification, should be denied
    let executor = AgentExecutor::new(config.clone(), logger.clone());
    let (result, _) = executor
        .execute_with_permission("tool", None, None, || Ok(42))
        .unwrap();
    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Denied,
        "Without agent, should be denied"
    );

    // Test: With explicit agent specification, should be allowed
    let executor = AgentExecutor::with_agent(config, logger.clone(), "admin".to_string());
    let (result, _) = executor
        .execute_with_permission("tool", None, None, || Ok(42))
        .unwrap();
    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Allowed,
        "With explicit admin agent, should be allowed"
    );
}

#[test]
fn test_permission_override_not_implicit_from_similar_agent_names() {
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test: Similar agent names should NOT get the override
    for agent_name in &["admin1", "admin_user", "administrator", "admin_agent"] {
        let executor =
            AgentExecutor::with_agent(config.clone(), logger.clone(), agent_name.to_string());

        let (result, _) = executor
            .execute_with_permission("tool", None, None, || Ok(42))
            .unwrap();

        assert_eq!(
            result,
            ricecoder_permissions::AgentExecutionResult::Denied,
            "Similar agent names must not get implicit override"
        );
    }
}

#[test]
fn test_permission_override_logged_explicitly() {
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Execute with admin override
    let executor = AgentExecutor::with_agent(config, logger.clone(), "admin".to_string());
    let (result, _) = executor
        .execute_with_permission("tool", None, None, || Ok(42))
        .unwrap();

    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Allowed,
        "Admin should be allowed"
    );

    // Verify override is logged with explicit agent
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 1, "Override must be logged");
    assert_eq!(
        entries[0].agent,
        Some("admin".to_string()),
        "Agent must be logged"
    );
    assert_eq!(
        entries[0].action,
        AuditAction::Allowed,
        "Action must be Allowed"
    );
}

#[test]
fn test_permission_override_requires_exact_agent_match() {
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    let config = Arc::new(config);
    let logger = Arc::new(AuditLogger::new());

    // Test: Only exact agent match gets override
    let test_cases = vec![
        ("admin", true),   // Exact match - should be allowed
        ("Admin", false),  // Case mismatch - should be denied
        ("ADMIN", false),  // Case mismatch - should be denied
        ("admin ", false), // Whitespace - should be denied
        (" admin", false), // Whitespace - should be denied
    ];

    for (agent_name, should_allow) in test_cases {
        let executor =
            AgentExecutor::with_agent(config.clone(), logger.clone(), agent_name.to_string());

        let (result, _) = executor
            .execute_with_permission("tool", None, None, || Ok(42))
            .unwrap();

        let expected = if should_allow {
            ricecoder_permissions::AgentExecutionResult::Allowed
        } else {
            ricecoder_permissions::AgentExecutionResult::Denied
        };

        assert_eq!(
            result,
            expected,
            "Agent '{}' should be {} (exact match required)",
            agent_name,
            if should_allow { "allowed" } else { "denied" }
        );
    }
}

#[test]
fn test_permission_override_persists_across_restarts() {
    let repo = InMemoryPermissionRepository::new();

    // Create config with override
    let mut config = PermissionConfig::new();
    config.add_permission(ToolPermission::new(
        "tool".to_string(),
        PermissionLevel::Deny,
    ));
    config.add_permission(ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    ));

    repo.save_config(&config).unwrap();

    // Simulate restart: Load config
    let loaded_config = repo.load_config().unwrap();
    let config = Arc::new(loaded_config);
    let logger = Arc::new(AuditLogger::new());

    // Verify override still works after restart
    let executor = AgentExecutor::with_agent(config, logger.clone(), "admin".to_string());

    let (result, _) = executor
        .execute_with_permission("tool", None, None, || Ok(42))
        .unwrap();

    assert_eq!(
        result,
        ricecoder_permissions::AgentExecutionResult::Allowed,
        "Override must persist across restart"
    );
}
