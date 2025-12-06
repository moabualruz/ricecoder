//! Version coordination across dependent projects

use crate::error::{OrchestrationError, Result};
use crate::models::Project;
use crate::analyzers::{Version, VersionValidator, DependencyGraph};
use std::collections::{HashMap, HashSet};

/// Coordinates version updates across dependent projects
#[derive(Debug, Clone)]
pub struct VersionCoordinator {
    /// Dependency graph for tracking project relationships
    dependency_graph: DependencyGraph,

    /// Current versions of all projects
    project_versions: HashMap<String, String>,

    /// Version constraints for each project
    version_constraints: HashMap<String, Vec<String>>,
}

/// Result of a version update operation
#[derive(Debug, Clone)]
pub struct VersionUpdateResult {
    /// Project that was updated
    pub project: String,

    /// Old version
    pub old_version: String,

    /// New version
    pub new_version: String,

    /// Projects that need to be updated as a result
    pub affected_projects: Vec<String>,

    /// Whether the update was successful
    pub success: bool,

    /// Error message if update failed
    pub error: Option<String>,
}

/// Plan for coordinating version updates
#[derive(Debug, Clone)]
pub struct VersionUpdatePlan {
    /// Updates to be applied in order
    pub updates: Vec<VersionUpdateStep>,

    /// Total number of projects affected
    pub total_affected: usize,

    /// Whether the plan is valid
    pub is_valid: bool,

    /// Validation errors if any
    pub validation_errors: Vec<String>,
}

/// A single step in a version update plan
#[derive(Debug, Clone)]
pub struct VersionUpdateStep {
    /// Project to update
    pub project: String,

    /// New version to apply
    pub new_version: String,

    /// Projects that depend on this project
    pub dependents: Vec<String>,

    /// Whether this is a breaking change
    pub is_breaking: bool,
}

impl VersionCoordinator {
    /// Creates a new version coordinator
    pub fn new(dependency_graph: DependencyGraph) -> Self {
        Self {
            dependency_graph,
            project_versions: HashMap::new(),
            version_constraints: HashMap::new(),
        }
    }

    /// Registers a project with its current version
    pub fn register_project(&mut self, project: &Project) {
        self.project_versions.insert(project.name.clone(), project.version.clone());
    }

    /// Registers a dependency constraint
    pub fn register_constraint(&mut self, project: &str, constraint: String) {
        self.version_constraints
            .entry(project.to_string())
            .or_default()
            .push(constraint);
    }

    /// Updates a project version and propagates to dependents
    pub fn update_version(
        &mut self,
        project: &str,
        new_version: &str,
    ) -> Result<VersionUpdateResult> {
        // Validate the new version format
        Version::parse(new_version)?;

        // Get the old version
        let old_version = self
            .project_versions
            .get(project)
            .cloned()
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project.to_string()))?;

        // Check if this is a breaking change
        let _is_breaking = VersionValidator::is_breaking_change(&old_version, new_version)?;

        // Get all projects that depend on this one
        let dependents = self.dependency_graph.get_dependents(project);

        // Validate that the new version is compatible with all dependent constraints
        if let Some(constraints) = self.version_constraints.get(project) {
            VersionValidator::validate_update(&old_version, new_version, 
                &constraints.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
        }

        // Update the version
        self.project_versions.insert(project.to_string(), new_version.to_string());

        Ok(VersionUpdateResult {
            project: project.to_string(),
            old_version,
            new_version: new_version.to_string(),
            affected_projects: dependents,
            success: true,
            error: None,
        })
    }

    /// Creates a plan for coordinating version updates across projects
    pub fn plan_version_updates(
        &self,
        updates: Vec<(String, String)>,
    ) -> Result<VersionUpdatePlan> {
        let mut plan = VersionUpdatePlan {
            updates: Vec::new(),
            total_affected: 0,
            is_valid: true,
            validation_errors: Vec::new(),
        };

        let mut affected_projects = HashSet::new();
        let mut processed = HashSet::new();

        for (project, new_version) in updates {
            // Validate version format
            if let Err(e) = Version::parse(&new_version) {
                plan.is_valid = false;
                plan.validation_errors.push(format!(
                    "Invalid version for {}: {}",
                    project, e
                ));
                continue;
            }

            // Get current version
            let old_version = match self.project_versions.get(&project) {
                Some(v) => v.clone(),
                None => {
                    plan.is_valid = false;
                    plan.validation_errors.push(format!("Project not found: {}", project));
                    continue;
                }
            };

            // Check if breaking change
            let is_breaking = match VersionValidator::is_breaking_change(&old_version, &new_version) {
                Ok(b) => b,
                Err(e) => {
                    plan.is_valid = false;
                    plan.validation_errors.push(format!(
                        "Failed to check breaking change for {}: {}",
                        project, e
                    ));
                    continue;
                }
            };

            // Get dependents
            let dependents = self.dependency_graph.get_dependents(&project);
            affected_projects.extend(dependents.clone());

            // Create update step
            plan.updates.push(VersionUpdateStep {
                project: project.clone(),
                new_version,
                dependents,
                is_breaking,
            });

            processed.insert(project);
        }

        plan.total_affected = affected_projects.len();
        Ok(plan)
    }

    /// Gets all projects that need to be updated due to a change in the given project
    pub fn get_affected_projects(&self, project: &str) -> Vec<String> {
        self.dependency_graph.get_dependents(project)
    }

    /// Validates that a version update maintains all constraints
    pub fn validate_version_update(
        &self,
        project: &str,
        new_version: &str,
    ) -> Result<bool> {
        // Get current version
        let current_version = self
            .project_versions
            .get(project)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project.to_string()))?;

        // Get all constraints for this project
        let constraints = self
            .version_constraints
            .get(project)
            .map(|c| c.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        // Validate the update
        VersionValidator::validate_update(current_version, new_version, &constraints)
    }

    /// Gets the current version of a project
    pub fn get_version(&self, project: &str) -> Option<String> {
        self.project_versions.get(project).cloned()
    }

    /// Gets all version constraints for a project
    pub fn get_constraints(&self, project: &str) -> Vec<String> {
        self.version_constraints
            .get(project)
            .cloned()
            .unwrap_or_default()
    }

    /// Checks if a version update would be a breaking change
    pub fn is_breaking_change(&self, project: &str, new_version: &str) -> Result<bool> {
        let current_version = self
            .project_versions
            .get(project)
            .ok_or_else(|| OrchestrationError::ProjectNotFound(project.to_string()))?;

        VersionValidator::is_breaking_change(current_version, new_version)
    }

    /// Gets the dependency graph
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Gets all registered projects
    pub fn get_all_projects(&self) -> Vec<String> {
        self.project_versions.keys().cloned().collect()
    }

    /// Clears all registered projects and constraints
    pub fn clear(&mut self) {
        self.project_versions.clear();
        self.version_constraints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_coordinator() -> VersionCoordinator {
        let graph = DependencyGraph::new(false);
        VersionCoordinator::new(graph)
    }

    #[test]
    fn test_version_coordinator_creation() {
        let coordinator = create_test_coordinator();
        assert_eq!(coordinator.get_all_projects().len(), 0);
    }

    #[test]
    fn test_register_project() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);
        assert_eq!(coordinator.get_version("test-project"), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_register_constraint() {
        let mut coordinator = create_test_coordinator();
        coordinator.register_constraint("test-project", "^1.0.0".to_string());

        let constraints = coordinator.get_constraints("test-project");
        assert_eq!(constraints.len(), 1);
        assert_eq!(constraints[0], "^1.0.0");
    }

    #[test]
    fn test_update_version_success() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);
        let result = coordinator.update_version("test-project", "1.1.0").unwrap();

        assert!(result.success);
        assert_eq!(result.old_version, "1.0.0");
        assert_eq!(result.new_version, "1.1.0");
        assert_eq!(coordinator.get_version("test-project"), Some("1.1.0".to_string()));
    }

    #[test]
    fn test_update_version_invalid_format() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);
        let result = coordinator.update_version("test-project", "invalid");

        assert!(result.is_err());
    }

    #[test]
    fn test_update_version_not_found() {
        let mut coordinator = create_test_coordinator();
        let result = coordinator.update_version("nonexistent", "1.0.0");

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_version_update() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);
        coordinator.register_constraint("test-project", "^1.0.0".to_string());

        // Valid update
        assert!(coordinator.validate_version_update("test-project", "1.1.0").unwrap());

        // Invalid update (breaks constraint)
        assert!(coordinator.validate_version_update("test-project", "2.0.0").is_err());
    }

    #[test]
    fn test_is_breaking_change() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);

        // Minor version change is not breaking
        assert!(!coordinator.is_breaking_change("test-project", "1.1.0").unwrap());

        // Major version change is breaking
        assert!(coordinator.is_breaking_change("test-project", "2.0.0").unwrap());
    }

    #[test]
    fn test_plan_version_updates() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);

        let updates = vec![("test-project".to_string(), "1.1.0".to_string())];
        let plan = coordinator.plan_version_updates(updates).unwrap();

        assert!(plan.is_valid);
        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].project, "test-project");
        assert_eq!(plan.updates[0].new_version, "1.1.0");
    }

    #[test]
    fn test_plan_version_updates_invalid_version() {
        let coordinator = create_test_coordinator();
        let updates = vec![("test-project".to_string(), "invalid".to_string())];
        let plan = coordinator.plan_version_updates(updates).unwrap();

        assert!(!plan.is_valid);
        assert!(!plan.validation_errors.is_empty());
    }

    #[test]
    fn test_plan_version_updates_missing_project() {
        let coordinator = create_test_coordinator();
        let updates = vec![("nonexistent".to_string(), "1.0.0".to_string())];
        let plan = coordinator.plan_version_updates(updates).unwrap();

        assert!(!plan.is_valid);
        assert!(!plan.validation_errors.is_empty());
    }

    #[test]
    fn test_get_affected_projects() {
        let coordinator = create_test_coordinator();
        let affected = coordinator.get_affected_projects("test-project");
        assert_eq!(affected.len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut coordinator = create_test_coordinator();
        let project = Project {
            path: std::path::PathBuf::from("/path/to/project"),
            name: "test-project".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project);
        coordinator.register_constraint("test-project", "^1.0.0".to_string());

        assert_eq!(coordinator.get_all_projects().len(), 1);

        coordinator.clear();
        assert_eq!(coordinator.get_all_projects().len(), 0);
        assert_eq!(coordinator.get_constraints("test-project").len(), 0);
    }

    #[test]
    fn test_multiple_projects() {
        let mut coordinator = create_test_coordinator();

        let project1 = Project {
            path: std::path::PathBuf::from("/path/to/project1"),
            name: "project1".to_string(),
            project_type: "rust".to_string(),
            version: "1.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        let project2 = Project {
            path: std::path::PathBuf::from("/path/to/project2"),
            name: "project2".to_string(),
            project_type: "rust".to_string(),
            version: "2.0.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        };

        coordinator.register_project(&project1);
        coordinator.register_project(&project2);

        assert_eq!(coordinator.get_all_projects().len(), 2);
        assert_eq!(coordinator.get_version("project1"), Some("1.0.0".to_string()));
        assert_eq!(coordinator.get_version("project2"), Some("2.0.0".to_string()));
    }

    #[test]
    fn test_version_update_step_creation() {
        let step = VersionUpdateStep {
            project: "test-project".to_string(),
            new_version: "1.1.0".to_string(),
            dependents: vec!["dependent1".to_string()],
            is_breaking: false,
        };

        assert_eq!(step.project, "test-project");
        assert_eq!(step.new_version, "1.1.0");
        assert_eq!(step.dependents.len(), 1);
        assert!(!step.is_breaking);
    }

    #[test]
    fn test_version_update_result_creation() {
        let result = VersionUpdateResult {
            project: "test-project".to_string(),
            old_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
            affected_projects: vec!["dependent1".to_string()],
            success: true,
            error: None,
        };

        assert_eq!(result.project, "test-project");
        assert_eq!(result.old_version, "1.0.0");
        assert_eq!(result.new_version, "1.1.0");
        assert!(result.success);
        assert!(result.error.is_none());
    }
}
