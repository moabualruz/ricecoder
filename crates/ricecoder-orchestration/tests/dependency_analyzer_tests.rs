//! Unit tests for DependencyAnalyzer
//!
//! Tests for simple and complex dependency graphs, circular dependency detection,
//! and transitive dependency resolution.

use std::path::PathBuf;

use ricecoder_orchestration::{
    DependencyAnalyzer, DependencyType, Project, ProjectDependency, ProjectStatus,
};

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
fn test_simple_dependency_graph() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> C
    analyzer.add_project(create_test_project("project-a"));
    analyzer.add_project(create_test_project("project-b"));
    analyzer.add_project(create_test_project("project-c"));

    // Add dependencies
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

    // Verify direct dependencies
    let a_deps = analyzer.get_direct_dependencies("project-a");
    assert_eq!(a_deps.len(), 1);
    assert_eq!(a_deps[0].to, "project-b");

    let b_deps = analyzer.get_direct_dependencies("project-b");
    assert_eq!(b_deps.len(), 1);
    assert_eq!(b_deps[0].to, "project-c");

    let c_deps = analyzer.get_direct_dependencies("project-c");
    assert_eq!(c_deps.len(), 0);
}

#[test]
fn test_complex_dependency_graph() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create a diamond dependency: A -> B, A -> C, B -> D, C -> D
    for name in &["project-a", "project-b", "project-c", "project-d"] {
        analyzer.add_project(create_test_project(name));
    }

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

    analyzer.add_dependency(ProjectDependency {
        from: "project-b".to_string(),
        to: "project-d".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    analyzer.add_dependency(ProjectDependency {
        from: "project-c".to_string(),
        to: "project-d".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Verify A has 2 direct dependencies
    let a_deps = analyzer.get_direct_dependencies("project-a");
    assert_eq!(a_deps.len(), 2);

    // Verify D has 0 direct dependencies
    let d_deps = analyzer.get_direct_dependencies("project-d");
    assert_eq!(d_deps.len(), 0);

    // Verify upstream dependents of D
    let d_dependents = analyzer.get_upstream_dependents("project-d");
    assert_eq!(d_dependents.len(), 2);
    assert!(d_dependents.contains(&"project-b".to_string()));
    assert!(d_dependents.contains(&"project-c".to_string()));
}

#[test]
fn test_circular_dependency_detection_simple() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> A (circular)
    analyzer.add_project(create_test_project("project-a"));
    analyzer.add_project(create_test_project("project-b"));

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

    // Detect circular dependencies
    let result = analyzer.detect_circular_dependencies();
    assert!(result.is_err());
}

#[test]
fn test_circular_dependency_detection_complex() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> C -> A (circular)
    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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

    // Detect circular dependencies
    let result = analyzer.detect_circular_dependencies();
    assert!(result.is_err());
}

#[test]
fn test_no_circular_dependencies() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> C (no cycle)
    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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

    // Detect circular dependencies
    let result = analyzer.detect_circular_dependencies();
    assert!(result.is_ok());
}

#[test]
fn test_transitive_dependency_resolution() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> C -> D
    for name in &["project-a", "project-b", "project-c", "project-d"] {
        analyzer.add_project(create_test_project(name));
    }

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
        to: "project-d".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Get transitive dependencies of A
    let transitive = analyzer.get_transitive_dependencies("project-a").unwrap();
    assert_eq!(transitive.len(), 2);

    // Should include C and D (but not B, which is direct)
    let to_names: Vec<String> = transitive.iter().map(|d| d.to.clone()).collect();
    assert!(to_names.contains(&"project-c".to_string()));
    assert!(to_names.contains(&"project-d".to_string()));
}

#[test]
fn test_topological_sort_simple() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create projects: A -> B -> C
    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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

    // Perform topological sort
    let sorted = analyzer.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);

    // Verify order
    let a_idx = sorted.iter().position(|x| x == "project-a").unwrap();
    let b_idx = sorted.iter().position(|x| x == "project-b").unwrap();
    let c_idx = sorted.iter().position(|x| x == "project-c").unwrap();

    assert!(a_idx < b_idx);
    assert!(b_idx < c_idx);
}

#[test]
fn test_topological_sort_diamond() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create diamond: A -> B, A -> C, B -> D, C -> D
    for name in &["project-a", "project-b", "project-c", "project-d"] {
        analyzer.add_project(create_test_project(name));
    }

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

    analyzer.add_dependency(ProjectDependency {
        from: "project-b".to_string(),
        to: "project-d".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    analyzer.add_dependency(ProjectDependency {
        from: "project-c".to_string(),
        to: "project-d".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Perform topological sort
    let sorted = analyzer.topological_sort().unwrap();
    assert_eq!(sorted.len(), 4);

    // Verify order constraints
    let a_idx = sorted.iter().position(|x| x == "project-a").unwrap();
    let b_idx = sorted.iter().position(|x| x == "project-b").unwrap();
    let c_idx = sorted.iter().position(|x| x == "project-c").unwrap();
    let d_idx = sorted.iter().position(|x| x == "project-d").unwrap();

    assert!(a_idx < b_idx);
    assert!(a_idx < c_idx);
    assert!(b_idx < d_idx);
    assert!(c_idx < d_idx);
}

#[test]
fn test_topological_sort_with_cycle() {
    let mut analyzer = DependencyAnalyzer::new();

    // Create cycle: A -> B -> A
    analyzer.add_project(create_test_project("project-a"));
    analyzer.add_project(create_test_project("project-b"));

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

    // Topological sort should fail
    let result = analyzer.topological_sort();
    assert!(result.is_err());
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
fn test_get_upstream_dependents() {
    let mut analyzer = DependencyAnalyzer::new();

    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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

    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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
fn test_get_adjacency_list() {
    let mut analyzer = DependencyAnalyzer::new();

    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

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
fn test_multiple_dependency_types() {
    let mut analyzer = DependencyAnalyzer::new();

    for name in &["project-a", "project-b", "project-c"] {
        analyzer.add_project(create_test_project(name));
    }

    // Add direct dependency
    analyzer.add_dependency(ProjectDependency {
        from: "project-a".to_string(),
        to: "project-b".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    // Add dev dependency
    analyzer.add_dependency(ProjectDependency {
        from: "project-a".to_string(),
        to: "project-c".to_string(),
        dependency_type: DependencyType::Dev,
        version_constraint: "^0.1.0".to_string(),
    });

    let deps = analyzer.get_direct_dependencies("project-a");
    assert_eq!(deps.len(), 2);

    let direct_count = deps
        .iter()
        .filter(|d| d.dependency_type == DependencyType::Direct)
        .count();
    let dev_count = deps
        .iter()
        .filter(|d| d.dependency_type == DependencyType::Dev)
        .count();

    assert_eq!(direct_count, 1);
    assert_eq!(dev_count, 1);
}

#[test]
fn test_clear_analyzer() {
    let mut analyzer = DependencyAnalyzer::new();

    analyzer.add_project(create_test_project("project-a"));
    analyzer.add_project(create_test_project("project-b"));

    analyzer.add_dependency(ProjectDependency {
        from: "project-a".to_string(),
        to: "project-b".to_string(),
        dependency_type: DependencyType::Direct,
        version_constraint: "^0.1.0".to_string(),
    });

    assert_eq!(analyzer.get_projects().len(), 2);
    assert_eq!(analyzer.get_all_dependencies().len(), 1);

    analyzer.clear();

    assert_eq!(analyzer.get_projects().len(), 0);
    assert_eq!(analyzer.get_all_dependencies().len(), 0);
}
