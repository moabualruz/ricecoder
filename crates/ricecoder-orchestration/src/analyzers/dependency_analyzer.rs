//! Dependency analysis for cross-project relationships

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    error::{OrchestrationError, Result},
    models::{DependencyType, Project, ProjectDependency},
};

/// Analyzes project dependencies and builds dependency graphs
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// Map of project names to their dependencies
    dependencies: HashMap<String, Vec<ProjectDependency>>,

    /// Map of project names to projects
    projects: HashMap<String, Project>,
}

impl DependencyAnalyzer {
    /// Creates a new dependency analyzer
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            projects: HashMap::new(),
        }
    }

    /// Adds a project to the analyzer
    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.name.clone(), project);
    }

    /// Adds a dependency between two projects
    pub fn add_dependency(&mut self, dependency: ProjectDependency) {
        self.dependencies
            .entry(dependency.from.clone())
            .or_default()
            .push(dependency);
    }

    /// Parses dependencies from a Cargo.toml file
    /// Extracts dependencies, dev-dependencies, and build-dependencies
    pub fn parse_cargo_toml(&mut self, content: &str, project_name: &str) -> Result<()> {
        let parsed: toml::Value = toml::from_str(content).map_err(|e| {
            OrchestrationError::ConfigurationError(format!("Failed to parse Cargo.toml: {}", e))
        })?;

        // Extract package version if available
        let _package_version = parsed
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0");

        // Parse dependencies section
        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for (dep_name, dep_value) in deps {
                let version = extract_version_constraint(dep_value);
                let dependency = ProjectDependency {
                    from: project_name.to_string(),
                    to: dep_name.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: version,
                };
                self.add_dependency(dependency);
            }
        }

        // Parse dev-dependencies section
        if let Some(dev_deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
            for (dep_name, dep_value) in dev_deps {
                let version = extract_version_constraint(dep_value);
                let dependency = ProjectDependency {
                    from: project_name.to_string(),
                    to: dep_name.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: version,
                };
                self.add_dependency(dependency);
            }
        }

        // Parse build-dependencies section
        if let Some(build_deps) = parsed.get("build-dependencies").and_then(|d| d.as_table()) {
            for (dep_name, dep_value) in build_deps {
                let version = extract_version_constraint(dep_value);
                let dependency = ProjectDependency {
                    from: project_name.to_string(),
                    to: dep_name.clone(),
                    dependency_type: DependencyType::Direct,
                    version_constraint: version,
                };
                self.add_dependency(dependency);
            }
        }

        Ok(())
    }

    /// Gets all direct dependencies of a project
    pub fn get_direct_dependencies(&self, project_name: &str) -> Vec<ProjectDependency> {
        self.dependencies
            .get(project_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Gets all transitive dependencies of a project
    pub fn get_transitive_dependencies(
        &self,
        project_name: &str,
    ) -> Result<Vec<ProjectDependency>> {
        let mut transitive = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct dependencies
        for dep in self.get_direct_dependencies(project_name) {
            queue.push_back(dep.to.clone());
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Get dependencies of this project
            for dep in self.get_direct_dependencies(&current) {
                if !visited.contains(&dep.to) {
                    transitive.push(ProjectDependency {
                        from: project_name.to_string(),
                        to: dep.to.clone(),
                        dependency_type: DependencyType::Transitive,
                        version_constraint: dep.version_constraint.clone(),
                    });
                    queue.push_back(dep.to.clone());
                }
            }
        }

        Ok(transitive)
    }

    /// Detects circular dependencies in the dependency graph
    pub fn detect_circular_dependencies(&self) -> Result<Vec<Vec<String>>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for project_name in self.projects.keys() {
            if !visited.contains(project_name) {
                self.dfs_cycle_detection(
                    project_name,
                    &mut visited,
                    &mut rec_stack,
                    &mut cycles,
                    Vec::new(),
                )?;
            }
        }

        if !cycles.is_empty() {
            return Err(OrchestrationError::CircularDependency(format!(
                "Found {} circular dependencies",
                cycles.len()
            )));
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn dfs_cycle_detection(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
        mut path: Vec<String>,
    ) -> Result<()> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        for dep in self.get_direct_dependencies(node) {
            if !visited.contains(&dep.to) {
                self.dfs_cycle_detection(&dep.to, visited, rec_stack, cycles, path.clone())?;
            } else if rec_stack.contains(&dep.to) {
                // Found a cycle
                if let Some(start_idx) = path.iter().position(|x| x == &dep.to) {
                    let cycle = path[start_idx..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        rec_stack.remove(node);
        Ok(())
    }

    /// Gets all projects that depend on a given project (upstream)
    pub fn get_upstream_dependents(&self, project_name: &str) -> Vec<String> {
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

    /// Gets all projects that a given project depends on (downstream)
    pub fn get_downstream_dependencies(&self, project_name: &str) -> Vec<String> {
        self.get_direct_dependencies(project_name)
            .iter()
            .map(|d| d.to.clone())
            .collect()
    }

    /// Validates that all dependencies reference existing projects
    pub fn validate_dependencies(&self) -> Result<()> {
        for (from, deps) in &self.dependencies {
            // Check that the 'from' project exists
            if !self.projects.contains_key(from) {
                return Err(OrchestrationError::DependencyValidationFailed(format!(
                    "Project '{}' not found",
                    from
                )));
            }

            // Check that all 'to' projects exist
            for dep in deps {
                if !self.projects.contains_key(&dep.to) {
                    return Err(OrchestrationError::DependencyValidationFailed(format!(
                        "Dependency target '{}' not found for project '{}'",
                        dep.to, from
                    )));
                }
            }
        }

        Ok(())
    }

    /// Gets the dependency graph as an adjacency list
    pub fn get_adjacency_list(&self) -> HashMap<String, Vec<String>> {
        let mut adj_list = HashMap::new();

        for project_name in self.projects.keys() {
            let deps = self.get_direct_dependencies(project_name);
            let dep_names: Vec<String> = deps.iter().map(|d| d.to.clone()).collect();
            adj_list.insert(project_name.clone(), dep_names);
        }

        adj_list
    }

    /// Performs a topological sort of projects based on dependencies
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for project_name in self.projects.keys() {
            in_degree.insert(project_name.clone(), 0);
            adj_list.insert(project_name.clone(), Vec::new());
        }

        // Build adjacency list and calculate in-degrees
        for (from, deps) in &self.dependencies {
            for dep in deps {
                adj_list
                    .entry(from.clone())
                    .or_default()
                    .push(dep.to.clone());
                *in_degree.entry(dep.to.clone()).or_insert(0) += 1;
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

            if let Some(neighbors) = adj_list.get(&node) {
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

    /// Gets all projects
    pub fn get_projects(&self) -> Vec<Project> {
        self.projects.values().cloned().collect()
    }

    /// Gets all dependencies
    pub fn get_all_dependencies(&self) -> Vec<ProjectDependency> {
        self.dependencies
            .values()
            .flat_map(|deps| deps.clone())
            .collect()
    }

    /// Clears all data
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.projects.clear();
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts version constraint from a TOML dependency value
fn extract_version_constraint(value: &toml::Value) -> String {
    match value {
        // Simple version string: "1.0.0"
        toml::Value::String(version) => version.clone(),
        // Detailed dependency: { version = "1.0.0", features = [...] }
        toml::Value::Table(table) => table
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        // Workspace dependency: { workspace = true }
        _ => "*".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn create_test_project(name: &str) -> Project {
        Project {
            path: PathBuf::from(format!("/path/to/{}", name)),
            name: name.to_string(),
            project_type: "rust".to_string(),
            version: "0.1.0".to_string(),
            status: crate::models::ProjectStatus::Healthy,
        }
    }

    #[test]
    fn test_add_project() {
        let mut analyzer = DependencyAnalyzer::new();
        let project = create_test_project("project-a");

        analyzer.add_project(project.clone());

        assert_eq!(analyzer.get_projects().len(), 1);
        assert_eq!(analyzer.get_projects()[0].name, "project-a");
    }

    #[test]
    fn test_add_dependency() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));

        let dep = ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        };

        analyzer.add_dependency(dep);

        let deps = analyzer.get_direct_dependencies("project-a");
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].to, "project-b");
    }

    #[test]
    fn test_get_direct_dependencies() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let deps = analyzer.get_direct_dependencies("project-a");
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_get_transitive_dependencies() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        // A -> B -> C
        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let transitive = analyzer.get_transitive_dependencies("project-a").unwrap();
        assert_eq!(transitive.len(), 1);
        assert_eq!(transitive[0].to, "project-c");
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        // A -> B -> C -> A (circular)
        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-c".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.detect_circular_dependencies();
        assert!(result.is_err());
    }

    #[test]
    fn test_no_circular_dependencies() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        // A -> B -> C (no cycle)
        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.detect_circular_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_upstream_dependents() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let dependents = analyzer.get_upstream_dependents("project-c");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"project-a".to_string()));
        assert!(dependents.contains(&"project-b".to_string()));
    }

    #[test]
    fn test_get_downstream_dependencies() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let deps = analyzer.get_downstream_dependencies("project-a");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"project-b".to_string()));
        assert!(deps.contains(&"project-c".to_string()));
    }

    #[test]
    fn test_validate_dependencies_success() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.validate_dependencies();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies_missing_from() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-b"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.validate_dependencies();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dependencies_missing_to() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.validate_dependencies();
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        // A -> B -> C
        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let sorted = analyzer.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        // A should come before B, B should come before C
        let a_idx = sorted.iter().position(|x| x == "project-a").unwrap();
        let b_idx = sorted.iter().position(|x| x == "project-b").unwrap();
        let c_idx = sorted.iter().position(|x| x == "project-c").unwrap();

        assert!(a_idx < b_idx);
        assert!(b_idx < c_idx);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));

        // A -> B -> A (cycle)
        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-b".to_string(),
            to: "project-a".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let result = analyzer.topological_sort();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_adjacency_list() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));
        analyzer.add_project(create_test_project("project-b"));
        analyzer.add_project(create_test_project("project-c"));

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-b".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        analyzer.add_dependency(ProjectDependency {
            from: "project-a".to_string(),
            to: "project-c".to_string(),
            dependency_type: DependencyType::Direct,
            version_constraint: "^0.1.0".to_string(),
        });

        let adj_list = analyzer.get_adjacency_list();
        assert_eq!(adj_list.len(), 3);
        assert_eq!(adj_list.get("project-a").unwrap().len(), 2);
        assert_eq!(adj_list.get("project-b").unwrap().len(), 0);
        assert_eq!(adj_list.get("project-c").unwrap().len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_project(create_test_project("project-a"));

        assert_eq!(analyzer.get_projects().len(), 1);

        analyzer.clear();

        assert_eq!(analyzer.get_projects().len(), 0);
    }
}
