//! Unit tests for ImpactAnalyzer
//!
//! Tests various change scenarios, impact tracing through dependencies,
//! and report generation.

use ricecoder_orchestration::{ImpactAnalyzer, ImpactLevel, ProjectChange};

#[test]
fn test_simple_impact_analysis() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-b".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.change_id, "change-1");
    assert_eq!(report.affected_projects.len(), 1);
    assert!(report.affected_projects.contains(&"project-a".to_string()));
    assert_eq!(report.impact_level, ImpactLevel::Critical);
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

    assert_eq!(report.affected_projects.len(), 2);
    assert!(report.affected_projects.contains(&"project-a".to_string()));
    assert!(report.affected_projects.contains(&"project-b".to_string()));
}

#[test]
fn test_diamond_dependency() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_project("project-c".to_string());
    analyzer.add_project("project-d".to_string());

    // A -> B -> D, A -> C -> D
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

    assert_eq!(report.affected_projects.len(), 3);
    assert!(report.affected_projects.contains(&"project-a".to_string()));
    assert!(report.affected_projects.contains(&"project-b".to_string()));
    assert!(report.affected_projects.contains(&"project-c".to_string()));
}

#[test]
fn test_breaking_api_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "api".to_string(),
        description: "Breaking API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::Critical);
}

#[test]
fn test_non_breaking_api_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "api".to_string(),
        description: "Non-breaking API change".to_string(),
        is_breaking: false,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::Medium);
}

#[test]
fn test_breaking_dependency_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "dependency".to_string(),
        description: "Breaking dependency change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::High);
}

#[test]
fn test_non_breaking_dependency_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "dependency".to_string(),
        description: "Non-breaking dependency change".to_string(),
        is_breaking: false,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::Medium);
}

#[test]
fn test_breaking_config_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "config".to_string(),
        description: "Breaking config change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::High);
}

#[test]
fn test_non_breaking_config_change_impact_level() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-a".to_string(),
        change_type: "config".to_string(),
        description: "Non-breaking config change".to_string(),
        is_breaking: false,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.impact_level, ImpactLevel::Low);
}

#[test]
fn test_impact_details_generation() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-b".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.details.len(), 1);
    assert_eq!(report.details[0].project, "project-a");
    assert!(!report.details[0].reason.is_empty());
    assert!(!report.details[0].required_actions.is_empty());
}

#[test]
fn test_impact_details_contain_required_actions() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-b".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    let actions = &report.details[0].required_actions;
    assert!(actions.iter().any(|a| a.contains("Review")));
    assert!(actions.iter().any(|a| a.contains("Update")));
    assert!(actions.iter().any(|a| a.contains("tests")));
}

#[test]
fn test_multiple_affected_projects() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_project("project-c".to_string());

    // Both A and B depend on C
    analyzer.add_dependency("project-a".to_string(), "project-c".to_string());
    analyzer.add_dependency("project-b".to_string(), "project-c".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-c".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    assert_eq!(report.affected_projects.len(), 2);
    assert!(report.affected_projects.contains(&"project-a".to_string()));
    assert!(report.affected_projects.contains(&"project-b".to_string()));
    assert_eq!(report.details.len(), 2);
}

#[test]
fn test_get_affected_projects() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

    let affected = analyzer.get_affected_projects("project-b");

    assert_eq!(affected.len(), 1);
    assert!(affected.contains(&"project-a".to_string()));
}

#[test]
fn test_count_affected_projects() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_project("project-c".to_string());

    analyzer.add_dependency("project-a".to_string(), "project-c".to_string());
    analyzer.add_dependency("project-b".to_string(), "project-c".to_string());

    let count = analyzer.count_affected_projects("project-c");

    assert_eq!(count, 2);
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
fn test_breaking_change_reason() {
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

    assert!(report.details[0].reason.contains("Breaking"));
    assert!(report.details[0].reason.contains("api"));
}

#[test]
fn test_non_breaking_change_reason() {
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

    assert!(report.details[0].reason.contains("Non-breaking"));
    assert!(report.details[0].reason.contains("api"));
}

#[test]
fn test_complex_dependency_chain() {
    let mut analyzer = ImpactAnalyzer::new();

    // Create a chain: A -> B -> C -> D -> E
    for i in 0..5 {
        analyzer.add_project(format!("project-{}", i));
    }

    for i in 0..4 {
        analyzer.add_dependency(
            format!("project-{}", i),
            format!("project-{}", i + 1),
        );
    }

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-4".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    // All projects except the changed one should be affected
    assert_eq!(report.affected_projects.len(), 4);
    for i in 0..4 {
        assert!(report.affected_projects.contains(&format!("project-{}", i)));
    }
}

#[test]
fn test_no_duplicate_affected_projects() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());

    // Add the same dependency twice
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());
    analyzer.add_dependency("project-a".to_string(), "project-b".to_string());

    let change = ProjectChange {
        change_id: "change-1".to_string(),
        project: "project-b".to_string(),
        change_type: "api".to_string(),
        description: "API change".to_string(),
        is_breaking: true,
    };

    let report = analyzer.analyze_impact(&change).unwrap();

    // Should have exactly 1 affected project, not 2
    assert_eq!(report.affected_projects.len(), 1);
    assert_eq!(report.details.len(), 1);
}

#[test]
fn test_get_projects() {
    let mut analyzer = ImpactAnalyzer::new();
    analyzer.add_project("project-a".to_string());
    analyzer.add_project("project-b".to_string());
    analyzer.add_project("project-c".to_string());

    let projects = analyzer.get_projects();

    assert_eq!(projects.len(), 3);
    assert!(projects.contains(&"project-a".to_string()));
    assert!(projects.contains(&"project-b".to_string()));
    assert!(projects.contains(&"project-c".to_string()));
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
