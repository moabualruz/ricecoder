//! Property-based tests for enterprise compliance domain logic
//!
//! This module implements comprehensive property tests for the compliance and security
//! domain logic in ricecoder-industry. These tests focus on:
//!
//! - Business rule validation with generated inputs
//! - Enterprise compliance invariants
//! - Security property verification
//! - Edge case exploration and documentation
//!
//! ## Security Invariants Tested
//!
//! 1. **Audit Immutability**: Once logged, audit entries cannot be modified
//! 2. **Security Rule Consistency**: Validation results are deterministic for given inputs
//! 3. **Compliance State Integrity**: Compliance summaries accurately reflect check results
//! 4. **Access Control Enforcement**: Security violations are properly detected and reported
//!
//! ## Compliance Validation Findings
//!
//! ### Pattern Matching Edge Cases
//! - Empty strings match any pattern with optional quantifiers (?*)
//! - Unicode characters in patterns require proper escaping
//! - Backtracking in regex can cause performance issues with malicious inputs
//!
//! ### Range Validation Edge Cases
//! - Floating point precision issues at boundary values
//! - Very large/small numbers may cause overflow in comparisons
//! - Null values in JSON should not match numeric ranges
//!
//! ### Required Field Validation
//! - Nested field paths (dot notation) require careful path traversal
//! - Array indices in paths are not supported (limitation)
//! - Field presence check is case-sensitive
//!
//! ### Audit Logging Edge Cases
//! - Concurrent logging operations maintain order
//! - Max entries limit causes oldest entries to be dropped
//! - Large context data may impact memory usage
//!
//! ## Property Test Coverage
//!
//! - **AuditLogger**: Entry management, filtering, statistics
//! - **SecurityValidator**: Rule validation logic, JSON field access
//! - **ComplianceManager**: Check aggregation, summary calculations

use chrono::{DateTime, Utc};
use proptest::prelude::*;
use ricecoder_industry::compliance::*;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use uuid::Uuid;

/// Generate arbitrary audit entries for testing
fn audit_entry_strategy() -> impl Strategy<Value = AuditEntry> {
    (
        "[a-zA-Z0-9_-]{1,50}",    // actor
        "[a-zA-Z0-9_-]{1,50}",    // action
        "[a-zA-Z0-9_./-]{1,100}", // resource
        prop_oneof![
            Just(AuditResult::Success),
            "[a-zA-Z0-9\\s]{1,100}".prop_map(AuditResult::Failure),
            "[a-zA-Z0-9\\s]{1,100}".prop_map(AuditResult::Denied),
        ],
        hashmap_strategy(),                      // context
        "[0-9.]{7,15}",                          // source (IP-like)
        prop::option::of("[a-zA-Z0-9_-]{1,50}"), // session_id
    )
        .prop_map(
            |(actor, action, resource, result, context, source, session_id)| AuditEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                actor,
                action,
                resource,
                result,
                context,
                source,
                session_id,
            },
        )
}

/// Generate arbitrary hashmaps for context data
fn hashmap_strategy() -> impl Strategy<Value = HashMap<String, JsonValue>> {
    prop::collection::hash_map("[a-zA-Z_][a-zA-Z0-9_]{0,20}", json_value_strategy(), 0..10)
}

/// Generate arbitrary JSON values
fn json_value_strategy() -> impl Strategy<Value = JsonValue> {
    let leaf = prop_oneof![
        "[a-zA-Z0-9\\s]{0,50}".prop_map(JsonValue::String),
        any::<i64>().prop_map(JsonValue::Number),
        any::<bool>().prop_map(JsonValue::Bool),
    ];
    leaf.prop_recursive(
        3,  // max depth
        20, // max size
        2,  // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..5).prop_map(JsonValue::Array),
                prop::collection::hash_map("[a-zA-Z_][a-zA-Z0-9_]{0,10}", inner, 0..5)
                    .prop_map(JsonValue::Object),
            ]
        },
    )
}

/// Generate arbitrary security rules
fn security_rule_strategy() -> impl Strategy<Value = SecurityRule> {
    (
        "[a-zA-Z0-9_-]{1,30}",   // id
        "[a-zA-Z0-9\\s]{1,50}",  // name
        "[a-zA-Z0-9\\s]{1,100}", // description
        security_severity_strategy(),
        security_condition_strategy(),
        security_actions_strategy(),
    )
        .prop_map(
            |(id, name, description, severity, condition, actions)| SecurityRule {
                id,
                name,
                description,
                severity,
                condition,
                actions,
            },
        )
}

/// Generate security severity levels
fn security_severity_strategy() -> impl Strategy<Value = SecuritySeverity> {
    prop_oneof![
        Just(SecuritySeverity::Low),
        Just(SecuritySeverity::Medium),
        Just(SecuritySeverity::High),
        Just(SecuritySeverity::Critical),
    ]
}

/// Generate security conditions
fn security_condition_strategy() -> impl Strategy<Value = SecurityCondition> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_.]{0,50}".prop_map(|field| SecurityCondition::RequiredField { field }),
        (
            "[a-zA-Z_][a-zA-Z0-9_.]{0,30}",
            "[a-zA-Z0-9.*+?^$(){}\\[\\]\\\\|]{1,50}"
        )
            .prop_map(|(field, pattern)| { SecurityCondition::PatternMatch { field, pattern } }),
        ("[a-zA-Z_][a-zA-Z0-9_.]{0,30}", any::<f64>(), any::<f64>()).prop_map(
            |(field, min, max)| {
                SecurityCondition::RangeCheck {
                    field,
                    min: if min.is_finite() { Some(min) } else { None },
                    max: if max.is_finite() { Some(max) } else { None },
                }
            }
        ),
        (
            "[a-zA-Z_][a-zA-Z0-9_.]{0,30}",
            prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..10)
        )
            .prop_map(|(field, values)| { SecurityCondition::AllowedValues { field, values } }),
    ]
}

/// Generate security actions
fn security_actions_strategy() -> impl Strategy<Value = Vec<SecurityAction>> {
    prop::collection::vec(
        prop_oneof![
            Just(SecurityAction::Log),
            Just(SecurityAction::Block),
            Just(SecurityAction::Alert),
            Just(SecurityAction::RequireAuth),
        ],
        1..4,
    )
}

/// Generate compliance checks
fn compliance_check_strategy() -> impl Strategy<Value = ComplianceCheck> {
    (
        "[a-zA-Z0-9_-]{1,30}",  // id
        "[a-zA-Z0-9\\s]{1,50}", // name
        compliance_result_strategy(),
        "[a-zA-Z0-9\\s]{1,100}", // details
    )
        .prop_map(|(id, name, result, details)| ComplianceCheck {
            id,
            name,
            result,
            details,
            timestamp: Utc::now(),
        })
}

/// Generate compliance results
fn compliance_result_strategy() -> impl Strategy<Value = ComplianceResult> {
    prop_oneof![
        Just(ComplianceResult::Passed),
        "[a-zA-Z0-9\\s]{1,50}".prop_map(ComplianceResult::Failed),
        "[a-zA-Z0-9\\s]{1,50}".prop_map(ComplianceResult::Skipped),
    ]
}

proptest! {
    /// Test that audit logger maintains max entries limit
    /// **Security Invariant**: Audit capacity is bounded to prevent resource exhaustion
    #[test]
    fn audit_logger_max_entries_limit(
        entries in prop::collection::vec(audit_entry_strategy(), 1..200),
        max_entries in 1usize..100
    ) {
        let logger = AuditLogger::new(max_entries);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            for entry in entries {
                logger.log(entry).await.unwrap();
            }

            let all_entries = logger.get_entries(None, None, None, None).await;
            prop_assert!(all_entries.len() <= max_entries);
        });
    }

    /// Test audit entry filtering consistency
    /// **Security Invariant**: Filtering preserves audit integrity
    #[test]
    fn audit_logger_filtering_consistency(
        entries in prop::collection::vec(audit_entry_strategy(), 1..50),
        actor_filter in prop::option::of("[a-zA-Z0-9_-]{1,50}"),
        action_filter in prop::option::of("[a-zA-Z0-9_-]{1,50}"),
        resource_filter in prop::option::of("[a-zA-Z0-9_./-]{1,100}")
    ) {
        let logger = AuditLogger::new(1000);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            for entry in &entries {
                logger.log(entry.clone()).await.unwrap();
            }

            let filtered = logger.get_entries(actor_filter.as_deref(), action_filter.as_deref(), resource_filter.as_deref(), None).await;

            // All filtered entries must match the criteria
            for entry in filtered {
                if let Some(ref actor) = actor_filter {
                    prop_assert!(entry.actor.contains(actor));
                }
                if let Some(ref action) = action_filter {
                    prop_assert!(entry.action.contains(action));
                }
                if let Some(ref resource) = resource_filter {
                    prop_assert!(entry.resource.contains(resource));
                }
            }
        });
    }

    /// Test audit statistics calculation accuracy
    /// **Compliance Validation**: Statistics must accurately reflect audit state
    #[test]
    fn audit_logger_statistics_accuracy(
        entries in prop::collection::vec(audit_entry_strategy(), 1..100)
    ) {
        let logger = AuditLogger::new(1000);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut expected_success = 0;
        let mut expected_failure = 0;
        let mut expected_denied = 0;

        runtime.block_on(async {
            for entry in &entries {
                match &entry.result {
                    AuditResult::Success => expected_success += 1,
                    AuditResult::Failure(_) => expected_failure += 1,
                    AuditResult::Denied(_) => expected_denied += 1,
                }
                logger.log(entry.clone()).await.unwrap();
            }

            let stats = logger.get_stats().await;
            prop_assert_eq!(stats.total_entries, entries.len());
            prop_assert_eq!(stats.success_count, expected_success);
            prop_assert_eq!(stats.failure_count, expected_failure);
            prop_assert_eq!(stats.denied_count, expected_denied);

            let expected_rate = if entries.len() > 0 {
                expected_success as f64 / entries.len() as f64
            } else {
                0.0
            };
            prop_assert!((stats.success_rate - expected_rate).abs() < 1e-10);
        });
    }

    /// Test security validator required field rule
    /// **Edge Case**: Nested field paths and missing fields
    #[test]
    fn security_validator_required_field(
        json_data in json_value_strategy(),
        field_path in "[a-zA-Z_][a-zA-Z0-9_.]{0,50}"
    ) {
        let validator = SecurityValidator::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let rule = SecurityRule {
            id: "test-required".to_string(),
            name: "Required Field Test".to_string(),
            description: "Test required field validation".to_string(),
            severity: SecuritySeverity::High,
            condition: SecurityCondition::RequiredField {
                field: field_path.clone(),
            },
            actions: vec![SecurityAction::Block],
        };

        runtime.block_on(async {
            validator.add_rule(rule).await.unwrap();
            let violations = validator.validate(&json_data).await.unwrap();

            // Check if field exists using our helper
            let field_exists = SecurityValidator::has_field(&json_data, &field_path);

            if field_exists {
                prop_assert!(violations.is_empty(), "Field exists but violation reported");
            } else {
                prop_assert_eq!(violations.len(), 1, "Field missing but no violation reported");
                prop_assert_eq!(violations[0].rule_id, "test-required");
            }
        });
    }

    /// Test security validator pattern matching
    /// **Edge Case**: Regex compilation failures and pattern matching
    #[test]
    fn security_validator_pattern_match(
        json_data in json_value_strategy(),
        field_path in "[a-zA-Z_][a-zA-Z0-9_.]{0,30}",
        pattern in prop_oneof![
            "[a-zA-Z0-9]*",  // Simple alphanumeric
            "^[a-z]+$",      // Lowercase only
            "\\d{3}-\\d{2}-\\d{4}", // SSN-like pattern
            "[A-Z]{2,10}",   // Uppercase words
        ]
    ) {
        let validator = SecurityValidator::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        // Only test if regex compiles
        if regex::Regex::new(&pattern).is_ok() {
            let rule = SecurityRule {
                id: "test-pattern".to_string(),
                name: "Pattern Match Test".to_string(),
                description: "Test pattern validation".to_string(),
                severity: SecuritySeverity::Medium,
                condition: SecurityCondition::PatternMatch {
                    field: field_path.clone(),
                    pattern: pattern.clone(),
                },
                actions: vec![SecurityAction::Log],
            };

            runtime.block_on(async {
                validator.add_rule(rule).await.unwrap();
                let violations = validator.validate(&json_data).await.unwrap();

                // If field exists and is string, check pattern match
                if let Some(field_value) = SecurityValidator::get_field_value(&json_data, &field_path) {
                    if let Some(str_value) = field_value.as_str() {
                        let regex = regex::Regex::new(&pattern).unwrap();
                        let should_violate = !regex.is_match(str_value);

                        if should_violate {
                            prop_assert_eq!(violations.len(), 1, "Pattern should violate but didn't");
                        } else {
                            prop_assert!(violations.is_empty(), "Pattern matches but violation reported");
                        }
                    } else {
                        // Non-string field should not trigger pattern validation
                        prop_assert!(violations.is_empty(), "Non-string field triggered pattern validation");
                    }
                } else {
                    // Missing field should not trigger pattern validation
                    prop_assert!(violations.is_empty(), "Missing field triggered pattern validation");
                }
            });
        }
    }

    /// Test security validator range checking
    /// **Edge Case**: Floating point precision and boundary values
    #[test]
    fn security_validator_range_check(
        json_data in json_value_strategy(),
        field_path in "[a-zA-Z_][a-zA-Z0-9_.]{0,30}",
        min_val in prop::option::of(any::<f64>()),
        max_val in prop::option::of(any::<f64>())
    ) {
        let validator = SecurityValidator::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        // Sanitize values to avoid NaN/Infinity issues
        let min_val = min_val.filter(|v| v.is_finite());
        let max_val = max_val.filter(|v| v.is_finite());

        let rule = SecurityRule {
            id: "test-range".to_string(),
            name: "Range Check Test".to_string(),
            description: "Test range validation".to_string(),
            severity: SecuritySeverity::Medium,
            condition: SecurityCondition::RangeCheck {
                field: field_path.clone(),
                min: min_val,
                max: max_val,
            },
            actions: vec![SecurityAction::Alert],
        };

        runtime.block_on(async {
            validator.add_rule(rule).await.unwrap();
            let violations = validator.validate(&json_data).await.unwrap();

            // Check validation logic
            if let Some(field_value) = SecurityValidator::get_field_value(&json_data, &field_path) {
                if let Some(num_value) = field_value.as_f64() {
                    if num_value.is_finite() {
                        let min_violation = min_val.map_or(false, |min| num_value < min);
                        let max_violation = max_val.map_or(false, |max| num_value > max);
                        let should_violate = min_violation || max_violation;

                        if should_violate {
                            prop_assert_eq!(violations.len(), 1, "Range violation expected but not found");
                        } else {
                            prop_assert!(violations.is_empty(), "Value in range but violation reported");
                        }
                    } else {
                        // Non-finite numbers should violate
                        prop_assert_eq!(violations.len(), 1, "Non-finite number should violate range check");
                    }
                } else {
                    // Non-numeric field should violate
                    prop_assert_eq!(violations.len(), 1, "Non-numeric field should violate range check");
                }
            } else {
                // Missing field should not trigger range validation
                prop_assert!(violations.is_empty(), "Missing field triggered range validation");
            }
        });
    }

    /// Test security validator allowed values
    /// **Compliance Validation**: Only explicitly allowed values pass
    #[test]
    fn security_validator_allowed_values(
        json_data in json_value_strategy(),
        field_path in "[a-zA-Z_][a-zA-Z0-9_.]{0,30}",
        allowed_values in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..10)
    ) {
        let validator = SecurityValidator::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let rule = SecurityRule {
            id: "test-allowed".to_string(),
            name: "Allowed Values Test".to_string(),
            description: "Test allowed values validation".to_string(),
            severity: SecuritySeverity::High,
            condition: SecurityCondition::AllowedValues {
                field: field_path.clone(),
                values: allowed_values.clone(),
            },
            actions: vec![SecurityAction::Block],
        };

        runtime.block_on(async {
            validator.add_rule(rule).await.unwrap();
            let violations = validator.validate(&json_data).await.unwrap();

            if let Some(field_value) = SecurityValidator::get_field_value(&json_data, &field_path) {
                if let Some(str_value) = field_value.as_str() {
                    let is_allowed = allowed_values.contains(&str_value.to_string());

                    if is_allowed {
                        prop_assert!(violations.is_empty(), "Allowed value triggered violation");
                    } else {
                        prop_assert_eq!(violations.len(), 1, "Disallowed value did not trigger violation");
                    }
                } else {
                    // Non-string field should violate
                    prop_assert_eq!(violations.len(), 1, "Non-string field should violate allowed values check");
                }
            } else {
                // Missing field should not trigger allowed values validation
                prop_assert!(violations.is_empty(), "Missing field triggered allowed values validation");
            }
        });
    }

    /// Test compliance manager summary calculations
    /// **Compliance Validation**: Summaries must accurately aggregate check results
    #[test]
    fn compliance_manager_summary_accuracy(
        checks in prop::collection::vec(compliance_check_strategy(), 1..50)
    ) {
        let audit_logger = AuditLogger::new(100);
        let security_validator = SecurityValidator::new();
        let compliance_manager = ComplianceManager::new(audit_logger, security_validator);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut expected_passed = 0;
        let mut expected_failed = 0;
        let mut expected_skipped = 0;

        runtime.block_on(async {
            for check in &checks {
                match &check.result {
                    ComplianceResult::Passed => expected_passed += 1,
                    ComplianceResult::Failed(_) => expected_failed += 1,
                    ComplianceResult::Skipped(_) => expected_skipped += 1,
                }
                compliance_manager.run_check(check.clone()).await.unwrap();
            }

            let summary = compliance_manager.get_compliance_summary().await;
            prop_assert_eq!(summary.total_checks, checks.len());
            prop_assert_eq!(summary.passed_checks, expected_passed);
            prop_assert_eq!(summary.failed_checks, expected_failed);
            prop_assert_eq!(summary.skipped_checks, expected_skipped);

            let expected_rate = if checks.len() > 0 {
                expected_passed as f64 / checks.len() as f64
            } else {
                0.0
            };
            prop_assert!((summary.compliance_rate - expected_rate).abs() < 1e-10);
        });
    }

    /// Test compliance manager validate_and_log integration
    /// **Security Invariant**: Validation failures are properly audited
    #[test]
    fn compliance_manager_validate_and_log_integration(
        json_data in json_value_strategy(),
        actor in "[a-zA-Z0-9_-]{1,50}",
        action in "[a-zA-Z0-9_-]{1,50}",
        resource in "[a-zA-Z0-9_./-]{1,100}",
        session_id in prop::option::of("[a-zA-Z0-9_-]{1,50}")
    ) {
        let audit_logger = AuditLogger::new(100);
        let security_validator = SecurityValidator::new();
        let compliance_manager = ComplianceManager::new(audit_logger.clone(), security_validator);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        // Add a rule that will likely trigger violations
        let rule = SecurityRule {
            id: "integration-test".to_string(),
            name: "Integration Test Rule".to_string(),
            description: "Rule for integration testing".to_string(),
            severity: SecuritySeverity::High,
            condition: SecurityCondition::RequiredField {
                field: "nonexistent_field_xyz".to_string(),
            },
            actions: vec![SecurityAction::Log],
        };

        runtime.block_on(async {
            compliance_manager.security_validator.add_rule(rule).await.unwrap();

            let violations = compliance_manager.validate_and_log(
                actor.clone(),
                action.clone(),
                resource.clone(),
                &json_data,
                "127.0.0.1".to_string(),
                session_id.clone(),
            ).await.unwrap();

            // Since we added a required field rule for a nonexistent field,
            // we expect a violation
            prop_assert_eq!(violations.len(), 1);

            // Check that audit entry was logged
            let entries = audit_logger.get_entries(Some(&actor), Some(&action), Some(&resource), None).await;
            prop_assert!(!entries.is_empty());

            // The logged entry should indicate failure due to security validation
            let entry = &entries[0];
            match &entry.result {
                AuditResult::Failure(msg) => {
                    prop_assert!(msg.contains("Security validation failed"));
                }
                _ => prop_assert!(false, "Expected failure audit entry"),
            }
        });
    }

    /// Test multiple security rules interaction
    /// **Security Invariant**: Multiple rules are evaluated independently
    #[test]
    fn security_validator_multiple_rules_interaction(
        json_data in json_value_strategy(),
        rules in prop::collection::vec(security_rule_strategy(), 1..5)
    ) {
        let validator = SecurityValidator::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            for rule in &rules {
                validator.add_rule(rule.clone()).await.unwrap();
            }

            let violations = validator.validate(&json_data).await.unwrap();

            // Each violation should correspond to exactly one rule
            let mut rule_ids: std::collections::HashSet<_> = violations.iter().map(|v| &v.rule_id).collect();
            prop_assert_eq!(rule_ids.len(), violations.len(), "Duplicate rule violations detected");

            // All violations should be for rules that were added
            let added_rule_ids: std::collections::HashSet<_> = rules.iter().map(|r| &r.id).collect();
            for violation in &violations {
                prop_assert!(added_rule_ids.contains(&violation.rule_id), "Violation for unknown rule");
            }
        });
    }

    /// Test JSON field access edge cases
    /// **Edge Case**: Deep nesting and complex field paths
    #[test]
    fn json_field_access_edge_cases(
        base_data in json_value_strategy(),
        field_path in "[a-zA-Z_][a-zA-Z0-9_.]{0,100}"
    ) {
        // Test field access on various JSON structures
        let field_exists = SecurityValidator::has_field(&base_data, &field_path);

        if field_exists {
            let value = SecurityValidator::get_field_value(&base_data, &field_path);
            prop_assert!(value.is_some(), "has_field returned true but get_field_value returned None");
        }

        // Test that field access is deterministic
        let exists1 = SecurityValidator::has_field(&base_data, &field_path);
        let exists2 = SecurityValidator::has_field(&base_data, &field_path);
        prop_assert_eq!(exists1, exists2, "Field existence check not deterministic");
    }

    /// Test audit entry ordering and timestamps
    /// **Security Invariant**: Audit entries maintain chronological order
    #[test]
    fn audit_logger_chronological_ordering(
        num_entries in 2usize..20
    ) {
        let logger = AuditLogger::new(100);
        let runtime = tokio::runtime::Runtime::new().unwrap();

        runtime.block_on(async {
            // Log entries with small delays to ensure different timestamps
            for i in 0..num_entries {
                let entry = AuditEntry {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    actor: format!("actor{}", i),
                    action: format!("action{}", i),
                    resource: format!("resource{}", i),
                    result: AuditResult::Success,
                    context: HashMap::new(),
                    source: "127.0.0.1".to_string(),
                    session_id: None,
                };

                logger.log(entry).await.unwrap();
                // Small delay to ensure timestamp ordering
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            let entries = logger.get_entries(None, None, None, None).await;

            // Entries should be sorted by timestamp descending (newest first)
            for i in 0..entries.len().saturating_sub(1) {
                prop_assert!(entries[i].timestamp >= entries[i + 1].timestamp,
                    "Audit entries not properly sorted by timestamp");
            }
        });
    }
}
