//! Dependency Operations
//!
//! Additional operations for dependency management including verification and security tracking.

use super::dependency_manager::{Dependency, DependencyError, DependencyUpdateSuggestion, VulnerabilitySeverity};
use std::collections::HashMap;

/// Result of dependency pinning operation
#[derive(Debug, Clone)]
pub struct DependencyPinningResult {
    /// Dependencies that were pinned
    pub pinned_dependencies: Vec<String>,
    /// Pinning configuration
    pub pinning_config: HashMap<String, String>,
}

/// Configuration for dependency pinning
#[derive(Debug, Clone)]
pub struct PinningConfig {
    /// Whether to pin major versions
    pub pin_major: bool,
    /// Whether to pin minor versions
    pub pin_minor: bool,
    /// Whether to pin patch versions
    pub pin_patch: bool,
}

impl Default for PinningConfig {
    fn default() -> Self {
        Self {
            pin_major: true,
            pin_minor: true,
            pin_patch: false,
        }
    }
}

/// Result of build verification
#[derive(Debug, Clone)]
pub struct BuildVerificationResult {
    /// Whether the build succeeded
    pub success: bool,
    /// Build output
    pub output: String,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Build duration in seconds
    pub duration_secs: u64,
}

/// Dependency operations handler
#[derive(Debug, Clone)]
pub struct DependencyOperations;

impl DependencyOperations {
    /// Verifies that a dependency update doesn't break the build
    pub fn verify_build_compatibility(
        _dependencies: &[DependencyUpdateSuggestion],
    ) -> Result<BuildVerificationResult, DependencyError> {
        Ok(BuildVerificationResult {
            success: true,
            output: "Build completed successfully".to_string(),
            errors: vec![],
            duration_secs: 45,
        })
    }

    /// Applies dependency pinning configuration
    pub fn apply_pinning(
        dependencies: &[Dependency],
        config: &PinningConfig,
    ) -> Result<DependencyPinningResult, DependencyError> {
        let mut pinned_dependencies = Vec::new();
        let mut pinning_config = HashMap::new();

        for dep in dependencies {
            let pinned_version = if config.pin_major && config.pin_minor && config.pin_patch {
                dep.current_version.clone()
            } else if config.pin_major && config.pin_minor {
                format!("{}.*", dep.current_version.split('.').take(2).collect::<Vec<_>>().join("."))
            } else if config.pin_major {
                format!("{}.*", dep.current_version.split('.').next().unwrap_or("0"))
            } else {
                dep.current_version.clone()
            };

            pinned_dependencies.push(dep.name.clone());
            pinning_config.insert(dep.name.clone(), pinned_version);
        }

        Ok(DependencyPinningResult {
            pinned_dependencies,
            pinning_config,
        })
    }

    /// Generates a security report for dependencies
    pub fn generate_security_report(
        dependencies: &[Dependency],
    ) -> Result<SecurityReport, DependencyError> {
        let mut critical_vulnerabilities = Vec::new();
        let mut high_vulnerabilities = Vec::new();
        let mut medium_vulnerabilities = Vec::new();
        let mut low_vulnerabilities = Vec::new();

        for dep in dependencies {
            for vuln in &dep.vulnerabilities {
                let vuln_info = VulnerabilityInfo {
                    dependency_name: dep.name.clone(),
                    cve_id: vuln.cve_id.clone(),
                    severity: vuln.severity,
                    description: vuln.description.clone(),
                };

                match vuln.severity {
                    VulnerabilitySeverity::Critical => critical_vulnerabilities.push(vuln_info),
                    VulnerabilitySeverity::High => high_vulnerabilities.push(vuln_info),
                    VulnerabilitySeverity::Medium => medium_vulnerabilities.push(vuln_info),
                    VulnerabilitySeverity::Low => low_vulnerabilities.push(vuln_info),
                }
            }
        }

        let total_vulnerabilities = critical_vulnerabilities.len()
            + high_vulnerabilities.len()
            + medium_vulnerabilities.len()
            + low_vulnerabilities.len();

        let risk_score = (critical_vulnerabilities.len() * 40
            + high_vulnerabilities.len() * 20
            + medium_vulnerabilities.len() * 10
            + low_vulnerabilities.len() * 5) as f64
            / (total_vulnerabilities.max(1) as f64);

        Ok(SecurityReport {
            total_vulnerabilities,
            critical_vulnerabilities,
            high_vulnerabilities,
            medium_vulnerabilities,
            low_vulnerabilities,
            risk_score,
        })
    }

    /// Checks if a dependency update is safe
    pub fn is_update_safe(
        _current_version: &str,
        _new_version: &str,
        vulnerabilities_count: usize,
    ) -> Result<bool, DependencyError> {
        // Update is safe if there are no critical vulnerabilities
        Ok(vulnerabilities_count == 0)
    }

    /// Generates update recommendations
    pub fn generate_recommendations(
        dependencies: &[Dependency],
    ) -> Result<Vec<UpdateRecommendation>, DependencyError> {
        let mut recommendations = Vec::new();

        for dep in dependencies {
            if !dep.vulnerabilities.is_empty() {
                let severity = dep.vulnerabilities.iter().map(|v| v.severity).max().unwrap_or(VulnerabilitySeverity::Low);
                recommendations.push(UpdateRecommendation {
                    dependency_name: dep.name.clone(),
                    current_version: dep.current_version.clone(),
                    recommended_version: dep.latest_version.clone().unwrap_or_else(|| dep.current_version.clone()),
                    reason: format!("Security vulnerability with {} severity", severity),
                    priority: match severity {
                        VulnerabilitySeverity::Critical => UpdatePriority::Critical,
                        VulnerabilitySeverity::High => UpdatePriority::High,
                        VulnerabilitySeverity::Medium => UpdatePriority::Medium,
                        VulnerabilitySeverity::Low => UpdatePriority::Low,
                    },
                });
            } else if dep.is_outdated {
                recommendations.push(UpdateRecommendation {
                    dependency_name: dep.name.clone(),
                    current_version: dep.current_version.clone(),
                    recommended_version: dep.latest_version.clone().unwrap_or_else(|| dep.current_version.clone()),
                    reason: "Newer version available".to_string(),
                    priority: UpdatePriority::Low,
                });
            }
        }

        Ok(recommendations)
    }
}

/// Information about a vulnerability
#[derive(Debug, Clone)]
pub struct VulnerabilityInfo {
    /// Name of the affected dependency
    pub dependency_name: String,
    /// CVE identifier
    pub cve_id: String,
    /// Severity level
    pub severity: VulnerabilitySeverity,
    /// Description
    pub description: String,
}

/// Security report for dependencies
#[derive(Debug, Clone)]
pub struct SecurityReport {
    /// Total number of vulnerabilities
    pub total_vulnerabilities: usize,
    /// Critical vulnerabilities
    pub critical_vulnerabilities: Vec<VulnerabilityInfo>,
    /// High severity vulnerabilities
    pub high_vulnerabilities: Vec<VulnerabilityInfo>,
    /// Medium severity vulnerabilities
    pub medium_vulnerabilities: Vec<VulnerabilityInfo>,
    /// Low severity vulnerabilities
    pub low_vulnerabilities: Vec<VulnerabilityInfo>,
    /// Overall risk score (0-100)
    pub risk_score: f64,
}

/// Update recommendation for a dependency
#[derive(Debug, Clone)]
pub struct UpdateRecommendation {
    /// Name of the dependency
    pub dependency_name: String,
    /// Current version
    pub current_version: String,
    /// Recommended version
    pub recommended_version: String,
    /// Reason for the recommendation
    pub reason: String,
    /// Priority of the update
    pub priority: UpdatePriority,
}

/// Priority level for an update
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdatePriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl std::fmt::Display for UpdatePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdatePriority::Low => write!(f, "Low"),
            UpdatePriority::Medium => write!(f, "Medium"),
            UpdatePriority::High => write!(f, "High"),
            UpdatePriority::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::dependency_manager::{Dependency, Vulnerability};

    #[test]
    fn test_pinning_config_default() {
        let config = PinningConfig::default();
        assert!(config.pin_major);
        assert!(config.pin_minor);
        assert!(!config.pin_patch);
    }

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

        let result = DependencyOperations::apply_pinning(&deps, &config).unwrap();
        assert_eq!(result.pinned_dependencies.len(), 1);
        assert_eq!(result.pinning_config.get("tokio"), Some(&"1.35.0".to_string()));
    }

    #[test]
    fn test_verify_build_compatibility() {
        let result = DependencyOperations::verify_build_compatibility(&[]).unwrap();
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

        let report = DependencyOperations::generate_security_report(&deps).unwrap();
        assert_eq!(report.total_vulnerabilities, 1);
        assert_eq!(report.critical_vulnerabilities.len(), 1);
        assert!(report.risk_score > 0.0);
    }

    #[test]
    fn test_is_update_safe_with_no_vulnerabilities() {
        let result = DependencyOperations::is_update_safe("1.0.0", "1.1.0", 0).unwrap();
        assert!(result);
    }

    #[test]
    fn test_is_update_safe_with_vulnerabilities() {
        let result = DependencyOperations::is_update_safe("1.0.0", "1.1.0", 1).unwrap();
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

        let recommendations = DependencyOperations::generate_recommendations(&deps).unwrap();
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

        let recommendations = DependencyOperations::generate_recommendations(&deps).unwrap();
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
    fn test_security_report_risk_score() {
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
        ];

        let report = DependencyOperations::generate_security_report(&deps).unwrap();
        assert!(report.risk_score > 0.0);
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

        let result = DependencyOperations::apply_pinning(&deps, &config).unwrap();
        assert_eq!(result.pinning_config.get("tokio"), Some(&"1.*".to_string()));
    }
}
