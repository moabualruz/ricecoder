//! Unit Tests for Dependency Management
//!
//! Tests for dependency scanning, update suggestions, PR creation, and security tracking.

use ricecoder_github::managers::{
    Dependency, DependencyManager, DependencyOperations, PinningConfig, UpdatePriority,
    UpdateReason, UpdateRiskLevel, Vulnerability, VulnerabilitySeverity,
};

// Tests for DependencyManager

#[test]
fn test_dependency_manager_creation() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    assert_eq!(manager.owner, "owner");
    assert_eq!(manager.repo, "repo");
}

#[test]
fn test_scan_dependencies_returns_results() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    assert!(!result.dependencies.is_empty());
    assert!(result.outdated_count > 0);
}

#[test]
fn test_scan_dependencies_identifies_vulnerabilities() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    assert!(result.vulnerable_count > 0);
    assert!(result.total_vulnerabilities > 0);
}

#[test]
fn test_scan_dependencies_counts_match() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");

    let actual_outdated = result.dependencies.iter().filter(|d| d.is_outdated).count();
    assert_eq!(result.outdated_count, actual_outdated);

    let actual_vulnerable = result
        .dependencies
        .iter()
        .filter(|d| !d.vulnerabilities.is_empty())
        .count();
    assert_eq!(result.vulnerable_count, actual_vulnerable);
}

#[test]
fn test_suggest_updates_returns_suggestions() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");
    assert!(!suggestions.is_empty());
}

#[test]
fn test_suggest_updates_identifies_security_vulnerabilities() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");

    let security_suggestions: Vec<_> = suggestions
        .iter()
        .filter(|s| {
            matches!(
                s.reason,
                UpdateReason::SecurityVulnerability | UpdateReason::OutdatedAndVulnerable
            )
        })
        .collect();
    assert!(!security_suggestions.is_empty());
}

#[test]
fn test_suggest_updates_sets_correct_risk_levels() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");

    for suggestion in suggestions {
        if !suggestion.dependency.vulnerabilities.is_empty() {
            assert_eq!(suggestion.risk_level, UpdateRiskLevel::High);
        }
    }
}

#[test]
fn test_create_update_pr_with_suggestions() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");

    let pr_result = manager
        .create_update_pr(&suggestions)
        .expect("Creating PR should succeed");

    assert!(pr_result.pr_number > 0);
    assert!(!pr_result.updated_dependencies.is_empty());
    assert!(!pr_result.pr_url.is_empty());
    assert!(!pr_result.branch_name.is_empty());
}

#[test]
fn test_create_update_pr_includes_all_dependencies() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");

    let pr_result = manager
        .create_update_pr(&suggestions)
        .expect("Creating PR should succeed");

    assert_eq!(pr_result.updated_dependencies.len(), suggestions.len());
}

#[test]
fn test_create_update_pr_fails_with_no_suggestions() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager.create_update_pr(&[]);
    assert!(result.is_err());
}

#[test]
fn test_create_update_pr_branch_name_format() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let suggestions = manager
        .suggest_updates(&scan_result)
        .expect("Suggesting updates should succeed");

    let pr_result = manager
        .create_update_pr(&suggestions)
        .expect("Creating PR should succeed");

    assert!(pr_result.branch_name.starts_with("deps/update-"));
}

#[test]
fn test_verify_update_returns_result() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager
        .verify_update(42)
        .expect("Verification should succeed");
    assert!(result.build_passed);
    assert!(result.tests_passed);
}

#[test]
fn test_verify_update_has_status_message() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let result = manager
        .verify_update(42)
        .expect("Verification should succeed");
    assert!(!result.status_message.is_empty());
}

#[test]
fn test_track_vulnerabilities_returns_report() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let report = manager
        .track_vulnerabilities(&scan_result)
        .expect("Tracking vulnerabilities should succeed");

    assert_eq!(
        report.total_vulnerabilities,
        scan_result.total_vulnerabilities
    );
}

#[test]
fn test_track_vulnerabilities_counts_by_severity() {
    let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
    let scan_result = manager
        .scan_dependencies()
        .expect("Scanning should succeed");
    let report = manager
        .track_vulnerabilities(&scan_result)
        .expect("Tracking vulnerabilities should succeed");

    let sum = report.critical_count + report.high_count + report.medium_count + report.low_count;
    assert_eq!(sum, report.total_vulnerabilities);
}

// Tests for Dependency struct

#[test]
fn test_dependency_clone() {
    let dep = Dependency {
        name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: Some("2.0.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    };
    let cloned = dep.clone();
    assert_eq!(dep, cloned);
}

#[test]
fn test_dependency_equality() {
    let dep1 = Dependency {
        name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: Some("2.0.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    };
    let dep2 = Dependency {
        name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: Some("2.0.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    };
    assert_eq!(dep1, dep2);
}

// Tests for Vulnerability struct

#[test]
fn test_vulnerability_clone() {
    let vuln = Vulnerability {
        cve_id: "CVE-2021-44228".to_string(),
        severity: VulnerabilitySeverity::Critical,
        description: "RCE vulnerability".to_string(),
        affected_versions: vec!["1.0.0".to_string()],
    };
    let cloned = vuln.clone();
    assert_eq!(vuln, cloned);
}

// Tests for VulnerabilitySeverity

#[test]
fn test_vulnerability_severity_ordering() {
    assert!(VulnerabilitySeverity::Low < VulnerabilitySeverity::Medium);
    assert!(VulnerabilitySeverity::Medium < VulnerabilitySeverity::High);
    assert!(VulnerabilitySeverity::High < VulnerabilitySeverity::Critical);
}

#[test]
fn test_vulnerability_severity_display() {
    assert_eq!(VulnerabilitySeverity::Low.to_string(), "Low");
    assert_eq!(VulnerabilitySeverity::Medium.to_string(), "Medium");
    assert_eq!(VulnerabilitySeverity::High.to_string(), "High");
    assert_eq!(VulnerabilitySeverity::Critical.to_string(), "Critical");
}

// Tests for UpdateReason

#[test]
fn test_update_reason_display() {
    assert_eq!(UpdateReason::Outdated.to_string(), "Outdated");
    assert_eq!(
        UpdateReason::SecurityVulnerability.to_string(),
        "Security Vulnerability"
    );
    assert_eq!(
        UpdateReason::OutdatedAndVulnerable.to_string(),
        "Outdated and Vulnerable"
    );
}

// Tests for UpdateRiskLevel

#[test]
fn test_update_risk_level_ordering() {
    assert!(UpdateRiskLevel::Low < UpdateRiskLevel::Medium);
    assert!(UpdateRiskLevel::Medium < UpdateRiskLevel::High);
}

#[test]
fn test_update_risk_level_display() {
    assert_eq!(UpdateRiskLevel::Low.to_string(), "Low");
    assert_eq!(UpdateRiskLevel::Medium.to_string(), "Medium");
    assert_eq!(UpdateRiskLevel::High.to_string(), "High");
}

// Tests for DependencyOperations

#[test]
fn test_apply_pinning_with_full_version() {
    let deps = vec![Dependency {
        name: "tokio".to_string(),
        current_version: "1.35.0".to_string(),
        latest_version: Some("1.36.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    }];

    let config = PinningConfig {
        pin_major: true,
        pin_minor: true,
        pin_patch: true,
    };

    let result =
        DependencyOperations::apply_pinning(&deps, &config).expect("Pinning should succeed");
    assert_eq!(result.pinned_dependencies.len(), 1);
    assert_eq!(
        result.pinning_config.get("tokio"),
        Some(&"1.35.0".to_string())
    );
}

#[test]
fn test_apply_pinning_with_major_only() {
    let deps = vec![Dependency {
        name: "tokio".to_string(),
        current_version: "1.35.0".to_string(),
        latest_version: Some("1.36.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    }];

    let config = PinningConfig {
        pin_major: true,
        pin_minor: false,
        pin_patch: false,
    };

    let result =
        DependencyOperations::apply_pinning(&deps, &config).expect("Pinning should succeed");
    assert_eq!(result.pinning_config.get("tokio"), Some(&"1.*".to_string()));
}

#[test]
fn test_verify_build_compatibility() {
    let result =
        DependencyOperations::verify_build_compatibility(&[]).expect("Verification should succeed");
    assert!(result.success);
    assert!(result.errors.is_empty());
}

#[test]
fn test_generate_security_report_with_vulnerabilities() {
    let deps = vec![Dependency {
        name: "log4j".to_string(),
        current_version: "2.14.0".to_string(),
        latest_version: Some("2.21.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![Vulnerability {
            cve_id: "CVE-2021-44228".to_string(),
            severity: VulnerabilitySeverity::Critical,
            description: "RCE vulnerability".to_string(),
            affected_versions: vec!["2.14.0".to_string()],
        }],
        dep_type: "runtime".to_string(),
    }];

    let report = DependencyOperations::generate_security_report(&deps)
        .expect("Report generation should succeed");
    assert_eq!(report.total_vulnerabilities, 1);
    assert_eq!(report.critical_vulnerabilities.len(), 1);
    assert!(report.risk_score > 0.0);
}

#[test]
fn test_is_update_safe_with_no_vulnerabilities() {
    let result = DependencyOperations::is_update_safe("1.0.0", "1.1.0", 0)
        .expect("Safety check should succeed");
    assert!(result);
}

#[test]
fn test_is_update_safe_with_vulnerabilities() {
    let result = DependencyOperations::is_update_safe("1.0.0", "1.1.0", 1)
        .expect("Safety check should succeed");
    assert!(!result);
}

#[test]
fn test_generate_recommendations_for_outdated() {
    let deps = vec![Dependency {
        name: "serde".to_string(),
        current_version: "1.0.190".to_string(),
        latest_version: Some("1.0.195".to_string()),
        is_outdated: true,
        vulnerabilities: vec![],
        dep_type: "runtime".to_string(),
    }];

    let recommendations = DependencyOperations::generate_recommendations(&deps)
        .expect("Recommendations should succeed");
    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0].dependency_name, "serde");
}

#[test]
fn test_generate_recommendations_for_vulnerable() {
    let deps = vec![Dependency {
        name: "openssl".to_string(),
        current_version: "1.1.1k".to_string(),
        latest_version: Some("3.0.0".to_string()),
        is_outdated: true,
        vulnerabilities: vec![Vulnerability {
            cve_id: "CVE-2023-0286".to_string(),
            severity: VulnerabilitySeverity::High,
            description: "X.509 bypass".to_string(),
            affected_versions: vec!["1.1.1k".to_string()],
        }],
        dep_type: "runtime".to_string(),
    }];

    let recommendations = DependencyOperations::generate_recommendations(&deps)
        .expect("Recommendations should succeed");
    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0].priority, UpdatePriority::High);
}

#[test]
fn test_update_priority_ordering() {
    assert!(UpdatePriority::Low < UpdatePriority::Medium);
    assert!(UpdatePriority::Medium < UpdatePriority::High);
    assert!(UpdatePriority::High < UpdatePriority::Critical);
}

#[test]
fn test_update_priority_display() {
    assert_eq!(UpdatePriority::Low.to_string(), "Low");
    assert_eq!(UpdatePriority::Medium.to_string(), "Medium");
    assert_eq!(UpdatePriority::High.to_string(), "High");
    assert_eq!(UpdatePriority::Critical.to_string(), "Critical");
}

#[test]
fn test_pinning_config_default() {
    let config = PinningConfig::default();
    assert!(config.pin_major);
    assert!(config.pin_minor);
    assert!(!config.pin_patch);
}

#[test]
fn test_apply_pinning_preserves_all_dependencies() {
    let deps = vec![
        Dependency {
            name: "dep1".to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: None,
            is_outdated: false,
            vulnerabilities: vec![],
            dep_type: "runtime".to_string(),
        },
        Dependency {
            name: "dep2".to_string(),
            current_version: "2.0.0".to_string(),
            latest_version: None,
            is_outdated: false,
            vulnerabilities: vec![],
            dep_type: "dev".to_string(),
        },
    ];

    let config = PinningConfig::default();
    let result =
        DependencyOperations::apply_pinning(&deps, &config).expect("Pinning should succeed");

    assert_eq!(result.pinned_dependencies.len(), 2);
    assert!(result.pinning_config.contains_key("dep1"));
    assert!(result.pinning_config.contains_key("dep2"));
}

#[test]
fn test_generate_security_report_categorizes_by_severity() {
    let deps = vec![
        Dependency {
            name: "dep1".to_string(),
            current_version: "1.0.0".to_string(),
            latest_version: None,
            is_outdated: false,
            vulnerabilities: vec![Vulnerability {
                cve_id: "CVE-2023-0001".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "Critical issue".to_string(),
                affected_versions: vec!["1.0.0".to_string()],
            }],
            dep_type: "runtime".to_string(),
        },
        Dependency {
            name: "dep2".to_string(),
            current_version: "2.0.0".to_string(),
            latest_version: None,
            is_outdated: false,
            vulnerabilities: vec![Vulnerability {
                cve_id: "CVE-2023-0002".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "High issue".to_string(),
                affected_versions: vec!["2.0.0".to_string()],
            }],
            dep_type: "runtime".to_string(),
        },
    ];

    let report = DependencyOperations::generate_security_report(&deps)
        .expect("Report generation should succeed");
    assert_eq!(report.critical_vulnerabilities.len(), 1);
    assert_eq!(report.high_vulnerabilities.len(), 1);
}
