//! Impact analysis for changes across projects

use crate::error::Result;
use crate::models::{ImpactDetail, ImpactLevel, ImpactReport};
use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a change to a project
#[derive(Debug, Clone)]
pub struct ProjectChange {
    /// Identifier for the change
    pub change_id: String,

    /// Project that was changed
    pub project: String,

    /// Type of change (e.g., "api", "dependency", "config")
    pub change_type: String,

    /// Description of the change
    pub description: String,

    /// Whether this is a breaking change
    pub is_breaking: bool,
}

/// Analyzes the impact of changes on dependent projects
#[derive(Debug, Clone)]
pub struct ImpactAnalyzer {
    /// Adjacency list for forward dependencies (project -> dependents)
    dependents_map: HashMap<String, Vec<String>>,

    /// All projects in the workspace
    projects: HashSet<String>,
}

impl ImpactAnalyzer {
    /// Creates a new impact analyzer
    pub fn new() -> Self {
        Self {
            dependents_map: HashMap::new(),
            projects: HashSet::new(),
        }
    }

    /// Adds a project to the analyzer
    pub fn add_project(&mut self, project_name: String) {
        self.projects.insert(project_name.clone());
        self.dependents_map.entry(project_name).or_default();
    }

    /// Adds a dependency relationship (from depends on to)
    pub fn add_dependency(&mut self, from: String, to: String) {
        // Ensure both projects exist
        self.projects.insert(from.clone());
        self.projects.insert(to.clone());

        // Add to dependents map: if 'to' changes, 'from' is affected
        let dependents = self.dependents_map.entry(to).or_default();
        
        // Only add if not already present (avoid duplicates)
        if !dependents.contains(&from) {
            dependents.push(from);
        }
    }

    /// Analyzes the impact of a change on dependent projects
    pub fn analyze_impact(&self, change: &ProjectChange) -> Result<ImpactReport> {
        // Find all affected projects
        let affected_projects = self.find_affected_projects(&change.project);

        // Determine impact level based on change type and breaking status
        let impact_level = self.determine_impact_level(change);

        // Generate detailed impact information
        let details = self.generate_impact_details(change, &affected_projects);

        Ok(ImpactReport {
            change_id: change.change_id.clone(),
            affected_projects: affected_projects.clone(),
            impact_level,
            details,
        })
    }

    /// Finds all projects affected by a change to a specific project
    fn find_affected_projects(&self, changed_project: &str) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(changed_project.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Find all projects that depend on the current project
            if let Some(dependents) = self.dependents_map.get(&current) {
                for dependent in dependents {
                    if !visited.contains(dependent) {
                        affected.insert(dependent.clone());
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        affected.into_iter().collect()
    }

    /// Determines the impact level of a change
    fn determine_impact_level(&self, change: &ProjectChange) -> ImpactLevel {
        match (change.change_type.as_str(), change.is_breaking) {
            // Breaking API changes have critical impact
            ("api", true) => ImpactLevel::Critical,
            // Breaking dependency changes have high impact
            ("dependency", true) => ImpactLevel::High,
            // Breaking config changes have high impact
            ("config", true) => ImpactLevel::High,
            // Breaking other changes have high impact
            (_, true) => ImpactLevel::High,
            // Non-breaking API changes have medium impact
            ("api", false) => ImpactLevel::Medium,
            // Dependency changes have medium impact
            ("dependency", false) => ImpactLevel::Medium,
            // Config changes have low impact
            ("config", false) => ImpactLevel::Low,
            // Other changes have low impact
            _ => ImpactLevel::Low,
        }
    }

    /// Generates detailed impact information for affected projects
    fn generate_impact_details(
        &self,
        change: &ProjectChange,
        affected_projects: &[String],
    ) -> Vec<ImpactDetail> {
        affected_projects
            .iter()
            .map(|project| {
                let reason = self.generate_impact_reason(change);
                let required_actions = self.generate_required_actions(change);

                ImpactDetail {
                    project: project.clone(),
                    reason,
                    required_actions,
                }
            })
            .collect()
    }

    /// Generates a reason for the impact
    fn generate_impact_reason(&self, change: &ProjectChange) -> String {
        if change.is_breaking {
            format!(
                "Breaking {} change in {}: {}",
                change.change_type, change.project, change.description
            )
        } else {
            format!(
                "Non-breaking {} change in {}: {}",
                change.change_type, change.project, change.description
            )
        }
    }

    /// Generates required actions to address the impact
    fn generate_required_actions(&self, change: &ProjectChange) -> Vec<String> {
        let mut actions = vec!["Review the change".to_string()];

        match change.change_type.as_str() {
            "api" => {
                actions.push("Update API usage".to_string());
                if change.is_breaking {
                    actions.push("Update imports and function calls".to_string());
                }
            }
            "dependency" => {
                actions.push("Update dependency version".to_string());
                if change.is_breaking {
                    actions.push("Review breaking changes in dependency".to_string());
                }
            }
            "config" => {
                actions.push("Update configuration".to_string());
            }
            _ => {
                actions.push("Verify compatibility".to_string());
            }
        }

        actions.push("Run tests".to_string());

        actions
    }

    /// Analyzes impact for multiple changes
    pub fn analyze_multiple_impacts(
        &self,
        changes: &[ProjectChange],
    ) -> Result<Vec<ImpactReport>> {
        changes
            .iter()
            .map(|change| self.analyze_impact(change))
            .collect()
    }

    /// Gets all projects that would be affected by a change to a specific project
    pub fn get_affected_projects(&self, project: &str) -> Vec<String> {
        self.find_affected_projects(project)
    }

    /// Gets the number of projects affected by a change
    pub fn count_affected_projects(&self, project: &str) -> usize {
        self.find_affected_projects(project).len()
    }

    /// Checks if a project would be affected by a change
    pub fn is_affected(&self, changed_project: &str, target_project: &str) -> bool {
        self.find_affected_projects(changed_project)
            .contains(&target_project.to_string())
    }

    /// Gets all projects in the analyzer
    pub fn get_projects(&self) -> Vec<String> {
        self.projects.iter().cloned().collect()
    }

    /// Gets the dependency map
    pub fn get_dependents_map(&self) -> &HashMap<String, Vec<String>> {
        &self.dependents_map
    }

    /// Clears all data from the analyzer
    pub fn clear(&mut self) {
        self.dependents_map.clear();
        self.projects.clear();
    }
}

impl Default for ImpactAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_analyzer() {
        let analyzer = ImpactAnalyzer::new();
        assert_eq!(analyzer.get_projects().len(), 0);
    }

    #[test]
    fn test_add_project() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());

        assert_eq!(analyzer.get_projects().len(), 1);
        assert!(analyzer.get_projects().contains(&"project-a".to_string()));
    }

    #[test]
    fn test_add_dependency() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let affected = analyzer.get_affected_projects("project-b");
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&"project-a".to_string()));
    }

    #[test]
    fn test_analyze_breaking_api_change() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: "api".to_string(),
            description: "Removed deprecated function".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.change_id, "change-1");
        assert_eq!(report.affected_projects.len(), 1);
        assert_eq!(report.impact_level, ImpactLevel::Critical);
        assert!(report.details[0].required_actions.len() > 0);
    }

    #[test]
    fn test_analyze_non_breaking_api_change() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: "api".to_string(),
            description: "Added new function".to_string(),
            is_breaking: false,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.affected_projects.len(), 1);
        assert_eq!(report.impact_level, ImpactLevel::Medium);
    }

    #[test]
    fn test_analyze_config_change() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: "config".to_string(),
            description: "Updated configuration".to_string(),
            is_breaking: false,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.affected_projects.len(), 1);
        assert_eq!(report.impact_level, ImpactLevel::Low);
    }

    #[test]
    fn test_transitive_impact() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_project("project-c".to_string());

        // A depends on B, B depends on C
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());
        analyzer.add_dependency("project-b".to_string(), "project-c".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-c".to_string(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        // Both A and B should be affected
        assert_eq!(report.affected_projects.len(), 2);
        assert!(report.affected_projects.contains(&"project-a".to_string()));
        assert!(report.affected_projects.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_no_affected_projects() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.affected_projects.len(), 0);
    }

    #[test]
    fn test_is_affected() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        assert!(analyzer.is_affected("project-b", "project-a"));
        assert!(!analyzer.is_affected("project-a", "project-b"));
    }

    #[test]
    fn test_count_affected_projects() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_project("project-c".to_string());

        analyzer.add_dependency("project-a".to_string(), "project-c".to_string());
        analyzer.add_dependency("project-b".to_string(), "project-c".to_string());

        assert_eq!(analyzer.count_affected_projects("project-c"), 2);
    }

    #[test]
    fn test_analyze_multiple_impacts() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_project("project-c".to_string());

        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());
        analyzer.add_dependency("project-b".to_string(), "project-c".to_string());

        let changes = vec![
            ProjectChange {
                change_id: "change-1".to_string(),
                project: "project-b".to_string(),
                change_type: "api".to_string(),
                description: "API change".to_string(),
                is_breaking: true,
            },
            ProjectChange {
                change_id: "change-2".to_string(),
                project: "project-c".to_string(),
                change_type: "config".to_string(),
                description: "Config change".to_string(),
                is_breaking: false,
            },
        ];

        let reports = analyzer.analyze_multiple_impacts(&changes).unwrap();

        assert_eq!(reports.len(), 2);
        // Change to B affects A (1 project)
        assert_eq!(reports[0].affected_projects.len(), 1);
        // Change to C affects B and A (2 projects)
        assert_eq!(reports[1].affected_projects.len(), 2);
    }

    #[test]
    fn test_clear_analyzer() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());

        assert_eq!(analyzer.get_projects().len(), 2);

        analyzer.clear();

        assert_eq!(analyzer.get_projects().len(), 0);
    }

    #[test]
    fn test_impact_detail_generation() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: "api".to_string(),
            description: "Removed function".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.details.len(), 1);
        assert_eq!(report.details[0].project, "project-a");
        assert!(report.details[0].reason.contains("Breaking"));
        assert!(report.details[0].required_actions.len() > 0);
    }

    #[test]
    fn test_dependency_change_impact() {
        let mut analyzer = ImpactAnalyzer::new();
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: "dependency".to_string(),
            description: "Updated dependency".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        assert_eq!(report.impact_level, ImpactLevel::High);
    }

    #[test]
    fn test_complex_dependency_graph() {
        let mut analyzer = ImpactAnalyzer::new();

        // Create a diamond dependency graph
        // A -> B -> D
        // A -> C -> D
        analyzer.add_project("project-a".to_string());
        analyzer.add_project("project-b".to_string());
        analyzer.add_project("project-c".to_string());
        analyzer.add_project("project-d".to_string());

        analyzer.add_dependency("project-a".to_string(), "project-b".to_string());
        analyzer.add_dependency("project-a".to_string(), "project-c".to_string());
        analyzer.add_dependency("project-b".to_string(), "project-d".to_string());
        analyzer.add_dependency("project-c".to_string(), "project-d".to_string());

        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: "project-d".to_string(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).unwrap();

        // A, B, and C should be affected (A depends on B and C, B and C depend on D)
        assert_eq!(report.affected_projects.len(), 3);
        assert!(report.affected_projects.contains(&"project-a".to_string()));
        assert!(report.affected_projects.contains(&"project-b".to_string()));
        assert!(report.affected_projects.contains(&"project-c".to_string()));
    }
}
