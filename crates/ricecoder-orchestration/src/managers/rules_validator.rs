//! Rules validation for workspace orchestration
//!
//! Validates workspace rules and compliance across all projects.

use std::collections::{HashMap, HashSet};

use crate::{
    error::Result,
    models::{Project, ProjectDependency, RuleType, Workspace, WorkspaceRule},
};

/// Validates workspace rules and compliance
#[derive(Debug, Clone)]
pub struct RulesValidator {
    /// Workspace being validated
    workspace: Workspace,
}

/// Result of rules validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,

    /// Violations found
    pub violations: Vec<RuleViolation>,

    /// Warnings
    pub warnings: Vec<String>,
}

/// A rule violation
#[derive(Debug, Clone)]
pub struct RuleViolation {
    /// Name of the violated rule
    pub rule_name: String,

    /// Type of rule
    pub rule_type: RuleType,

    /// Description of the violation
    pub description: String,

    /// Projects involved in the violation
    pub affected_projects: Vec<String>,

    /// Severity level
    pub severity: ViolationSeverity,
}

/// Severity level of a rule violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Warning level
    Warning,

    /// Error level
    Error,

    /// Critical level
    Critical,
}

impl RulesValidator {
    /// Creates a new rules validator
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }

    /// Validates all workspace rules
    pub fn validate_all(&self) -> Result<ValidationResult> {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        for rule in &self.workspace.config.rules {
            if !rule.enabled {
                continue;
            }

            match rule.rule_type {
                RuleType::DependencyConstraint => {
                    if let Err(e) = self.validate_dependency_constraints(rule, &mut violations) {
                        warnings.push(format!("Error validating dependency constraints: {}", e));
                    }
                }
                RuleType::NamingConvention => {
                    if let Err(e) = self.validate_naming_conventions(rule, &mut violations) {
                        warnings.push(format!("Error validating naming conventions: {}", e));
                    }
                }
                RuleType::ArchitecturalBoundary => {
                    if let Err(e) = self.validate_architectural_boundaries(rule, &mut violations) {
                        warnings.push(format!("Error validating architectural boundaries: {}", e));
                    }
                }
            }
        }

        let passed = violations
            .iter()
            .all(|v| v.severity == ViolationSeverity::Warning);

        Ok(ValidationResult {
            passed,
            violations,
            warnings,
        })
    }

    /// Validates dependency constraint rules
    fn validate_dependency_constraints(
        &self,
        rule: &WorkspaceRule,
        violations: &mut Vec<RuleViolation>,
    ) -> Result<()> {
        // Check for circular dependencies
        if rule.name == "no-circular-deps" {
            let circular_deps = self.find_circular_dependencies();
            for (projects, cycle) in circular_deps {
                violations.push(RuleViolation {
                    rule_name: rule.name.clone(),
                    rule_type: rule.rule_type,
                    description: format!("Circular dependency detected: {}", cycle),
                    affected_projects: projects,
                    severity: ViolationSeverity::Error,
                });
            }
        }

        Ok(())
    }

    /// Validates naming convention rules
    fn validate_naming_conventions(
        &self,
        rule: &WorkspaceRule,
        violations: &mut Vec<RuleViolation>,
    ) -> Result<()> {
        // Check project naming conventions
        if rule.name == "naming-convention" {
            for project in &self.workspace.projects {
                // Project names should be lowercase with hyphens
                if !self.is_valid_project_name(&project.name) {
                    violations.push(RuleViolation {
                        rule_name: rule.name.clone(),
                        rule_type: rule.rule_type,
                        description: format!(
                            "Project name '{}' does not follow naming convention (should be lowercase with hyphens)",
                            project.name
                        ),
                        affected_projects: vec![project.name.clone()],
                        severity: ViolationSeverity::Warning,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validates architectural boundary rules
    fn validate_architectural_boundaries(
        &self,
        rule: &WorkspaceRule,
        violations: &mut Vec<RuleViolation>,
    ) -> Result<()> {
        // Check for cross-layer dependencies
        if rule.name == "no-cross-layer-deps" {
            let cross_layer_deps = self.find_cross_layer_dependencies();
            for (from, to) in cross_layer_deps {
                violations.push(RuleViolation {
                    rule_name: rule.name.clone(),
                    rule_type: rule.rule_type,
                    description: format!(
                        "Cross-layer dependency detected: {} depends on {}",
                        from, to
                    ),
                    affected_projects: vec![from, to],
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        Ok(())
    }

    /// Finds circular dependencies in the workspace
    fn find_circular_dependencies(&self) -> Vec<(Vec<String>, String)> {
        let mut circular_deps = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for project in &self.workspace.projects {
            if !visited.contains(&project.name) {
                if let Some(cycle) = self.find_cycle(&project.name, &mut visited, &mut rec_stack) {
                    circular_deps.push((cycle.clone(), cycle.join(" -> ")));
                }
            }
        }

        circular_deps
    }

    /// Finds a cycle starting from a project
    fn find_cycle(
        &self,
        project: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        visited.insert(project.to_string());
        rec_stack.insert(project.to_string());

        // Get dependencies of this project
        let deps: Vec<String> = self
            .workspace
            .dependencies
            .iter()
            .filter(|d| d.from == project)
            .map(|d| d.to.clone())
            .collect();

        for dep in deps {
            if !visited.contains(&dep) {
                if let Some(mut cycle) = self.find_cycle(&dep, visited, rec_stack) {
                    cycle.insert(0, project.to_string());
                    return Some(cycle);
                }
            } else if rec_stack.contains(&dep) {
                return Some(vec![project.to_string(), dep]);
            }
        }

        rec_stack.remove(project);
        None
    }

    /// Checks if a project name follows naming conventions
    fn is_valid_project_name(&self, name: &str) -> bool {
        // Project names should be lowercase with hyphens
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// Finds cross-layer dependencies
    fn find_cross_layer_dependencies(&self) -> Vec<(String, String)> {
        let mut cross_layer_deps = Vec::new();

        // Determine project layers based on naming conventions
        let layers = self.determine_project_layers();

        for dep in &self.workspace.dependencies {
            let from_layer = layers.get(&dep.from).copied().unwrap_or(0);
            let to_layer = layers.get(&dep.to).copied().unwrap_or(0);

            // Cross-layer dependency if from_layer > to_layer (depends on lower layer)
            if from_layer > to_layer {
                cross_layer_deps.push((dep.from.clone(), dep.to.clone()));
            }
        }

        cross_layer_deps
    }

    /// Determines project layers based on naming conventions
    fn determine_project_layers(&self) -> HashMap<String, u32> {
        let mut layers = HashMap::new();

        for project in &self.workspace.projects {
            let layer = if project.name.starts_with("ricecoder-core") {
                0
            } else if project.name.starts_with("ricecoder-") {
                1
            } else {
                2
            };

            layers.insert(project.name.clone(), layer);
        }

        layers
    }

    /// Validates a specific project against all rules
    pub fn validate_project(&self, project: &Project) -> Result<ValidationResult> {
        let mut violations = Vec::new();
        let warnings = Vec::new();

        for rule in &self.workspace.config.rules {
            if !rule.enabled {
                continue;
            }

            if rule.rule_type == RuleType::NamingConvention
                && !self.is_valid_project_name(&project.name)
            {
                violations.push(RuleViolation {
                    rule_name: rule.name.clone(),
                    rule_type: rule.rule_type,
                    description: format!(
                        "Project name '{}' does not follow naming convention",
                        project.name
                    ),
                    affected_projects: vec![project.name.clone()],
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        let passed = violations
            .iter()
            .all(|v| v.severity == ViolationSeverity::Warning);

        Ok(ValidationResult {
            passed,
            violations,
            warnings,
        })
    }

    /// Validates a dependency against all rules
    pub fn validate_dependency(&self, dep: &ProjectDependency) -> Result<ValidationResult> {
        let mut violations = Vec::new();
        let warnings = Vec::new();

        for rule in &self.workspace.config.rules {
            if !rule.enabled {
                continue;
            }

            if rule.rule_type == RuleType::DependencyConstraint {
                // Check if dependency creates a cycle
                if self.would_create_cycle(&dep.from, &dep.to) {
                    violations.push(RuleViolation {
                        rule_name: rule.name.clone(),
                        rule_type: rule.rule_type,
                        description: format!(
                            "Dependency would create a cycle: {} -> {}",
                            dep.from, dep.to
                        ),
                        affected_projects: vec![dep.from.clone(), dep.to.clone()],
                        severity: ViolationSeverity::Error,
                    });
                }
            }
        }

        let passed = violations
            .iter()
            .all(|v| v.severity == ViolationSeverity::Warning);

        Ok(ValidationResult {
            passed,
            violations,
            warnings,
        })
    }

    /// Checks if adding a dependency would create a cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // Check if there's already a path from 'to' to 'from'
        self.has_path(to, from)
    }

    /// Checks if there's a path from one project to another
    fn has_path(&self, from: &str, to: &str) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        self.has_path_recursive(from, to, &mut visited)
    }

    /// Recursively checks if there's a path from one project to another
    fn has_path_recursive(&self, from: &str, to: &str, visited: &mut HashSet<String>) -> bool {
        if from == to {
            return true;
        }

        if visited.contains(from) {
            return false;
        }

        visited.insert(from.to_string());

        for dep in &self.workspace.dependencies {
            if dep.from == from && self.has_path_recursive(&dep.to, to, visited) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ProjectStatus, WorkspaceConfig, WorkspaceMetrics};

    fn create_test_workspace() -> Workspace {
        Workspace {
            root: std::path::PathBuf::from("/workspace"),
            projects: vec![
                Project {
                    path: std::path::PathBuf::from("/workspace/project-a"),
                    name: "project-a".to_string(),
                    project_type: "rust".to_string(),
                    version: "0.1.0".to_string(),
                    status: ProjectStatus::Healthy,
                },
                Project {
                    path: std::path::PathBuf::from("/workspace/project-b"),
                    name: "project-b".to_string(),
                    project_type: "rust".to_string(),
                    version: "0.1.0".to_string(),
                    status: ProjectStatus::Healthy,
                },
            ],
            dependencies: vec![],
            config: WorkspaceConfig::default(),
            metrics: WorkspaceMetrics::default(),
        }
    }

    #[test]
    fn test_rules_validator_creation() {
        let workspace = create_test_workspace();
        let validator = RulesValidator::new(workspace);
        assert_eq!(validator.workspace.projects.len(), 2);
    }

    #[test]
    fn test_is_valid_project_name() {
        let workspace = create_test_workspace();
        let validator = RulesValidator::new(workspace);

        assert!(validator.is_valid_project_name("project-a"));
        assert!(validator.is_valid_project_name("my-project-123"));
        assert!(!validator.is_valid_project_name("Project-A"));
        assert!(!validator.is_valid_project_name("-project"));
        assert!(!validator.is_valid_project_name("project-"));
    }

    #[test]
    fn test_validate_naming_conventions() {
        let mut workspace = create_test_workspace();
        workspace.projects.push(Project {
            path: std::path::PathBuf::from("/workspace/InvalidProject"),
            name: "InvalidProject".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        });

        // Add naming convention rule to the workspace config
        workspace.config.rules.push(WorkspaceRule {
            name: "naming-convention".to_string(),
            rule_type: RuleType::NamingConvention,
            enabled: true,
        });

        let validator = RulesValidator::new(workspace);
        let result = validator.validate_all().unwrap();

        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_find_circular_dependencies() {
        let mut workspace = create_test_workspace();
        workspace.dependencies = vec![
            ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: crate::models::DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            },
            ProjectDependency {
                from: "project-b".to_string(),
                to: "project-a".to_string(),
                dependency_type: crate::models::DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            },
        ];

        let validator = RulesValidator::new(workspace);
        let circular_deps = validator.find_circular_dependencies();

        assert!(!circular_deps.is_empty());
    }

    #[test]
    fn test_would_create_cycle() {
        let mut workspace = create_test_workspace();
        workspace.dependencies = vec![ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: crate::models::DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        }];

        let validator = RulesValidator::new(workspace);

        // Adding project-b -> project-a would create a cycle
        assert!(validator.would_create_cycle("project-b", "project-a"));

        // Adding project-b -> project-c would not create a cycle
        assert!(!validator.would_create_cycle("project-b", "project-c"));
    }

    #[test]
    fn test_validate_project() {
        let workspace = create_test_workspace();
        let validator = RulesValidator::new(workspace);

        let project = Project {
            path: std::path::PathBuf::from("/workspace/project-a"),
            name: "project-a".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        };

        let result = validator.validate_project(&project).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_validate_dependency() {
        let workspace = create_test_workspace();
        let validator = RulesValidator::new(workspace);

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: crate::models::DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        let result = validator.validate_dependency(&dep).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_has_path() {
        let mut workspace = create_test_workspace();
        workspace.dependencies = vec![ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: crate::models::DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        }];

        let validator = RulesValidator::new(workspace);

        assert!(validator.has_path("project-a", "project-b"));
        assert!(!validator.has_path("project-b", "project-a"));
    }

    #[test]
    fn test_determine_project_layers() {
        let mut workspace = create_test_workspace();
        workspace.projects.push(Project {
            path: std::path::PathBuf::from("/workspace/ricecoder-core"),
            name: "ricecoder-core".to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        });

        let validator = RulesValidator::new(workspace);
        let layers = validator.determine_project_layers();

        assert_eq!(layers.get("ricecoder-core"), Some(&0));
        assert_eq!(layers.get("project-a"), Some(&2));
    }
}
