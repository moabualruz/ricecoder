//! Dependency validation and compatibility checking

use crate::error::{OrchestrationError, Result};
use crate::models::{Project, ProjectDependency};
use std::collections::HashMap;

use super::version_validator::VersionValidator;

/// Validates dependency compatibility and version constraints
#[derive(Debug, Clone)]
pub struct DependencyValidator {
    /// Map of project names to their versions
    project_versions: HashMap<String, String>,

    /// Map of project names to their dependencies
    dependencies: HashMap<String, Vec<ProjectDependency>>,
}

impl DependencyValidator {
    /// Creates a new dependency validator
    pub fn new() -> Self {
        Self {
            project_versions: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Registers a project with its version
    pub fn register_project(&mut self, project: &Project) {
        self.project_versions
            .insert(project.name.clone(), project.version.clone());
    }

    /// Registers a dependency
    pub fn register_dependency(&mut self, dependency: ProjectDependency) {
        self.dependencies
            .entry(dependency.from.clone())
            .or_insert_with(Vec::new)
            .push(dependency);
    }

    /// Validates that all dependencies have compatible versions
    pub fn validate_all_dependencies(&self) -> Result<()> {
        for (project_name, deps) in &self.dependencies {
            for dep in deps {
                self.validate_single_dependency(project_name, dep)?;
            }
        }
        Ok(())
    }

    /// Validates a single dependency
    pub fn validate_single_dependency(
        &self,
        _from_project: &str,
        dependency: &ProjectDependency,
    ) -> Result<()> {
        // Check if the target project exists
        let target_version = self
            .project_versions
            .get(&dependency.to)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(dependency.to.clone()))?;

        // Validate version constraint
        let is_compatible = VersionValidator::is_compatible(&dependency.version_constraint, target_version)?;
        
        if !is_compatible {
            return Err(OrchestrationError::DependencyValidationFailed(format!(
                "Version {} does not satisfy constraint {}",
                target_version, dependency.version_constraint
            )));
        }

        Ok(())
    }

    /// Validates a version update for a project
    pub fn validate_version_update(
        &self,
        project_name: &str,
        new_version: &str,
    ) -> Result<()> {
        // Get all projects that depend on this project
        let dependents = self.get_dependents(project_name);

        if dependents.is_empty() {
            // No dependents, update is always valid
            return Ok(());
        }

        // Collect all constraints from dependents
        let mut constraints = Vec::new();
        for dependent_name in dependents {
            if let Some(deps) = self.dependencies.get(&dependent_name) {
                for dep in deps {
                    if dep.to == project_name {
                        constraints.push(dep.version_constraint.clone());
                    }
                }
            }
        }

        // Validate the new version against all constraints
        let constraint_strs: Vec<&str> = constraints.iter().map(|s| s.as_str()).collect();

        let current_version = self
            .project_versions
            .get(project_name)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project_name.to_string()))?;

        VersionValidator::validate_update(current_version, new_version, &constraint_strs)?;

        Ok(())
    }

    /// Checks if a version update is a breaking change
    pub fn is_breaking_change(&self, project_name: &str, new_version: &str) -> Result<bool> {
        let current_version = self
            .project_versions
            .get(project_name)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project_name.to_string()))?;

        VersionValidator::is_breaking_change(current_version, new_version)
    }

    /// Gets all projects that depend on a given project
    pub fn get_dependents(&self, project_name: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        for (from, deps) in &self.dependencies {
            for dep in deps {
                if dep.to == project_name {
                    dependents.push(from.clone());
                }
            }
        }

        dependents
    }

    /// Gets all projects that a given project depends on
    pub fn get_dependencies(&self, project_name: &str) -> Vec<ProjectDependency> {
        self.dependencies
            .get(project_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Validates that a new version doesn't break any dependents
    pub fn validate_no_breaking_changes(&self, project_name: &str, new_version: &str) -> Result<()> {
        if !self.is_breaking_change(project_name, new_version)? {
            return Ok(());
        }

        // Breaking change detected - check if any dependents use exact or restrictive constraints
        let dependents = self.get_dependents(project_name);

        for dependent_name in dependents {
            if let Some(deps) = self.dependencies.get(&dependent_name) {
                for dep in deps {
                    if dep.to == project_name {
                        // Check if the constraint would reject the new version
                        if !VersionValidator::is_compatible(&dep.version_constraint, new_version)? {
                            return Err(OrchestrationError::DependencyValidationFailed(format!(
                                "Breaking change in {} would break dependent project {} (constraint: {})",
                                project_name, dependent_name, dep.version_constraint
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Gets validation report for a project
    pub fn get_validation_report(&self, project_name: &str) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            project: project_name.to_string(),
            version: self
                .project_versions
                .get(project_name)
                .cloned()
                .unwrap_or_default(),
            dependencies: Vec::new(),
            dependents: Vec::new(),
            issues: Vec::new(),
        };

        // Add dependencies
        if let Some(deps) = self.dependencies.get(project_name) {
            for dep in deps {
                report.dependencies.push(DependencyInfo {
                    target: dep.to.clone(),
                    constraint: dep.version_constraint.clone(),
                    satisfied: self.validate_single_dependency(project_name, dep).is_ok(),
                });
            }
        }

        // Add dependents
        for dependent_name in self.get_dependents(project_name) {
            report.dependents.push(dependent_name);
        }

        // Validate and collect issues
        if let Err(e) = self.validate_all_dependencies() {
            report.issues.push(e.to_string());
        }

        Ok(report)
    }

    /// Clears all registered data
    pub fn clear(&mut self) {
        self.project_versions.clear();
        self.dependencies.clear();
    }
}

impl Default for DependencyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a dependency
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    /// Target project name
    pub target: String,

    /// Version constraint
    pub constraint: String,

    /// Whether the constraint is satisfied
    pub satisfied: bool,
}

/// Validation report for a project
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Project name
    pub project: String,

    /// Project version
    pub version: String,

    /// Dependencies of this project
    pub dependencies: Vec<DependencyInfo>,

    /// Projects that depend on this project
    pub dependents: Vec<String>,

    /// Validation issues
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DependencyType, ProjectStatus};
    use std::path::PathBuf;

    fn create_test_project(name: &str, version: &str) -> Project {
        Project {
            path: PathBuf::from(format!("/path/to/{}", name)),
            name: name.to_string(),
            project_type: "rust".to_string(),
            version: version.to_string(),
            status: ProjectStatus::Healthy,
        }
    }

    #[test]
    fn test_register_project() {
        let mut validator = DependencyValidator::new();
        let project = create_test_project("project-a", "1.2.3");

        validator.register_project(&project);

        assert_eq!(validator.project_versions.get("project-a"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn test_register_dependency() {
        let mut validator = DependencyValidator::new();
        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        };

        validator.register_dependency(dep);

        assert_eq!(validator.dependencies.get("project-a").unwrap().len(), 1);
    }

    #[test]
    fn test_validate_single_dependency_success() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.2.3"));

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        };

        let result = validator.validate_single_dependency("project-a", &dep);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_single_dependency_missing_target() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        };

        let result = validator.validate_single_dependency("project-a", &dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_single_dependency_incompatible_version() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "2.0.0"));

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        };

        let result = validator.validate_single_dependency("project-a", &dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_dependencies() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.2.3"));
        validator.register_project(&create_test_project("project-c", "1.5.0"));

        validator.register_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let result = validator.validate_all_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_version_update_compatible() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.2.3"));

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let result = validator.validate_version_update("project-a", "1.2.4");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_version_update_incompatible() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.2.3"));

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let result = validator.validate_version_update("project-a", "2.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_breaking_change() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));

        assert!(!validator.is_breaking_change("project-a", "1.2.3").unwrap());
        assert!(validator.is_breaking_change("project-a", "2.0.0").unwrap());
    }

    #[test]
    fn test_get_dependents() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.0.0"));
        validator.register_project(&create_test_project("project-c", "1.0.0"));

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        validator.register_dependency(ProjectDependency {
            from: "project-c".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let dependents = validator.get_dependents("project-a");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"project-b".to_string()));
        assert!(dependents.contains(&"project-c".to_string()));
    }

    #[test]
    fn test_get_dependencies() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.0.0"));

        validator.register_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let deps = validator.get_dependencies("project-a");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].to, "project-b");
    }

    #[test]
    fn test_validate_no_breaking_changes_non_breaking() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.0.0"));

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let result = validator.validate_no_breaking_changes("project-a", "1.2.3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_no_breaking_changes_breaking() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.0.0"));

        validator.register_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let result = validator.validate_no_breaking_changes("project-a", "2.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_validation_report() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));
        validator.register_project(&create_test_project("project-b", "1.2.3"));

        validator.register_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^1.0.0".to_string(),
        });

        let report = validator.get_validation_report("project-a").unwrap();
        assert_eq!(report.project, "project-a");
        assert_eq!(report.version, "1.0.0");
        assert_eq!(report.dependencies.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut validator = DependencyValidator::new();
        validator.register_project(&create_test_project("project-a", "1.0.0"));

        assert_eq!(validator.project_versions.len(), 1);

        validator.clear();

        assert_eq!(validator.project_versions.len(), 0);
    }
}
