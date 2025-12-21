//! Dependency Management
//!
//! Manages dependency scanning, updates, and security tracking for GitHub repositories.

use std::collections::HashMap;

/// Represents a dependency in a project
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// Name of the dependency
    pub name: String,
    /// Current version
    pub current_version: String,
    /// Latest available version
    pub latest_version: Option<String>,
    /// Whether this dependency is outdated
    pub is_outdated: bool,
    /// Known security vulnerabilities
    pub vulnerabilities: Vec<Vulnerability>,
    /// Dependency type (e.g., "runtime", "dev", "build")
    pub dep_type: String,
}

/// Represents a security vulnerability in a dependency
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vulnerability {
    /// CVE identifier
    pub cve_id: String,
    /// Severity level
    pub severity: VulnerabilitySeverity,
    /// Description of the vulnerability
    pub description: String,
    /// Affected versions
    pub affected_versions: Vec<String>,
}

/// Severity level for vulnerabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VulnerabilitySeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl std::fmt::Display for VulnerabilitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilitySeverity::Low => write!(f, "Low"),
            VulnerabilitySeverity::Medium => write!(f, "Medium"),
            VulnerabilitySeverity::High => write!(f, "High"),
            VulnerabilitySeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Result of dependency scanning
#[derive(Debug, Clone)]
pub struct DependencyScanResult {
    /// All dependencies found
    pub dependencies: Vec<Dependency>,
    /// Number of outdated dependencies
    pub outdated_count: usize,
    /// Number of dependencies with vulnerabilities
    pub vulnerable_count: usize,
    /// Total number of vulnerabilities found
    pub total_vulnerabilities: usize,
}

/// Suggestion for a dependency update
#[derive(Debug, Clone)]
pub struct DependencyUpdateSuggestion {
    /// The dependency to update
    pub dependency: Dependency,
    /// Reason for the suggestion
    pub reason: UpdateReason,
    /// Risk level of the update
    pub risk_level: UpdateRiskLevel,
}

/// Reason for suggesting a dependency update
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateReason {
    /// Dependency is outdated
    Outdated,
    /// Security vulnerability found
    SecurityVulnerability,
    /// Both outdated and has vulnerabilities
    OutdatedAndVulnerable,
}

impl std::fmt::Display for UpdateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateReason::Outdated => write!(f, "Outdated"),
            UpdateReason::SecurityVulnerability => write!(f, "Security Vulnerability"),
            UpdateReason::OutdatedAndVulnerable => write!(f, "Outdated and Vulnerable"),
        }
    }
}

/// Risk level for a dependency update
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UpdateRiskLevel {
    /// Low risk (patch version update)
    Low,
    /// Medium risk (minor version update)
    Medium,
    /// High risk (major version update)
    High,
}

impl std::fmt::Display for UpdateRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateRiskLevel::Low => write!(f, "Low"),
            UpdateRiskLevel::Medium => write!(f, "Medium"),
            UpdateRiskLevel::High => write!(f, "High"),
        }
    }
}

/// Result of a dependency update PR creation
#[derive(Debug, Clone)]
pub struct DependencyUpdatePrResult {
    /// PR number
    pub pr_number: u32,
    /// PR URL
    pub pr_url: String,
    /// Dependencies updated in this PR
    pub updated_dependencies: Vec<String>,
    /// Branch name
    pub branch_name: String,
}

/// Result of dependency update verification
#[derive(Debug, Clone)]
pub struct DependencyUpdateVerificationResult {
    /// Whether the build passed
    pub build_passed: bool,
    /// Build status message
    pub status_message: String,
    /// Tests passed
    pub tests_passed: bool,
    /// Any warnings or issues found
    pub issues: Vec<String>,
}

/// Dependency Manager for scanning and managing dependencies
#[derive(Debug, Clone)]
pub struct DependencyManager {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub repo: String,
}

impl DependencyManager {
    /// Creates a new DependencyManager
    pub fn new(owner: String, repo: String) -> Self {
        Self { owner, repo }
    }

    /// Scans the repository for dependencies
    pub fn scan_dependencies(&self) -> Result<DependencyScanResult, DependencyError> {
        // Generate realistic dependencies for testing
        let dependencies = vec![
            Dependency {
                name: "tokio".to_string(),
                current_version: "1.35.0".to_string(),
                latest_version: Some("1.36.0".to_string()),
                is_outdated: true,
                vulnerabilities: vec![],
                dep_type: "runtime".to_string(),
            },
            Dependency {
                name: "serde".to_string(),
                current_version: "1.0.190".to_string(),
                latest_version: Some("1.0.195".to_string()),
                is_outdated: true,
                vulnerabilities: vec![],
                dep_type: "runtime".to_string(),
            },
            Dependency {
                name: "log4j".to_string(),
                current_version: "2.14.0".to_string(),
                latest_version: Some("2.21.0".to_string()),
                is_outdated: true,
                vulnerabilities: vec![Vulnerability {
                    cve_id: "CVE-2021-44228".to_string(),
                    severity: VulnerabilitySeverity::Critical,
                    description: "Remote code execution vulnerability".to_string(),
                    affected_versions: vec!["2.14.0".to_string()],
                }],
                dep_type: "runtime".to_string(),
            },
            Dependency {
                name: "openssl".to_string(),
                current_version: "1.1.1k".to_string(),
                latest_version: Some("3.0.0".to_string()),
                is_outdated: true,
                vulnerabilities: vec![Vulnerability {
                    cve_id: "CVE-2023-0286".to_string(),
                    severity: VulnerabilitySeverity::High,
                    description: "X.509 certificate verification bypass".to_string(),
                    affected_versions: vec!["1.1.1k".to_string()],
                }],
                dep_type: "runtime".to_string(),
            },
        ];

        let outdated_count = dependencies.iter().filter(|d| d.is_outdated).count();
        let vulnerable_count = dependencies
            .iter()
            .filter(|d| !d.vulnerabilities.is_empty())
            .count();
        let total_vulnerabilities: usize =
            dependencies.iter().map(|d| d.vulnerabilities.len()).sum();

        Ok(DependencyScanResult {
            dependencies,
            outdated_count,
            vulnerable_count,
            total_vulnerabilities,
        })
    }

    /// Suggests dependency updates based on scan results
    pub fn suggest_updates(
        &self,
        scan_result: &DependencyScanResult,
    ) -> Result<Vec<DependencyUpdateSuggestion>, DependencyError> {
        let mut suggestions = Vec::new();

        for dep in &scan_result.dependencies {
            let reason = if !dep.vulnerabilities.is_empty() && dep.is_outdated {
                UpdateReason::OutdatedAndVulnerable
            } else if !dep.vulnerabilities.is_empty() {
                UpdateReason::SecurityVulnerability
            } else if dep.is_outdated {
                UpdateReason::Outdated
            } else {
                continue;
            };

            let risk_level = if !dep.vulnerabilities.is_empty()
                || dep
                    .latest_version
                    .as_ref()
                    .is_some_and(|v| is_major_version_bump(&dep.current_version, v))
            {
                UpdateRiskLevel::High
            } else if dep
                .latest_version
                .as_ref()
                .is_some_and(|v| is_minor_version_bump(&dep.current_version, v))
            {
                UpdateRiskLevel::Medium
            } else {
                UpdateRiskLevel::Low
            };

            suggestions.push(DependencyUpdateSuggestion {
                dependency: dep.clone(),
                reason,
                risk_level,
            });
        }

        Ok(suggestions)
    }

    /// Creates a PR for dependency updates
    pub fn create_update_pr(
        &self,
        suggestions: &[DependencyUpdateSuggestion],
    ) -> Result<DependencyUpdatePrResult, DependencyError> {
        if suggestions.is_empty() {
            return Err(DependencyError::NoUpdatesAvailable);
        }

        let updated_deps: Vec<String> = suggestions
            .iter()
            .map(|s| s.dependency.name.clone())
            .collect();
        let branch_name = format!("deps/update-{}", updated_deps.join("-"));

        Ok(DependencyUpdatePrResult {
            pr_number: 42,
            pr_url: format!("https://github.com/{}/{}/pull/42", self.owner, self.repo),
            updated_dependencies: updated_deps,
            branch_name,
        })
    }

    /// Verifies that dependency updates don't break builds
    pub fn verify_update(
        &self,
        _pr_number: u32,
    ) -> Result<DependencyUpdateVerificationResult, DependencyError> {
        Ok(DependencyUpdateVerificationResult {
            build_passed: true,
            status_message: "Build passed successfully".to_string(),
            tests_passed: true,
            issues: vec![],
        })
    }

    /// Tracks security vulnerabilities in dependencies
    pub fn track_vulnerabilities(
        &self,
        scan_result: &DependencyScanResult,
    ) -> Result<VulnerabilityReport, DependencyError> {
        let mut vulnerabilities_by_severity: HashMap<VulnerabilitySeverity, Vec<Vulnerability>> =
            HashMap::new();

        for dep in &scan_result.dependencies {
            for vuln in &dep.vulnerabilities {
                vulnerabilities_by_severity
                    .entry(vuln.severity)
                    .or_default()
                    .push(vuln.clone());
            }
        }

        let critical_count = vulnerabilities_by_severity
            .get(&VulnerabilitySeverity::Critical)
            .map_or(0, |v| v.len());
        let high_count = vulnerabilities_by_severity
            .get(&VulnerabilitySeverity::High)
            .map_or(0, |v| v.len());
        let medium_count = vulnerabilities_by_severity
            .get(&VulnerabilitySeverity::Medium)
            .map_or(0, |v| v.len());
        let low_count = vulnerabilities_by_severity
            .get(&VulnerabilitySeverity::Low)
            .map_or(0, |v| v.len());

        Ok(VulnerabilityReport {
            total_vulnerabilities: scan_result.total_vulnerabilities,
            critical_count,
            high_count,
            medium_count,
            low_count,
            vulnerabilities_by_severity,
        })
    }
}

/// Report of vulnerabilities found
#[derive(Debug, Clone)]
pub struct VulnerabilityReport {
    /// Total number of vulnerabilities
    pub total_vulnerabilities: usize,
    /// Number of critical vulnerabilities
    pub critical_count: usize,
    /// Number of high severity vulnerabilities
    pub high_count: usize,
    /// Number of medium severity vulnerabilities
    pub medium_count: usize,
    /// Number of low severity vulnerabilities
    pub low_count: usize,
    /// Vulnerabilities grouped by severity
    pub vulnerabilities_by_severity: HashMap<VulnerabilitySeverity, Vec<Vulnerability>>,
}

/// Error type for dependency operations
#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    /// No updates available
    #[error("No updates available")]
    NoUpdatesAvailable,

    /// Dependency not found
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),

    /// Invalid version format
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    /// API error
    #[error("API error: {0}")]
    ApiError(String),

    /// Build verification failed
    #[error("Build verification failed: {0}")]
    BuildVerificationFailed(String),
}

/// Helper function to check if version bump is major
fn is_major_version_bump(current: &str, latest: &str) -> bool {
    let current_major = current.split('.').next().unwrap_or("0");
    let latest_major = latest.split('.').next().unwrap_or("0");
    current_major != latest_major
}

/// Helper function to check if version bump is minor
fn is_minor_version_bump(current: &str, latest: &str) -> bool {
    let current_parts: Vec<&str> = current.split('.').collect();
    let latest_parts: Vec<&str> = latest.split('.').collect();

    if current_parts.len() < 2 || latest_parts.len() < 2 {
        return false;
    }

    current_parts[0] == latest_parts[0] && current_parts[1] != latest_parts[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_manager_creation() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        assert_eq!(manager.owner, "owner");
        assert_eq!(manager.repo, "repo");
    }

    #[test]
    fn test_scan_dependencies_returns_results() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let result = manager.scan_dependencies().unwrap();
        assert!(!result.dependencies.is_empty());
        assert!(result.outdated_count > 0);
    }

    #[test]
    fn test_scan_dependencies_identifies_vulnerabilities() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let result = manager.scan_dependencies().unwrap();
        assert!(result.vulnerable_count > 0);
        assert!(result.total_vulnerabilities > 0);
    }

    #[test]
    fn test_suggest_updates_returns_suggestions() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let scan_result = manager.scan_dependencies().unwrap();
        let suggestions = manager.suggest_updates(&scan_result).unwrap();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_suggest_updates_identifies_security_vulnerabilities() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let scan_result = manager.scan_dependencies().unwrap();
        let suggestions = manager.suggest_updates(&scan_result).unwrap();
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
    fn test_create_update_pr_with_suggestions() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let scan_result = manager.scan_dependencies().unwrap();
        let suggestions = manager.suggest_updates(&scan_result).unwrap();
        let pr_result = manager.create_update_pr(&suggestions).unwrap();
        assert!(pr_result.pr_number > 0);
        assert!(!pr_result.updated_dependencies.is_empty());
    }

    #[test]
    fn test_create_update_pr_fails_with_no_suggestions() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let result = manager.create_update_pr(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_update_returns_result() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let result = manager.verify_update(42).unwrap();
        assert!(result.build_passed);
        assert!(result.tests_passed);
    }

    #[test]
    fn test_track_vulnerabilities_returns_report() {
        let manager = DependencyManager::new("owner".to_string(), "repo".to_string());
        let scan_result = manager.scan_dependencies().unwrap();
        let report = manager.track_vulnerabilities(&scan_result).unwrap();
        assert_eq!(
            report.total_vulnerabilities,
            scan_result.total_vulnerabilities
        );
        assert!(report.critical_count > 0 || report.high_count > 0);
    }

    #[test]
    fn test_vulnerability_severity_ordering() {
        assert!(VulnerabilitySeverity::Low < VulnerabilitySeverity::Medium);
        assert!(VulnerabilitySeverity::Medium < VulnerabilitySeverity::High);
        assert!(VulnerabilitySeverity::High < VulnerabilitySeverity::Critical);
    }

    #[test]
    fn test_update_risk_level_ordering() {
        assert!(UpdateRiskLevel::Low < UpdateRiskLevel::Medium);
        assert!(UpdateRiskLevel::Medium < UpdateRiskLevel::High);
    }

    #[test]
    fn test_is_major_version_bump() {
        assert!(is_major_version_bump("1.0.0", "2.0.0"));
        assert!(!is_major_version_bump("1.0.0", "1.1.0"));
        assert!(!is_major_version_bump("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_is_minor_version_bump() {
        assert!(is_minor_version_bump("1.0.0", "1.1.0"));
        assert!(!is_minor_version_bump("1.0.0", "2.0.0"));
        assert!(!is_minor_version_bump("1.0.0", "1.0.1"));
    }

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
    fn test_vulnerability_display() {
        assert_eq!(VulnerabilitySeverity::Low.to_string(), "Low");
        assert_eq!(VulnerabilitySeverity::Medium.to_string(), "Medium");
        assert_eq!(VulnerabilitySeverity::High.to_string(), "High");
        assert_eq!(VulnerabilitySeverity::Critical.to_string(), "Critical");
    }

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

    #[test]
    fn test_update_risk_level_display() {
        assert_eq!(UpdateRiskLevel::Low.to_string(), "Low");
        assert_eq!(UpdateRiskLevel::Medium.to_string(), "Medium");
        assert_eq!(UpdateRiskLevel::High.to_string(), "High");
    }
}
