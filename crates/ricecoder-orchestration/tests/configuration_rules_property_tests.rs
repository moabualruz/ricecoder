//! Property-based tests for configuration and rules management
//!
//! Tests verify correctness properties for configuration consistency and rules validation.

use proptest::prelude::*;
use ricecoder_orchestration::{
    ConfigManager, Project, ProjectStatus, RuleType, RulesValidator, ValidationRule, Workspace,
    WorkspaceConfig, WorkspaceMetrics, WorkspaceRule,
};
use serde_json::{json, Value};
use std::path::PathBuf;

// ============================================================================
// Property 2: Configuration Application Consistency
// ============================================================================

/// **Feature: ricecoder-orchestration, Property 2: Configuration Application Consistency**
///
/// For any workspace configuration, the ConfigManager SHALL apply workspace-level settings
/// consistently to all projects without conflicts or partial application.
#[test]
fn property_2_configuration_consistency() {
    proptest!(|(
        max_parallel in 1u32..32,
        timeout_ms in 1000u32..300000,
        enable_logging in any::<bool>(),
    )| {
        // Create a configuration with specific settings
        let config = WorkspaceConfig {
            rules: vec![],
            settings: json!({
                "max_parallel_operations": max_parallel,
                "transaction_timeout_ms": timeout_ms,
                "enable_audit_logging": enable_logging,
            }),
        };

        // Create a config manager and set the configuration
        let mut manager = ConfigManager::new(PathBuf::from("/workspace"));

        // Set each setting individually
        manager.set_setting("max_parallel_operations".to_string(), Value::Number(max_parallel.into())).unwrap();
        manager.set_setting("transaction_timeout_ms".to_string(), Value::Number(timeout_ms.into())).unwrap();
        manager.set_setting("enable_audit_logging".to_string(), Value::Bool(enable_logging)).unwrap();

        // Verify settings are applied consistently
        assert_eq!(
            manager.get_setting("max_parallel_operations"),
            Some(&Value::Number(max_parallel.into()))
        );
        assert_eq!(
            manager.get_setting("transaction_timeout_ms"),
            Some(&Value::Number(timeout_ms.into()))
        );
        assert_eq!(
            manager.get_setting("enable_audit_logging"),
            Some(&Value::Bool(enable_logging))
        );

        // Verify no partial application - all settings are present
        let settings = manager.get_config().settings.as_object().unwrap();
        assert_eq!(settings.len(), 3);
    });
}

/// **Feature: ricecoder-orchestration, Property 2: Configuration Application Consistency**
///
/// Configuration merging must be idempotent - merging the same configuration multiple times
/// produces the same result.
#[test]
fn property_2_configuration_merge_idempotent() {
    proptest!(|(
        key1 in "a-z+",
        value1 in "a-z0-9+",
        key2 in "a-z+",
        value2 in "a-z0-9+",
    )| {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));

        let base = WorkspaceConfig {
            rules: vec![],
            settings: json!({
                &key1: &value1,
            }),
        };

        let override_config = WorkspaceConfig {
            rules: vec![],
            settings: json!({
                &key2: &value2,
            }),
        };

        // Merge once
        let merged1 = manager.merge_configs(base.clone(), override_config.clone());

        // Merge again with the same inputs
        let merged2 = manager.merge_configs(base.clone(), override_config.clone());

        // Results must be identical (idempotent)
        assert_eq!(merged1.settings, merged2.settings);
    });
}

// ============================================================================
// Property 3: Rules Validation Correctness
// ============================================================================

/// **Feature: ricecoder-orchestration, Property 3: Rules Validation Correctness**
///
/// For any workspace rule, the RulesValidator SHALL correctly identify compliance violations
/// and reject non-compliant configurations.
#[test]
fn property_3_rules_validation_correctness() {
    proptest!(|(
        project_count in 1usize..10,
        has_invalid_name in any::<bool>(),
    )| {
        // Create projects with valid or invalid names
        let mut projects = Vec::new();
        for i in 0..project_count {
            let name = if has_invalid_name && i == 0 {
                format!("InvalidProject{}", i)
            } else {
                format!("project-{}", i)
            };

            projects.push(Project {
                path: PathBuf::from(format!("/workspace/{}", name)),
                name,
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Healthy,
            });
        }

        // Create workspace with naming convention rule
        let workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects,
            dependencies: vec![],
            config: WorkspaceConfig {
                rules: vec![WorkspaceRule {
                    name: "naming-convention".to_string(),
                    rule_type: RuleType::NamingConvention,
                    enabled: true,
                }],
                settings: json!({}),
            },
            metrics: WorkspaceMetrics::default(),
        };

        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all().unwrap();

        // If we have invalid names, violations must be found
        if has_invalid_name {
            assert!(!result.violations.is_empty());
            assert!(result.violations.iter().any(|v| v.rule_name == "naming-convention"));
        }
    });
}

/// **Feature: ricecoder-orchestration, Property 3: Rules Validation Correctness**
///
/// Circular dependency detection must be consistent - if a cycle exists, it must always be detected.
#[test]
fn property_3_circular_dependency_detection() {
    proptest!(|(
        create_cycle in any::<bool>(),
    )| {
        use ricecoder_orchestration::{ProjectDependency, DependencyType};

        let mut workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects: vec![
                Project {
                    path: PathBuf::from("/workspace/project-a"),
                    name: "project-a".to_string(),
                    project_type: "rust".to_string(),
                    version: "0.1.0".to_string(),
                    status: ProjectStatus::Healthy,
                },
                Project {
                    path: PathBuf::from("/workspace/project-b"),
                    name: "project-b".to_string(),
                    project_type: "rust".to_string(),
                    version: "0.1.0".to_string(),
                    status: ProjectStatus::Healthy,
                },
            ],
            dependencies: vec![],
            config: WorkspaceConfig {
                rules: vec![WorkspaceRule {
                    name: "no-circular-deps".to_string(),
                    rule_type: RuleType::DependencyConstraint,
                    enabled: true,
                }],
                settings: json!({}),
            },
            metrics: WorkspaceMetrics::default(),
        };

        // Add dependency from a to b
        workspace.dependencies.push(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        // Optionally create a cycle by adding b -> a
        if create_cycle {
            workspace.dependencies.push(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            });
        }

        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all().unwrap();

        // If cycle was created, violations must be found
        if create_cycle {
            assert!(!result.violations.is_empty());
        } else {
            // No cycle should be detected
            let has_cycle_violation = result.violations.iter().any(|v| {
                v.rule_name == "no-circular-deps"
            });
            assert!(!has_cycle_violation);
        }
    });
}

/// **Feature: ricecoder-orchestration, Property 3: Rules Validation Correctness**
///
/// Validation must be deterministic - validating the same configuration multiple times
/// produces identical results.
#[test]
fn property_3_validation_deterministic() {
    proptest!(|(
        project_count in 1usize..5,
    )| {
        // Create a workspace
        let mut projects = Vec::new();
        for i in 0..project_count {
            projects.push(Project {
                path: PathBuf::from(format!("/workspace/project-{}", i)),
                name: format!("project-{}", i),
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Healthy,
            });
        }

        let workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects,
            dependencies: vec![],
            config: WorkspaceConfig {
                rules: vec![WorkspaceRule {
                    name: "naming-convention".to_string(),
                    rule_type: RuleType::NamingConvention,
                    enabled: true,
                }],
                settings: json!({}),
            },
            metrics: WorkspaceMetrics::default(),
        };

        let validator = RulesValidator::new(workspace);

        // Validate multiple times
        let result1 = validator.validate_all().unwrap();
        let result2 = validator.validate_all().unwrap();
        let result3 = validator.validate_all().unwrap();

        // Results must be identical
        assert_eq!(result1.violations.len(), result2.violations.len());
        assert_eq!(result2.violations.len(), result3.violations.len());
        assert_eq!(result1.passed, result2.passed);
        assert_eq!(result2.passed, result3.passed);
    });
}

// ============================================================================
// Configuration Validation Tests
// ============================================================================

/// Test that configuration validation rejects invalid values
#[test]
fn property_configuration_validation_rejects_invalid() {
    proptest!(|(
        invalid_parallel in 0u32 | 33u32..1000,
        invalid_timeout in 0u32 | 300001u32..1000000,
    )| {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));

        // Test invalid max_parallel_operations
        let rule = ValidationRule {
            rule_type: "number".to_string(),
            min: Some(1.0),
            max: Some(32.0),
            allowed_values: None,
            pattern: None,
        };

        let result = manager.validate_value(
            "max_parallel_operations",
            &Value::Number(invalid_parallel.into()),
            &rule,
        );

        // Should reject values outside the range
        if invalid_parallel == 0 || invalid_parallel > 32 {
            assert!(result.is_err());
        }
    });
}

/// Test that configuration validation accepts valid values
#[test]
fn property_configuration_validation_accepts_valid() {
    proptest!(|(
        valid_parallel in 1u32..=32,
        valid_timeout in 1000u32..=300000,
    )| {
        let manager = ConfigManager::new(PathBuf::from("/workspace"));

        // Test valid max_parallel_operations
        let rule = ValidationRule {
            rule_type: "number".to_string(),
            min: Some(1.0),
            max: Some(32.0),
            allowed_values: None,
            pattern: None,
        };

        let result = manager.validate_value(
            "max_parallel_operations",
            &Value::Number(valid_parallel.into()),
            &rule,
        );

        // Should accept values in the range
        assert!(result.is_ok());

        // Test valid transaction_timeout_ms
        let rule2 = ValidationRule {
            rule_type: "number".to_string(),
            min: Some(1000.0),
            max: Some(300000.0),
            allowed_values: None,
            pattern: None,
        };

        let result2 = manager.validate_value(
            "transaction_timeout_ms",
            &Value::Number(valid_timeout.into()),
            &rule2,
        );

        // Should accept values in the range
        assert!(result2.is_ok());
    });
}

// ============================================================================
// Rules Validator Tests
// ============================================================================

/// Test that project name validation is consistent
#[test]
fn property_project_name_validation_consistent() {
    proptest!(|(
        name in "[a-z0-9-]{1,50}",
    )| {
        // Ensure name doesn't start or end with hyphen
        let name = if name.starts_with('-') || name.ends_with('-') {
            format!("project{}", name)
        } else {
            name
        };

        let workspace = Workspace {
            root: PathBuf::from("/workspace"),
            projects: vec![Project {
                path: PathBuf::from(format!("/workspace/{}", name)),
                name: name.clone(),
                project_type: "rust".to_string(),
                version: "0.1.0".to_string(),
                status: ProjectStatus::Healthy,
            }],
            dependencies: vec![],
            config: WorkspaceConfig::default(),
            metrics: WorkspaceMetrics::default(),
        };

        let validator = RulesValidator::new(workspace.clone());
        let project = workspace.projects[0].clone();

        // Validate the project multiple times
        let result1 = validator.validate_project(&project).unwrap();
        let result2 = validator.validate_project(&project).unwrap();

        // Results must be identical
        assert_eq!(result1.violations.len(), result2.violations.len());
        assert_eq!(result1.passed, result2.passed);
    });
}
