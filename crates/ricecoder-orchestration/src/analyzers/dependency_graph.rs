//! Dependency graph construction and maintenance

use crate::error::{OrchestrationError, Result};
use crate::models::{Project, ProjectDependency};
use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a directed graph of project dependencies
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Adjacency list representation of the graph
    adjacency_list: HashMap<String, Vec<String>>,

    /// Reverse adjacency list (for upstream queries)
    reverse_adjacency_list: HashMap<String, Vec<String>>,

    /// All projects in the graph
    projects: HashMap<String, Project>,

    /// All dependencies in the graph
    dependencies: HashMap<(String, String), ProjectDependency>,

    /// Whether cycles are allowed
    allow_cycles: bool,
}

impl DependencyGraph {
    /// Creates a new dependency graph
    pub fn new(allow_cycles: bool) -> Self {
        Self {
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
            projects: HashMap::new(),
            dependencies: HashMap::new(),
            allow_cycles,
        }
    }

    /// Adds a project to the graph
    pub fn add_project(&mut self, project: Project) -> Result<()> {
        let name = project.name.clone();
        self.projects.insert(name.clone(), project);
        self.adjacency_list.entry(name.clone()).or_default();
        self.reverse_adjacency_list.entry(name).or_default();
        Ok(())
    }

    /// Adds a dependency to the graph
    pub fn add_dependency(&mut self, dependency: ProjectDependency) -> Result<()> {
        // Validate that both projects exist
        if !self.projects.contains_key(&dependency.from) {
            return Err(OrchestrationError::ProjectNotFound(dependency.from.clone()));
        }
        if !self.projects.contains_key(&dependency.to) {
            return Err(OrchestrationError::ProjectNotFound(dependency.to.clone()));
        }

        // Check for cycles if not allowed
        if !self.allow_cycles && self.would_create_cycle(&dependency.from, &dependency.to) {
            return Err(OrchestrationError::CircularDependency(format!(
                "Adding dependency from {} to {} would create a cycle",
                dependency.from, dependency.to
            )));
        }

        // Add to adjacency lists
        self.adjacency_list
            .entry(dependency.from.clone())
            .or_default()
            .push(dependency.to.clone());

        self.reverse_adjacency_list
            .entry(dependency.to.clone())
            .or_default()
            .push(dependency.from.clone());

        // Store the dependency
        self.dependencies
            .insert((dependency.from.clone(), dependency.to.clone()), dependency);

        Ok(())
    }

    /// Removes a dependency from the graph
    pub fn remove_dependency(&mut self, from: &str, to: &str) -> Result<()> {
        // Remove from adjacency list
        if let Some(deps) = self.adjacency_list.get_mut(from) {
            deps.retain(|d| d != to);
        }

        // Remove from reverse adjacency list
        if let Some(deps) = self.reverse_adjacency_list.get_mut(to) {
            deps.retain(|d| d != from);
        }

        // Remove from dependencies
        self.dependencies
            .remove(&(from.to_string(), to.to_string()));

        Ok(())
    }

    /// Checks if adding a dependency would create a cycle
    fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        // If 'to' can reach 'from', then adding from->to would create a cycle
        self.can_reach(to, from)
    }

    /// Checks if one project can reach another through the dependency graph
    pub fn can_reach(&self, from: &str, to: &str) -> bool {
        if from == to {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if let Some(neighbors) = self.adjacency_list.get(&current) {
                for neighbor in neighbors {
                    if neighbor == to {
                        return true;
                    }
                    if !visited.contains(neighbor) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        false
    }

    /// Gets all direct dependencies of a project
    pub fn get_dependencies(&self, project: &str) -> Vec<String> {
        self.adjacency_list
            .get(project)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets all projects that depend on a given project
    pub fn get_dependents(&self, project: &str) -> Vec<String> {
        self.reverse_adjacency_list
            .get(project)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets all transitive dependencies of a project
    pub fn get_transitive_dependencies(&self, project: &str) -> Vec<String> {
        let mut transitive = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(direct_deps) = self.adjacency_list.get(project) {
            for dep in direct_deps {
                queue.push_back(dep.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            transitive.push(current.clone());

            if let Some(deps) = self.adjacency_list.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        transitive
    }

    /// Validates the graph consistency
    pub fn validate(&self) -> Result<()> {
        // Check that all dependencies reference existing projects
        for (from, to) in self.dependencies.keys() {
            if !self.projects.contains_key(from) {
                return Err(OrchestrationError::DependencyValidationFailed(format!(
                    "Project '{}' not found",
                    from
                )));
            }
            if !self.projects.contains_key(to) {
                return Err(OrchestrationError::DependencyValidationFailed(format!(
                    "Project '{}' not found",
                    to
                )));
            }
        }

        // Check for cycles if not allowed
        if !self.allow_cycles {
            self.detect_cycles()?;
        }

        Ok(())
    }

    /// Detects cycles in the graph
    pub fn detect_cycles(&self) -> Result<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for project_name in self.projects.keys() {
            if !visited.contains(project_name) {
                self.dfs_cycle_detection(project_name, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<()> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = self.adjacency_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, visited, rec_stack)?;
                } else if rec_stack.contains(neighbor) {
                    return Err(OrchestrationError::CircularDependency(format!(
                        "Cycle detected: {} -> {}",
                        node, neighbor
                    )));
                }
            }
        }

        rec_stack.remove(node);
        Ok(())
    }

    /// Gets the topological order of projects
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degrees
        for project_name in self.projects.keys() {
            in_degree.insert(project_name.clone(), 0);
        }

        // Calculate in-degrees
        for neighbors in self.adjacency_list.values() {
            for neighbor in neighbors {
                *in_degree.entry(neighbor.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());

            if let Some(neighbors) = self.adjacency_list.get(&node) {
                for neighbor in neighbors {
                    let degree = in_degree.entry(neighbor.clone()).or_insert(0);
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        if sorted.len() != self.projects.len() {
            return Err(OrchestrationError::CircularDependency(
                "Topological sort failed: circular dependencies detected".to_string(),
            ));
        }

        Ok(sorted)
    }

    /// Gets all projects in the graph
    pub fn get_projects(&self) -> Vec<Project> {
        self.projects.values().cloned().collect()
    }

    /// Gets all dependencies in the graph
    pub fn get_all_dependencies(&self) -> Vec<ProjectDependency> {
        self.dependencies.values().cloned().collect()
    }

    /// Gets the number of projects
    pub fn project_count(&self) -> usize {
        self.projects.len()
    }

    /// Gets the number of dependencies
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Checks if a project exists in the graph
    pub fn has_project(&self, name: &str) -> bool {
        self.projects.contains_key(name)
    }

    /// Checks if a dependency exists in the graph
    pub fn has_dependency(&self, from: &str, to: &str) -> bool {
        self.dependencies
            .contains_key(&(from.to_string(), to.to_string()))
    }

    /// Gets the adjacency list representation
    pub fn get_adjacency_list(&self) -> HashMap<String, Vec<String>> {
        self.adjacency_list.clone()
    }

    /// Gets the reverse adjacency list representation
    pub fn get_reverse_adjacency_list(&self) -> HashMap<String, Vec<String>> {
        self.reverse_adjacency_list.clone()
    }

    /// Clears the graph
    pub fn clear(&mut self) {
        self.adjacency_list.clear();
        self.reverse_adjacency_list.clear();
        self.projects.clear();
        self.dependencies.clear();
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DependencyType, ProjectStatus};
    use std::path::PathBuf;

    fn create_test_project(name: &str) -> Project {
        Project {
            path: PathBuf::from(format!("/path/to/{}", name)),
            name: name.to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: ProjectStatus::Healthy,
        }
    }

    #[test]
    fn test_create_graph() {
        let graph = DependencyGraph::new(false);
        assert_eq!(graph.project_count(), 0);
        assert_eq!(graph.dependency_count(), 0);
    }

    #[test]
    fn test_add_project() {
        let mut graph = DependencyGraph::new(false);
        let project = create_test_project("project-a");

        graph.add_project(project).unwrap();

        assert_eq!(graph.project_count(), 1);
        assert!(graph.has_project("project-a"));
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        graph.add_dependency(dep).unwrap();

        assert_eq!(graph.dependency_count(), 1);
        assert!(graph.has_dependency("project-a", "project-b"));
    }

    #[test]
    fn test_add_dependency_missing_project() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        let result = graph.add_dependency(dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_cycle_detection_prevented() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();

        // Add A -> B
        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        // Try to add B -> A (would create cycle)
        let result = graph.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_cycle_allowed() {
        let mut graph = DependencyGraph::new(true);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();

        // Add A -> B
        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        // Add B -> A (cycle allowed)
        let result = graph.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();
        graph.add_project(create_test_project("project-c")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let deps = graph.get_dependencies("project-a");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"project-b".to_string()));
        assert!(deps.contains(&"project-c".to_string()));
    }

    #[test]
    fn test_get_dependents() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();
        graph.add_project(create_test_project("project-c")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let dependents = graph.get_dependents("project-c");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"project-a".to_string()));
        assert!(dependents.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_can_reach() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();
        graph.add_project(create_test_project("project-c")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        assert!(graph.can_reach("project-a", "project-b"));
        assert!(graph.can_reach("project-a", "project-c"));
        assert!(graph.can_reach("project-b", "project-c"));
        assert!(!graph.can_reach("project-c", "project-a"));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();
        graph.add_project(create_test_project("project-c")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        let a_idx = sorted.iter().position(|x| x == "project-a").unwrap();
        let b_idx = sorted.iter().position(|x| x == "project-b").unwrap();
        let c_idx = sorted.iter().position(|x| x == "project-c").unwrap();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_remove_dependency() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        assert_eq!(graph.dependency_count(), 1);

        graph.remove_dependency("project-a", "project-b").unwrap();

        assert_eq!(graph.dependency_count(), 0);
        assert!(!graph.has_dependency("project-a", "project-b"));
    }

    #[test]
    fn test_validate_success() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let result = graph.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_transitive_dependencies() {
        let mut graph = DependencyGraph::new(false);
        graph.add_project(create_test_project("project-a")).unwrap();
        graph.add_project(create_test_project("project-b")).unwrap();
        graph.add_project(create_test_project("project-c")).unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-a".to_string(),
                to: "project-b".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        graph
            .add_dependency(ProjectDependency {
                from: "project-b".to_string(),
                to: "project-c".to_string(),
                dependency_type: DependencyType::Direct,
                version_constraint: "^0.1.0".to_string(),
            })
            .unwrap();

        let transitive = graph.get_transitive_dependencies("project-a");
        assert_eq!(transitive.len(), 2);
        assert!(transitive.contains(&"project-b".to_string()));
        assert!(transitive.contains(&"project-c".to_string()));
    }
}
