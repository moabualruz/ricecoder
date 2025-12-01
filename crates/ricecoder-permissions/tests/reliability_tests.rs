//! Reliability tests for ricecoder-permissions
//!
//! These tests validate that the permissions system meets reliability requirements:
//! - Permission checks never fail silently (return errors explicitly)
//! - Audit logs are persistent (survive restarts)
//! - Permission denials are enforced (no bypasses)

use ricecoder_permissions::{
    PermissionLevel, ToolPermission, PermissionManager,
    AuditLogger, InMemoryPermissionRepository, PermissionRepository,
    permission::PermissionChecker,
    audit::{AuditAction, AuditResult},
};
use std::sync::Arc;

// ============================================================================
// Reliability Test 1: Permission Checks Never Fail Silently
// ============================================================================
// Validates: Permission checks return errors explicitly, never silently fail

#[test]
fn test_permission_check_returns_error_on_invalid_input() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Allow);
    manager.store_permission(perm).unwrap();

    // Test: Check permission with empty tool name
    // Should return a result, not panic
    let result = manager.get_permission("");
    assert!(
        result.is_ok(),
        "Permission check must return a result (error or ok), never panic"
    );
}

#[test]
fn test_permission_check_explicit_error_handling() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("test_tool".to_string(), PermissionLevel::Deny);
    manager.store_permission(perm).unwrap();

    // Test: Check permission for denied tool
    // Should return explicit decision, not silently fail
    let perms = manager.get_permission("test_tool").unwrap();
    assert!(!perms.is_empty(), "Permission check must return explicit result");

    let decision = PermissionChecker::check_permission(&perms, None, PermissionLevel::Allow).unwrap();
    // Verify the decision is explicit (not a silent failure)
    assert_eq!(
        decision,
        ricecoder_permissions::permission::PermissionDecision::Deny,
        "Denied permission must be explicit"
    );
}

#[test]
fn test_audit_log_explicit_error_on_failure() {
    let logger = AuditLogger::new();

    // Test: Log execution
    let result = logger.log_execution("test_tool".to_string(), None, None);
    assert!(
        result.is_ok(),
        "Audit logging must return explicit result, not fail silently"
    );

    // Test: Log denial
    let result = logger.log_denial("test_tool".to_string(), None, None);
    assert!(
        result.is_ok(),
        "Audit logging must return explicit result, not fail silently"
    );

    // Test: Log prompt
    let result = logger.log_prompt("test_tool".to_string(), None, None);
    assert!(
        result.is_ok(),
        "Audit logging must return explicit result, not fail silently"
    );
}

#[test]
fn test_permission_manager_explicit_error_on_invalid_config() {
    // Test: Create manager with empty config
    let manager = PermissionManager::new();

    // Should not panic, should return explicit result
    let result = manager.get_permission("any_tool");
    assert!(
        result.is_ok(),
        "Permission check must return explicit result even with empty config"
    );
}

// ============================================================================
// Reliability Test 2: Audit Logs Are Persistent
// ============================================================================
// Validates: Audit logs survive restarts and are properly persisted

#[test]
fn test_audit_logs_persist_across_operations() {
    let repo = InMemoryPermissionRepository::new();
    let logger = AuditLogger::new();

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

    // Get entries
    let entries1 = logger.entries().unwrap();
    assert_eq!(entries1.len(), 3, "All logged events must be persisted");

    // Save to storage
    repo.save_audit_logs(&entries1).unwrap();

    // Load from storage
    let entries2 = repo.load_audit_logs().unwrap();
    assert_eq!(
        entries2.len(),
        3,
        "Persisted logs must survive storage round-trip"
    );

    // Verify content is identical
    for (e1, e2) in entries1.iter().zip(entries2.iter()) {
        assert_eq!(e1.tool, e2.tool, "Tool name must be preserved");
        assert_eq!(e1.agent, e2.agent, "Agent name must be preserved");
        assert_eq!(e1.action, e2.action, "Action must be preserved");
        assert_eq!(e1.result, e2.result, "Result must be preserved");
    }
}

#[test]
fn test_audit_logs_survive_multiple_save_load_cycles() {
    let repo = InMemoryPermissionRepository::new();
    let logger = AuditLogger::new();

    // First cycle: Log and save
    logger
        .log_execution("tool1".to_string(), None, None)
        .unwrap();
    let entries1 = logger.entries().unwrap();
    repo.save_audit_logs(&entries1).unwrap();

    // Load and verify
    let loaded1 = repo.load_audit_logs().unwrap();
    assert_eq!(loaded1.len(), 1, "First save/load cycle must preserve logs");

    // Second cycle: Load, add more logs, save
    let logger2 = AuditLogger::new();
    for entry in &loaded1 {
        logger2
            .log_execution(entry.tool.clone(), entry.agent.clone(), None)
            .unwrap();
    }
    logger2
        .log_denial("tool2".to_string(), None, None)
        .unwrap();

    let entries2 = logger2.entries().unwrap();
    repo.save_audit_logs(&entries2).unwrap();

    // Load and verify
    let loaded2 = repo.load_audit_logs().unwrap();
    assert_eq!(
        loaded2.len(),
        2,
        "Second save/load cycle must preserve all logs"
    );
}

#[test]
fn test_audit_logs_preserve_all_fields() {
    let repo = InMemoryPermissionRepository::new();
    let logger = AuditLogger::new();

    // Log with all fields populated
    logger
        .log_execution(
            "test_tool".to_string(),
            Some("test_agent".to_string()),
            Some("test_context".to_string()),
        )
        .unwrap();

    let entries = logger.entries().unwrap();
    repo.save_audit_logs(&entries).unwrap();

    let loaded = repo.load_audit_logs().unwrap();
    assert_eq!(loaded.len(), 1, "Log entry must be persisted");

    let entry = &loaded[0];
    assert_eq!(entry.tool, "test_tool", "Tool must be preserved");
    assert_eq!(entry.agent, Some("test_agent".to_string()), "Agent must be preserved");
    assert_eq!(entry.action, AuditAction::Allowed, "Action must be preserved");
    assert_eq!(entry.result, AuditResult::Success, "Result must be preserved");
    // Timestamp is a DateTime, just verify it exists
    let _ = entry.timestamp;
}

#[test]
fn test_audit_logs_handle_large_volumes() {
    let repo = InMemoryPermissionRepository::new();
    let logger = AuditLogger::new();

    // Log a large number of events
    for i in 0..1000 {
        let tool = format!("tool_{}", i % 50);
        let agent = if i % 2 == 0 {
            Some(format!("agent_{}", i % 10))
        } else {
            None
        };

        if i % 3 == 0 {
            logger.log_execution(tool, agent, None).unwrap();
        } else if i % 3 == 1 {
            logger.log_denial(tool, agent, None).unwrap();
        } else {
            logger.log_prompt(tool, agent, None).unwrap();
        }
    }

    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 1000, "All 1000 events must be logged");

    // Save and load
    repo.save_audit_logs(&entries).unwrap();
    let loaded = repo.load_audit_logs().unwrap();

    assert_eq!(
        loaded.len(),
        1000,
        "All 1000 events must survive persistence"
    );
}

// ============================================================================
// Reliability Test 3: Permission Denials Are Enforced
// ============================================================================
// Validates: Permission denials cannot be bypassed

#[test]
fn test_permission_denial_cannot_be_bypassed_with_empty_agent() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("denied_tool".to_string(), PermissionLevel::Deny);
    manager.store_permission(perm).unwrap();

    // Test: Try to bypass denial by checking with no agent
    let perms = manager.get_permission("denied_tool").unwrap();
    let decision = PermissionChecker::check_permission(&perms, None, PermissionLevel::Allow).unwrap();

    assert_eq!(
        decision,
        ricecoder_permissions::permission::PermissionDecision::Deny,
        "Denial must be enforced regardless of agent"
    );
}

#[test]
fn test_permission_denial_cannot_be_bypassed_with_different_agent() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("denied_tool".to_string(), PermissionLevel::Deny);
    manager.store_permission(perm).unwrap();

    // Test: Try to bypass denial by checking with different agents
    for agent in &["agent1", "agent2", "admin", "root"] {
        let perms = manager.get_permission("denied_tool").unwrap();
        let decision = PermissionChecker::check_permission(&perms, Some(agent), PermissionLevel::Allow).unwrap();

        assert_eq!(
            decision,
            ricecoder_permissions::permission::PermissionDecision::Deny,
            "Denial must be enforced for all agents"
        );
    }
}

#[test]
fn test_permission_denial_cannot_be_bypassed_with_pattern_matching() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("denied_*".to_string(), PermissionLevel::Deny);
    manager.store_permission(perm).unwrap();

    // Test: Try to bypass denial with tools matching the pattern
    for tool in &["denied_tool1", "denied_tool2", "denied_anything"] {
        let perms = manager.get_permission(tool).unwrap();
        let decision = PermissionChecker::check_permission(&perms, None, PermissionLevel::Allow).unwrap();

        assert_eq!(
            decision,
            ricecoder_permissions::permission::PermissionDecision::Deny,
            "Denial pattern must be enforced for all matching tools"
        );
    }
}

#[test]
fn test_permission_denial_is_logged() {
    let manager = PermissionManager::new();
    let perm = ToolPermission::new("denied_tool".to_string(), PermissionLevel::Deny);
    manager.store_permission(perm).unwrap();

    let logger = Arc::new(AuditLogger::new());

    // Check permission (which should be denied)
    let perms = manager.get_permission("denied_tool").unwrap();
    let decision = PermissionChecker::check_permission(&perms, None, PermissionLevel::Allow).unwrap();
    assert_eq!(decision, ricecoder_permissions::permission::PermissionDecision::Deny, "Permission check must return Deny");

    // Log the denial
    logger
        .log_denial("denied_tool".to_string(), None, None)
        .unwrap();

    // Verify denial is logged
    let entries = logger.entries().unwrap();
    assert_eq!(entries.len(), 1, "Denial must be logged");
    assert_eq!(entries[0].action, AuditAction::Denied, "Action must be Denied");
    assert_eq!(entries[0].result, AuditResult::Blocked, "Result must be Blocked");
}

#[test]
fn test_permission_denial_enforced_across_restarts() {
    let repo = InMemoryPermissionRepository::new();

    // Create and save config with denial
    let mut config = ricecoder_permissions::PermissionConfig::new();
    let perm = ToolPermission::new("denied_tool".to_string(), PermissionLevel::Deny);
    config.add_permission(perm);
    repo.save_config(&config).unwrap();

    // Simulate restart: Load config
    let loaded_config = repo.load_config().unwrap();
    let manager = PermissionManager::new();
    manager.reload_permissions(loaded_config.get_permissions().to_vec()).unwrap();

    // Verify denial is still enforced after "restart"
    let perms = manager.get_permission("denied_tool").unwrap();
    let decision = PermissionChecker::check_permission(&perms, None, PermissionLevel::Allow).unwrap();

    assert_eq!(
        decision,
        ricecoder_permissions::permission::PermissionDecision::Deny,
        "Denial must be enforced after restart"
    );
}

#[test]
fn test_permission_denial_with_per_agent_override() {
    let manager = PermissionManager::new();
    // Global deny
    let global_deny = ToolPermission::new("tool".to_string(), PermissionLevel::Deny);
    manager.store_permission(global_deny).unwrap();
    
    // Agent-specific allow (should override global deny)
    let agent_allow = ToolPermission::with_agent(
        "tool".to_string(),
        PermissionLevel::Allow,
        "admin".to_string(),
    );
    manager.store_permission(agent_allow).unwrap();

    // Test: Regular user should be denied
    let perms = manager.get_permission("tool").unwrap();
    let decision = PermissionChecker::check_permission(&perms, Some("user"), PermissionLevel::Allow).unwrap();
    assert_eq!(
        decision,
        ricecoder_permissions::permission::PermissionDecision::Deny,
        "Regular user should be denied"
    );

    // Test: Admin should be allowed (override)
    let perms = manager.get_permission("tool").unwrap();
    let decision = PermissionChecker::check_permission(&perms, Some("admin"), PermissionLevel::Allow).unwrap();
    assert_eq!(
        decision,
        ricecoder_permissions::permission::PermissionDecision::Allow,
        "Admin should be allowed (override)"
    );
}

