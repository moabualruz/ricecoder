//! Change propagation tracking across projects

use crate::error::Result;
use crate::models::ImpactLevel;
use std::collections::{HashMap, HashSet};

/// Type of change that can occur in a project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// API change (function signatures, types, etc.)
    Api,

    /// Dependency change (version updates, new dependencies)
    Dependency,

    /// Configuration change
    Config,

    /// Internal implementation change
    Internal,

    /// Documentation change
    Documentation,

    /// Test change
    Test,

    /// Other type of change
    Other,
}

impl ChangeType {
    /// Converts a string to a ChangeType
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "api" => ChangeType::Api,
            "dependency" => ChangeType::Dependency,
            "config" => ChangeType::Config,
            "internal" => ChangeType::Internal,
            "documentation" => ChangeType::Documentation,
            "test" => ChangeType::Test,
            _ => ChangeType::Other,
        }
    }

    /// Converts to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Api => "api",
            ChangeType::Dependency => "dependency",
            ChangeType::Config => "config",
            ChangeType::Internal => "internal",
            ChangeType::Documentation => "documentation",
            ChangeType::Test => "test",
            ChangeType::Other => "other",
        }
    }
}

/// Represents a change to a project
#[derive(Debug, Clone)]
pub struct Change {
    /// Unique identifier for the change
    pub id: String,

    /// Project that was changed
    pub project: String,

    /// Type of change
    pub change_type: ChangeType,

    /// Description of the change
    pub description: String,

    /// Whether this is a breaking change
    pub is_breaking: bool,

    /// Impact level of the change
    pub impact_level: ImpactLevel,
}

/// Tracks propagation of changes through the dependency graph
#[derive(Debug, Clone)]
pub struct ChangePropagationTracker {
    /// Map of project to its dependents
    dependents_map: HashMap<String, Vec<String>>,

    /// Map of changes by project
    changes_by_project: HashMap<String, Vec<Change>>,

    /// Map of affected projects by change
    affected_by_change: HashMap<String, HashSet<String>>,

    /// All projects in the workspace
    projects: HashSet<String>,
}

impl ChangePropagationTracker {
    /// Creates a new change propagation tracker
    pub fn new() -> Self {
        Self {
            dependents_map: HashMap::new(),
            changes_by_project: HashMap::new(),
            affected_by_change: HashMap::new(),
            projects: HashSet::new(),
        }
    }

    /// Adds a project to the tracker
    pub fn add_project(&mut self, project_name: String) {
        self.projects.insert(project_name.clone());
        self.dependents_map.entry(project_name).or_default();
    }

    /// Adds a dependency relationship (from depends on to)
    pub fn add_dependency(&mut self, from: String, to: String) {
        self.projects.insert(from.clone());
        self.projects.insert(to.clone());

        self.dependents_map
            .entry(to)
            .or_default()
            .push(from);
    }

    /// Tracks a change to a project
    pub fn track_change(&mut self, change: Change) -> Result<()> {
        // Validate that the project exists
        if !self.projects.contains(&change.project) {
            self.projects.insert(change.project.clone());
        }

        // Find all affected projects
        let affected = self.find_affected_projects(&change.project);

        // Store the change
        self.changes_by_project
            .entry(change.project.clone())
            .or_default()
            .push(change.clone());

        // Store affected projects
        self.affected_by_change
            .insert(change.id.clone(), affected.into_iter().collect());

        Ok(())
    }

    /// Finds all projects affected by a change to a specific project
    fn find_affected_projects(&self, changed_project: &str) -> Vec<String> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![changed_project.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(dependents) = self.dependents_map.get(&current) {
                for dependent in dependents {
                    if !visited.contains(dependent) {
                        affected.push(dependent.clone());
                        queue.push(dependent.clone());
                    }
                }
            }
        }

        affected
    }

    /// Gets all changes for a project
    pub fn get_changes_for_project(&self, project: &str) -> Vec<Change> {
        self.changes_by_project
            .get(project)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets all changes of a specific type
    pub fn get_changes_by_type(&self, change_type: ChangeType) -> Vec<Change> {
        self.changes_by_project
            .values()
            .flat_map(|changes| {
                changes
                    .iter()
                    .filter(|c| c.change_type == change_type)
                    .cloned()
            })
            .collect()
    }

    /// Gets all breaking changes
    pub fn get_breaking_changes(&self) -> Vec<Change> {
        self.changes_by_project
            .values()
            .flat_map(|changes| {
                changes
                    .iter()
                    .filter(|c| c.is_breaking)
                    .cloned()
            })
            .collect()
    }

    /// Gets all projects affected by a change
    pub fn get_affected_projects(&self, change_id: &str) -> Vec<String> {
        self.affected_by_change
            .get(change_id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets the number of projects affected by a change
    pub fn count_affected_projects(&self, change_id: &str) -> usize {
        self.affected_by_change
            .get(change_id)
            .map(|set| set.len())
            .unwrap_or(0)
    }

    /// Checks if a project is affected by a change
    pub fn is_affected(&self, change_id: &str, project: &str) -> bool {
        self.affected_by_change
            .get(change_id)
            .map(|set| set.contains(project))
            .unwrap_or(false)
    }

    /// Gets all changes affecting a specific project
    pub fn get_changes_affecting_project(&self, project: &str) -> Vec<Change> {
        self.affected_by_change
            .iter()
            .filter(|(_, affected)| affected.contains(&project.to_string()))
            .filter_map(|(change_id, _)| {
                self.changes_by_project
                    .values()
                    .flat_map(|changes| changes.iter())
                    .find(|c| &c.id == change_id)
                    .cloned()
            })
            .collect()
    }

    /// Filters changes by type
    pub fn filter_changes_by_type(
        &self,
        changes: &[Change],
        change_type: ChangeType,
    ) -> Vec<Change> {
        changes
            .iter()
            .filter(|c| c.change_type == change_type)
            .cloned()
            .collect()
    }

    /// Filters changes by breaking status
    pub fn filter_changes_by_breaking(
        &self,
        changes: &[Change],
        is_breaking: bool,
    ) -> Vec<Change> {
        changes
            .iter()
            .filter(|c| c.is_breaking == is_breaking)
            .cloned()
            .collect()
    }

    /// Filters changes by impact level
    pub fn filter_changes_by_impact(
        &self,
        changes: &[Change],
        impact_level: ImpactLevel,
    ) -> Vec<Change> {
        changes
            .iter()
            .filter(|c| c.impact_level == impact_level)
            .cloned()
            .collect()
    }

    /// Gets detailed information about a change
    pub fn get_change_details(&self, change_id: &str) -> Option<ChangeDetails> {
        let change = self.changes_by_project
            .values()
            .flat_map(|changes| changes.iter())
            .find(|c| c.id == change_id)?
            .clone();

        let affected_projects = self.get_affected_projects(change_id);
        let affected_count = affected_projects.len();

        Some(ChangeDetails {
            change,
            affected_projects,
            affected_count,
        })
    }

    /// Gets all changes
    pub fn get_all_changes(&self) -> Vec<Change> {
        self.changes_by_project
            .values()
            .flat_map(|changes| changes.iter().cloned())
            .collect()
    }

    /// Gets all projects
    pub fn get_projects(&self) -> Vec<String> {
        self.projects.iter().cloned().collect()
    }

    /// Clears all data
    pub fn clear(&mut self) {
        self.dependents_map.clear();
        self.changes_by_project.clear();
        self.affected_by_change.clear();
        self.projects.clear();
    }
}

impl Default for ChangePropagationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed information about a change
#[derive(Debug, Clone)]
pub struct ChangeDetails {
    /// The change
    pub change: Change,

    /// Projects affected by the change
    pub affected_projects: Vec<String>,

    /// Number of affected projects
    pub affected_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_from_str() {
        assert_eq!(ChangeType::from_str("api"), ChangeType::Api);
        assert_eq!(ChangeType::from_str("dependency"), ChangeType::Dependency);
        assert_eq!(ChangeType::from_str("config"), ChangeType::Config);
        assert_eq!(ChangeType::from_str("internal"), ChangeType::Internal);
        assert_eq!(ChangeType::from_str("documentation"), ChangeType::Documentation);
        assert_eq!(ChangeType::from_str("test"), ChangeType::Test);
        assert_eq!(ChangeType::from_str("unknown"), ChangeType::Other);
    }

    #[test]
    fn test_change_type_as_str() {
        assert_eq!(ChangeType::Api.as_str(), "api");
        assert_eq!(ChangeType::Dependency.as_str(), "dependency");
        assert_eq!(ChangeType::Config.as_str(), "config");
    }

    #[test]
    fn test_create_tracker() {
        let tracker = ChangePropagationTracker::new();
        assert_eq!(tracker.get_projects().len(), 0);
    }

    #[test]
    fn test_add_project() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        assert_eq!(tracker.get_projects().len(), 1);
    }

    #[test]
    fn test_add_dependency() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());
        tracker.add_project("project-b".to_string());
        tracker.add_dependency("project-a".to_string(), "project-b".to_string());

        assert_eq!(tracker.get_projects().len(), 2);
    }

    #[test]
    fn test_track_change() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());
        tracker.add_project("project-b".to_string());
        tracker.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        let affected = tracker.get_affected_projects("change-1");
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&"project-a".to_string()));
    }

    #[test]
    fn test_get_changes_for_project() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        let changes = tracker.get_changes_for_project("project-a");
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn test_get_changes_by_type() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change1 = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        let change2 = Change {
            id: "change-2".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Config,
            description: "Config change".to_string(),
            is_breaking: false,
            impact_level: ImpactLevel::Low,
        };

        tracker.track_change(change1).unwrap();
        tracker.track_change(change2).unwrap();

        let api_changes = tracker.get_changes_by_type(ChangeType::Api);
        assert_eq!(api_changes.len(), 1);

        let config_changes = tracker.get_changes_by_type(ChangeType::Config);
        assert_eq!(config_changes.len(), 1);
    }

    #[test]
    fn test_get_breaking_changes() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change1 = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "Breaking API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        let change2 = Change {
            id: "change-2".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "Non-breaking API change".to_string(),
            is_breaking: false,
            impact_level: ImpactLevel::Medium,
        };

        tracker.track_change(change1).unwrap();
        tracker.track_change(change2).unwrap();

        let breaking = tracker.get_breaking_changes();
        assert_eq!(breaking.len(), 1);
    }

    #[test]
    fn test_is_affected() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());
        tracker.add_project("project-b".to_string());
        tracker.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        assert!(tracker.is_affected("change-1", "project-a"));
        assert!(!tracker.is_affected("change-1", "project-b"));
    }

    #[test]
    fn test_filter_changes_by_type() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change1 = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        let change2 = Change {
            id: "change-2".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Config,
            description: "Config change".to_string(),
            is_breaking: false,
            impact_level: ImpactLevel::Low,
        };

        tracker.track_change(change1).unwrap();
        tracker.track_change(change2).unwrap();

        let all_changes = tracker.get_all_changes();
        let api_changes = tracker.filter_changes_by_type(&all_changes, ChangeType::Api);

        assert_eq!(api_changes.len(), 1);
    }

    #[test]
    fn test_filter_changes_by_breaking() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change1 = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "Breaking change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        let change2 = Change {
            id: "change-2".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "Non-breaking change".to_string(),
            is_breaking: false,
            impact_level: ImpactLevel::Medium,
        };

        tracker.track_change(change1).unwrap();
        tracker.track_change(change2).unwrap();

        let all_changes = tracker.get_all_changes();
        let breaking = tracker.filter_changes_by_breaking(&all_changes, true);

        assert_eq!(breaking.len(), 1);
    }

    #[test]
    fn test_get_change_details() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());
        tracker.add_project("project-b".to_string());
        tracker.add_dependency("project-a".to_string(), "project-b".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-b".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        let details = tracker.get_change_details("change-1").unwrap();
        assert_eq!(details.affected_count, 1);
        assert!(details.affected_projects.contains(&"project-a".to_string()));
    }

    #[test]
    fn test_transitive_propagation() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());
        tracker.add_project("project-b".to_string());
        tracker.add_project("project-c".to_string());

        tracker.add_dependency("project-a".to_string(), "project-b".to_string());
        tracker.add_dependency("project-b".to_string(), "project-c".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-c".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        let affected = tracker.get_affected_projects("change-1");
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&"project-a".to_string()));
        assert!(affected.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_clear_tracker() {
        let mut tracker = ChangePropagationTracker::new();
        tracker.add_project("project-a".to_string());

        let change = Change {
            id: "change-1".to_string(),
            project: "project-a".to_string(),
            change_type: ChangeType::Api,
            description: "API change".to_string(),
            is_breaking: true,
            impact_level: ImpactLevel::Critical,
        };

        tracker.track_change(change).unwrap();

        assert_eq!(tracker.get_all_changes().len(), 1);

        tracker.clear();

        assert_eq!(tracker.get_all_changes().len(), 0);
        assert_eq!(tracker.get_projects().len(), 0);
    }
}
