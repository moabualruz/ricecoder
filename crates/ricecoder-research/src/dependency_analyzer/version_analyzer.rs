//! Version analysis for dependencies

use super::{VersionConflict, VersionUpdate};
use crate::models::Dependency;

/// Analyzes dependency versions for conflicts and updates
#[derive(Debug)]
pub struct VersionAnalyzer;

impl VersionAnalyzer {
    /// Creates a new VersionAnalyzer
    pub fn new() -> Self {
        VersionAnalyzer
    }

    /// Finds version conflicts in dependencies
    pub fn find_conflicts(&self, dependencies: &[Dependency]) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();
        let mut version_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        // Group dependencies by name
        for dep in dependencies {
            version_map
                .entry(dep.name.clone())
                .or_default()
                .push(dep.version.clone());
        }

        // Find conflicts (same dependency with different versions)
        for (name, versions) in version_map {
            if versions.len() > 1 {
                // Remove duplicates and check if there are actual conflicts
                let mut unique_versions = versions;
                unique_versions.sort();
                unique_versions.dedup();

                if unique_versions.len() > 1 {
                    conflicts.push(VersionConflict {
                        dependency_name: name,
                        versions: unique_versions.clone(),
                        description: format!(
                            "Multiple versions found: {}",
                            unique_versions.join(", ")
                        ),
                    });
                }
            }
        }

        conflicts
    }

    /// Suggests version updates for dependencies
    pub fn suggest_updates(&self, _dependencies: &[Dependency]) -> Vec<VersionUpdate> {
        // This is a placeholder implementation
        // In a real implementation, this would check against package registries
        // for newer versions and suggest updates
        Vec::new()
    }
}

impl Default for VersionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_analyzer_creation() {
        let analyzer = VersionAnalyzer::new();
        assert!(true);
    }

    #[test]
    fn test_version_analyzer_no_conflicts() {
        let analyzer = VersionAnalyzer::new();
        let deps = vec![
            Dependency {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                constraints: None,
                is_dev: false,
            },
            Dependency {
                name: "tokio".to_string(),
                version: "1.0.0".to_string(),
                constraints: None,
                is_dev: false,
            },
        ];

        let conflicts = analyzer.find_conflicts(&deps);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_version_analyzer_finds_conflicts() {
        let analyzer = VersionAnalyzer::new();
        let deps = vec![
            Dependency {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                constraints: None,
                is_dev: false,
            },
            Dependency {
                name: "serde".to_string(),
                version: "2.0.0".to_string(),
                constraints: None,
                is_dev: false,
            },
        ];

        let conflicts = analyzer.find_conflicts(&deps);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].dependency_name, "serde");
        assert_eq!(conflicts[0].versions.len(), 2);
    }

    #[test]
    fn test_version_analyzer_default() {
        let analyzer = VersionAnalyzer::default();
        assert!(true);
    }
}
