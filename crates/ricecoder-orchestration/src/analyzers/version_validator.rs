//! Version constraint validation and compatibility checking

use crate::error::{OrchestrationError, Result};

/// Represents a semantic version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Parses a version string (e.g., "1.2.3")
    pub fn parse(version_str: &str) -> Result<Self> {
        let parts: Vec<&str> = version_str.trim_start_matches('v').split('.').collect();

        if parts.len() < 3 {
            return Err(OrchestrationError::VersionConstraintViolation(format!(
                "Invalid version format: {}",
                version_str
            )));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| OrchestrationError::VersionConstraintViolation(format!(
                "Invalid major version: {}",
                parts[0]
            )))?;

        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| OrchestrationError::VersionConstraintViolation(format!(
                "Invalid minor version: {}",
                parts[1]
            )))?;

        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| OrchestrationError::VersionConstraintViolation(format!(
                "Invalid patch version: {}",
                parts[2]
            )))?;

        Ok(Version { major, minor, patch })
    }

    /// Converts version to string
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Represents a version constraint (e.g., "^1.2.3", "~1.2.3", ">=1.2.3")
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionConstraint {
    /// Exact version match
    Exact(Version),

    /// Caret constraint: ^1.2.3 allows >=1.2.3 and <2.0.0
    Caret(Version),

    /// Tilde constraint: ~1.2.3 allows >=1.2.3 and <1.3.0
    Tilde(Version),

    /// Greater than or equal
    GreaterOrEqual(Version),

    /// Less than
    Less(Version),

    /// Range constraint: >=1.0.0 <2.0.0
    Range(Version, Version),
}

impl VersionConstraint {
    /// Parses a version constraint string
    pub fn parse(constraint_str: &str) -> Result<Self> {
        let constraint_str = constraint_str.trim();

        // Check for range constraint first (before checking for >= or <)
        if constraint_str.contains(" <") {
            // Range constraint like ">=1.0.0 <2.0.0"
            let parts: Vec<&str> = constraint_str.split(" <").collect();
            if parts.len() != 2 {
                return Err(OrchestrationError::VersionConstraintViolation(format!(
                    "Invalid range constraint: {}",
                    constraint_str
                )));
            }

            let lower_part = parts[0].trim();
            let lower = if lower_part.starts_with(">=") {
                Version::parse(lower_part.strip_prefix(">=").unwrap())?
            } else if lower_part.starts_with(">") {
                Version::parse(lower_part.strip_prefix(">").unwrap())?
            } else {
                Version::parse(lower_part)?
            };
            let upper = Version::parse(parts[1].trim())?;
            return Ok(VersionConstraint::Range(lower, upper));
        }

        if constraint_str.starts_with('^') {
            let version = Version::parse(&constraint_str[1..])?;
            Ok(VersionConstraint::Caret(version))
        } else if constraint_str.starts_with('~') {
            let version = Version::parse(&constraint_str[1..])?;
            Ok(VersionConstraint::Tilde(version))
        } else if constraint_str.starts_with(">=") {
            let version = Version::parse(&constraint_str[2..])?;
            Ok(VersionConstraint::GreaterOrEqual(version))
        } else if constraint_str.starts_with('<') {
            let version = Version::parse(&constraint_str[1..])?;
            Ok(VersionConstraint::Less(version))
        } else {
            // Try to parse as exact version
            let version = Version::parse(constraint_str)?;
            Ok(VersionConstraint::Exact(version))
        }
    }

    /// Checks if a version satisfies this constraint
    pub fn is_satisfied_by(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Caret(v) => {
                // ^1.2.3 allows >=1.2.3 and <2.0.0
                version >= v && version.major == v.major
            }
            VersionConstraint::Tilde(v) => {
                // ~1.2.3 allows >=1.2.3 and <1.3.0
                version >= v && version.major == v.major && version.minor == v.minor
            }
            VersionConstraint::GreaterOrEqual(v) => version >= v,
            VersionConstraint::Less(v) => version < v,
            VersionConstraint::Range(lower, upper) => version >= lower && version < upper,
        }
    }

    /// Converts constraint to string
    pub fn to_string(&self) -> String {
        match self {
            VersionConstraint::Exact(v) => v.to_string(),
            VersionConstraint::Caret(v) => format!("^{}", v.to_string()),
            VersionConstraint::Tilde(v) => format!("~{}", v.to_string()),
            VersionConstraint::GreaterOrEqual(v) => format!(">={}", v.to_string()),
            VersionConstraint::Less(v) => format!("<{}", v.to_string()),
            VersionConstraint::Range(lower, upper) => {
                format!(">={} <{}", lower.to_string(), upper.to_string())
            }
        }
    }
}

/// Validates version compatibility between projects
#[derive(Debug, Clone)]
pub struct VersionValidator;

impl VersionValidator {
    /// Checks if a new version is compatible with a constraint
    pub fn is_compatible(constraint: &str, new_version: &str) -> Result<bool> {
        let constraint = VersionConstraint::parse(constraint)?;
        let version = Version::parse(new_version)?;

        Ok(constraint.is_satisfied_by(&version))
    }

    /// Validates that a new version doesn't break dependent projects
    pub fn validate_update(
        _current_version: &str,
        new_version: &str,
        dependent_constraints: &[&str],
    ) -> Result<bool> {
        let _new_ver = Version::parse(new_version)?;

        // Check if new version satisfies all dependent constraints
        for constraint_str in dependent_constraints {
            if !Self::is_compatible(constraint_str, new_version)? {
                return Err(OrchestrationError::VersionConstraintViolation(format!(
                    "New version {} does not satisfy constraint {}",
                    new_version, constraint_str
                )));
            }
        }

        Ok(true)
    }

    /// Checks if a version update is a breaking change
    pub fn is_breaking_change(old_version: &str, new_version: &str) -> Result<bool> {
        let old = Version::parse(old_version)?;
        let new = Version::parse(new_version)?;

        // Breaking change if major version changes
        Ok(old.major != new.major)
    }

    /// Finds compatible versions within a range
    pub fn find_compatible_versions(
        constraint: &str,
        available_versions: &[&str],
    ) -> Result<Vec<String>> {
        let constraint = VersionConstraint::parse(constraint)?;
        let mut compatible = Vec::new();

        for version_str in available_versions {
            if let Ok(version) = Version::parse(version_str) {
                if constraint.is_satisfied_by(&version) {
                    compatible.push(version_str.to_string());
                }
            }
        }

        Ok(compatible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_with_v_prefix() {
        let v = Version::parse("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.2.4").unwrap();
        let v3 = Version::parse("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_caret_constraint() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.2.3").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.2.4").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.3.0").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_tilde_constraint() {
        let constraint = VersionConstraint::parse("~1.2.3").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.2.3").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.2.4").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("1.3.0").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_greater_or_equal_constraint() {
        let constraint = VersionConstraint::parse(">=1.2.3").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.2.3").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.2.4").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_less_constraint() {
        let constraint = VersionConstraint::parse("<2.0.0").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.2.3").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.9.9").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("2.0.1").unwrap()));
    }

    #[test]
    fn test_range_constraint() {
        // Range constraints are parsed as ">=X.Y.Z <A.B.C"
        let constraint = VersionConstraint::parse(">=1.0.0 <2.0.0").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.0.0").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.5.0").unwrap()));
        assert!(constraint.is_satisfied_by(&Version::parse("1.9.9").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("0.9.9").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_exact_constraint() {
        let constraint = VersionConstraint::parse("1.2.3").unwrap();

        assert!(constraint.is_satisfied_by(&Version::parse("1.2.3").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("1.2.4").unwrap()));
        assert!(!constraint.is_satisfied_by(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_is_compatible() {
        assert!(VersionValidator::is_compatible("^1.2.3", "1.2.4").unwrap());
        assert!(VersionValidator::is_compatible("^1.2.3", "1.3.0").unwrap());
        assert!(!VersionValidator::is_compatible("^1.2.3", "2.0.0").unwrap());
    }

    #[test]
    fn test_validate_update() {
        let constraints = vec!["^1.0.0", "~1.2.0"];
        assert!(VersionValidator::validate_update("1.2.3", "1.2.4", &constraints).unwrap());
        assert!(VersionValidator::validate_update("1.2.3", "1.3.0", &constraints).is_err());
    }

    #[test]
    fn test_is_breaking_change() {
        assert!(!VersionValidator::is_breaking_change("1.2.3", "1.2.4").unwrap());
        assert!(!VersionValidator::is_breaking_change("1.2.3", "1.3.0").unwrap());
        assert!(VersionValidator::is_breaking_change("1.2.3", "2.0.0").unwrap());
    }

    #[test]
    fn test_find_compatible_versions() {
        let available = vec!["1.0.0", "1.2.3", "1.2.4", "1.3.0", "2.0.0"];
        let compatible = VersionValidator::find_compatible_versions("^1.2.3", &available).unwrap();

        // ^1.2.3 allows >=1.2.3 and <2.0.0, so 1.2.3, 1.2.4, and 1.3.0 are compatible
        assert_eq!(compatible.len(), 3);
        assert!(compatible.contains(&"1.2.3".to_string()));
        assert!(compatible.contains(&"1.2.4".to_string()));
        assert!(compatible.contains(&"1.3.0".to_string()));
    }
}
