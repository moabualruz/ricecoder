//! Property-based tests for ricecoder-permissions
//!
//! These tests verify correctness properties that should hold across all inputs.

use proptest::prelude::*;
use ricecoder_permissions::{
    audit::{AuditAction, AuditResult},
    permission::PermissionChecker,
    AuditLogger, GlobMatcher, PermissionLevel, ToolPermission,
};

// ============================================================================
// Property 1: Permission Enforcement
// ============================================================================
// **Feature: ricecoder-permissions, Property 1: Permission Enforcement**
// For any tool with "deny" permission, the system SHALL not execute the tool.
// **Validates: Requirements 1.3, 2.2**

/// Strategy for generating tool names
fn tool_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z_][a-z0-9_]{0,20}".prop_map(|s| s.to_string())
}

/// Strategy for generating agent names
fn agent_name_strategy() -> impl Strategy<Value = String> {
    r"[a-z_][a-z0-9_]{0,10}".prop_map(|s| s.to_string())
}

proptest! {
    /// Property 1: Permission Enforcement
    /// For any tool with "deny" permission, the system SHALL not execute the tool.
    #[test]
    fn prop_permission_enforcement_deny_blocks_execution(
        tool_name in tool_name_strategy(),
        agent_name in prop::option::of(agent_name_strategy()),
    ) {
        // Create a permission that denies access to the tool
        let permission = if let Some(agent) = &agent_name {
            ToolPermission::with_agent(
                tool_name.clone(),
                PermissionLevel::Deny,
                agent.clone(),
            )
        } else {
            ToolPermission::new(tool_name.clone(), PermissionLevel::Deny)
        };

        // Check permission for the tool
        let decision = PermissionChecker::check_permission(
            &[permission],
            agent_name.as_deref(),
            PermissionLevel::Allow,
        ).unwrap();

        // Property: Deny permission must result in Deny decision
        prop_assert_eq!(
            decision,
            ricecoder_permissions::permission::PermissionDecision::Deny,
            "Tool with deny permission should not be executable"
        );
    }

    /// Property 1: Permission Enforcement (multiple permissions)
    /// For any tool with "deny" permission among multiple permissions,
    /// the system SHALL not execute the tool (most restrictive wins).
    #[test]
    fn prop_permission_enforcement_deny_with_multiple_permissions(
        tool_name in tool_name_strategy(),
        agent_name in prop::option::of(agent_name_strategy()),
    ) {
        // Create multiple permissions with at least one deny
        let permissions = vec![
            ToolPermission::new(tool_name.clone(), PermissionLevel::Allow),
            ToolPermission::new(tool_name.clone(), PermissionLevel::Ask),
            ToolPermission::new(tool_name.clone(), PermissionLevel::Deny),
        ];

        // Check permission for the tool
        let decision = PermissionChecker::check_permission(
            &permissions,
            agent_name.as_deref(),
            PermissionLevel::Allow,
        ).unwrap();

        // Property: Most restrictive (Deny) must be selected
        prop_assert_eq!(
            decision,
            ricecoder_permissions::permission::PermissionDecision::Deny,
            "Most restrictive permission (Deny) should be selected"
        );
    }

    /// Property 1: Permission Enforcement (per-agent override)
    /// For any tool with per-agent deny permission, the system SHALL not execute
    /// the tool for that agent, even if global permission is allow.
    #[test]
    fn prop_permission_enforcement_agent_override_deny(
        tool_name in tool_name_strategy(),
        agent_name in agent_name_strategy(),
    ) {
        // Create global allow and agent-specific deny
        let global_perm = ToolPermission::new(tool_name.clone(), PermissionLevel::Allow);
        let agent_perm = ToolPermission::with_agent(
            tool_name.clone(),
            PermissionLevel::Deny,
            agent_name.clone(),
        );

        // Check permission for the specific agent
        let decision = PermissionChecker::check_permission(
            &[global_perm, agent_perm],
            Some(&agent_name),
            PermissionLevel::Allow,
        ).unwrap();

        // Property: Agent-specific deny must override global allow
        prop_assert_eq!(
            decision,
            ricecoder_permissions::permission::checker::PermissionDecision::Deny,
            "Agent-specific deny should override global allow"
        );
    }
}

// ============================================================================
// Property 2: Glob Pattern Matching
// ============================================================================
// **Feature: ricecoder-permissions, Property 2: Glob Pattern Matching**
// For any glob pattern, matching SHALL be consistent and correct.
// **Validates: Requirements 3.1, 3.2**

/// Strategy for generating valid glob patterns
fn glob_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Exact matches
        r"[a-z_][a-z0-9_]{0,10}".prop_map(|s| s.to_string()),
        // Wildcard patterns
        r"[a-z_][a-z0-9_]{0,5}\*".prop_map(|s| s.to_string()),
        r"\*[a-z_][a-z0-9_]{0,5}".prop_map(|s| s.to_string()),
        // Question mark patterns
        r"[a-z_][a-z0-9_]{0,5}\?".prop_map(|s| s.to_string()),
        // Universal wildcard
        Just("*".to_string()),
    ]
}

proptest! {
    /// Property 2: Glob Pattern Matching - Determinism
    /// For any glob pattern and tool name, matching SHALL be deterministic
    /// (same input always produces same output).
    #[test]
    fn prop_glob_pattern_matching_deterministic(
        pattern in glob_pattern_strategy(),
        tool_name in tool_name_strategy(),
    ) {
        let matcher = GlobMatcher::new();

        // Match the same pattern and tool name multiple times
        let result1 = matcher.match_pattern(&pattern, &tool_name);
        let result2 = matcher.match_pattern(&pattern, &tool_name);
        let result3 = matcher.match_pattern(&pattern, &tool_name);

        // Property: All results must be identical
        prop_assert_eq!(
            result1, result2,
            "Pattern matching must be deterministic (first vs second)"
        );
        prop_assert_eq!(
            result2, result3,
            "Pattern matching must be deterministic (second vs third)"
        );
    }

    /// Property 2: Glob Pattern Matching - Wildcard matches all
    /// For the universal wildcard pattern "*", it SHALL match any tool name.
    #[test]
    fn prop_glob_pattern_wildcard_matches_all(
        tool_name in tool_name_strategy(),
    ) {
        let matcher = GlobMatcher::new();

        // Universal wildcard should match any tool name
        let result = matcher.match_pattern("*", &tool_name);

        // Property: Wildcard must match any tool name
        prop_assert!(
            result,
            "Universal wildcard '*' should match any tool name: {}",
            tool_name
        );
    }

    /// Property 2: Glob Pattern Matching - Exact match
    /// For an exact pattern (no wildcards), it SHALL only match identical tool names.
    #[test]
    fn prop_glob_pattern_exact_match(
        tool_name in tool_name_strategy(),
    ) {
        let matcher = GlobMatcher::new();

        // Exact pattern should only match identical tool name
        let result = matcher.match_pattern(&tool_name, &tool_name);

        // Property: Exact pattern must match identical tool name
        prop_assert!(
            result,
            "Exact pattern should match identical tool name"
        );
    }

    /// Property 2: Glob Pattern Matching - Exact mismatch
    /// For an exact pattern, it SHALL not match different tool names.
    #[test]
    fn prop_glob_pattern_exact_mismatch(
        tool_name1 in tool_name_strategy(),
        tool_name2 in tool_name_strategy(),
    ) {
        prop_assume!(tool_name1 != tool_name2);

        let matcher = GlobMatcher::new();

        // Exact pattern should not match different tool name
        let result = matcher.match_pattern(&tool_name1, &tool_name2);

        // Property: Exact pattern must not match different tool name
        prop_assert!(
            !result,
            "Exact pattern should not match different tool name"
        );
    }

    /// Property 2: Glob Pattern Matching - Conflict resolution
    /// For multiple matching patterns, the most specific pattern SHALL be selected.
    #[test]
    fn prop_glob_pattern_conflict_resolution(
        tool_name in tool_name_strategy(),
    ) {
        let matcher = GlobMatcher::new();

        // Create patterns with different specificity
        let pattern1 = "*".to_string();
        let pattern2 = format!("{}*", &tool_name[0..1.min(tool_name.len())]);
        let patterns = vec![
            pattern1.as_str(),      // Least specific
            pattern2.as_str(),      // More specific
            tool_name.as_str(),     // Most specific (exact match)
        ];

        // Resolve conflicts
        let best_match = matcher.resolve_conflicts(&patterns, &tool_name);

        // Property: Most specific pattern (exact match) should be selected
        // The exact match is at index 2
        if matcher.match_pattern(&tool_name, &tool_name) {
            prop_assert_eq!(
                best_match,
                Some(2),
                "Most specific pattern (exact match) should be selected"
            );
        }
    }
}

// ============================================================================
// Property 3: Audit Log Completeness
// ============================================================================
// **Feature: ricecoder-permissions, Property 3: Audit Log Completeness**
// For any tool execution, audit log SHALL contain a record.
// **Validates: Requirements 5.1, 5.2**

proptest! {
    /// Property 3: Audit Log Completeness - Execution logging
    /// For any tool execution, the audit log SHALL contain a record.
    #[test]
    fn prop_audit_log_completeness_execution(
        tool_name in tool_name_strategy(),
        agent_name in prop::option::of(agent_name_strategy()),
    ) {
        let logger = AuditLogger::new();

        // Log a tool execution
        let result = logger.log_execution(tool_name.clone(), agent_name.clone(), None);

        // Property: Logging must succeed
        prop_assert!(result.is_ok(), "Logging execution must succeed");

        // Property: Log must contain exactly one entry
        let entries = logger.entries().unwrap();
        prop_assert_eq!(
            entries.len(),
            1,
            "Log must contain exactly one entry after logging execution"
        );

        // Property: Log entry must have correct tool name
        prop_assert_eq!(
            &entries[0].tool,
            &tool_name,
            "Log entry must have correct tool name"
        );

        // Property: Log entry must have correct agent (if provided)
        if let Some(agent) = &agent_name {
            prop_assert_eq!(
                &entries[0].agent,
                &Some(agent.clone()),
                "Log entry must have correct agent"
            );
        }

        // Property: Log entry must have correct action
        prop_assert_eq!(
            entries[0].action,
            AuditAction::Allowed,
            "Log entry must have Allowed action for execution"
        );

        // Property: Log entry must have correct result
        prop_assert_eq!(
            entries[0].result,
            AuditResult::Success,
            "Log entry must have Success result for execution"
        );
    }

    /// Property 3: Audit Log Completeness - Denial logging
    /// For any tool denial, the audit log SHALL contain a record.
    #[test]
    fn prop_audit_log_completeness_denial(
        tool_name in tool_name_strategy(),
        agent_name in prop::option::of(agent_name_strategy()),
    ) {
        let logger = AuditLogger::new();

        // Log a tool denial
        let result = logger.log_denial(tool_name.clone(), agent_name.clone(), None);

        // Property: Logging must succeed
        prop_assert!(result.is_ok(), "Logging denial must succeed");

        // Property: Log must contain exactly one entry
        let entries = logger.entries().unwrap();
        prop_assert_eq!(
            entries.len(),
            1,
            "Log must contain exactly one entry after logging denial"
        );

        // Property: Log entry must have correct tool name
        prop_assert_eq!(
            &entries[0].tool,
            &tool_name,
            "Log entry must have correct tool name"
        );

        // Property: Log entry must have correct action
        prop_assert_eq!(
            entries[0].action,
            AuditAction::Denied,
            "Log entry must have Denied action for denial"
        );

        // Property: Log entry must have correct result
        prop_assert_eq!(
            entries[0].result,
            AuditResult::Blocked,
            "Log entry must have Blocked result for denial"
        );
    }

    /// Property 3: Audit Log Completeness - Multiple operations
    /// For any sequence of tool operations, the audit log SHALL contain
    /// a record for each operation.
    #[test]
    fn prop_audit_log_completeness_multiple_operations(
        operations in prop::collection::vec(
            (tool_name_strategy(), prop::option::of(agent_name_strategy())),
            1..10
        ),
    ) {
        let logger = AuditLogger::new();

        // Log each operation
        for (tool_name, agent_name) in &operations {
            logger.log_execution(tool_name.clone(), agent_name.clone(), None).unwrap();
        }

        // Property: Log must contain exactly as many entries as operations
        let entries = logger.entries().unwrap();
        prop_assert_eq!(
            entries.len(),
            operations.len(),
            "Log must contain one entry per operation"
        );

        // Property: Each log entry must have correct tool name
        for (i, (tool_name, _)) in operations.iter().enumerate() {
            prop_assert_eq!(
                &entries[i].tool,
                tool_name,
                "Log entry {} must have correct tool name",
                i
            );
        }
    }

    /// Property 3: Audit Log Completeness - Prompt logging
    /// For any permission prompt, the audit log SHALL contain a record.
    #[test]
    fn prop_audit_log_completeness_prompt(
        tool_name in tool_name_strategy(),
        agent_name in prop::option::of(agent_name_strategy()),
    ) {
        let logger = AuditLogger::new();

        // Log a permission prompt
        let result = logger.log_prompt(tool_name.clone(), agent_name.clone(), None);

        // Property: Logging must succeed
        prop_assert!(result.is_ok(), "Logging prompt must succeed");

        // Property: Log must contain exactly one entry
        let entries = logger.entries().unwrap();
        prop_assert_eq!(
            entries.len(),
            1,
            "Log must contain exactly one entry after logging prompt"
        );

        // Property: Log entry must have correct action
        prop_assert_eq!(
            entries[0].action,
            AuditAction::Prompted,
            "Log entry must have Prompted action for prompt"
        );
    }
}
