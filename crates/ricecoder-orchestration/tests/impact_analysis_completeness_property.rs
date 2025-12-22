//! Property-based tests for impact analysis completeness
//!
//! **Feature: ricecoder-orchestration, Property 4: Impact Analysis Completeness**
//! **Validates: Requirements 3.3**

use std::collections::HashSet;

use proptest::prelude::*;
use ricecoder_orchestration::{ImpactAnalyzer, ProjectChange};

/// Strategy for generating project names
fn project_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string())
}

/// Strategy for generating unique project names
fn unique_project_names_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec("[a-z][a-z0-9-]{0,10}".prop_map(|s| s.to_string()), 1..10).prop_map(
        |mut names| {
            // Make names unique by adding index
            for (i, name) in names.iter_mut().enumerate() {
                *name = format!("{}-{}", name, i);
            }
            names.sort();
            names.dedup();
            names
        },
    )
}

/// Strategy for generating dependency configurations
fn dependency_config_strategy() -> impl Strategy<Value = Vec<(usize, usize)>> {
    prop::collection::vec((0usize..10, 0usize..10), 0..20)
}

/// Strategy for generating change types
fn change_type_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("api".to_string()),
        Just("dependency".to_string()),
        Just("config".to_string()),
    ]
}

proptest! {
    /// Property 4: Impact Analysis Completeness
    ///
    /// For any change to a project, the ImpactAnalyzer SHALL identify all
    /// downstream projects that are affected by that change without false negatives.
    ///
    /// **Validates: Requirements 3.3**
    #[test]
    fn prop_impact_analysis_identifies_all_affected_projects(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
        changed_project_idx in 0usize..10,
        change_type in change_type_strategy(),
        is_breaking in any::<bool>(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut analyzer = ImpactAnalyzer::new();

        // Add all projects to the analyzer
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Add dependencies
        let mut expected_affected = HashSet::new();
        for (from_idx, to_idx) in &dependency_indices {
            if *from_idx < project_names.len() && *to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[*from_idx];
                let to = &project_names[*to_idx];

                analyzer.add_dependency(from.clone(), to.clone());
            }
        }

        // Determine which project changed
        let changed_idx = changed_project_idx % project_names.len();
        let changed_project = &project_names[changed_idx];

        // Manually compute expected affected projects (BFS from changed project)
        let mut visited = HashSet::new();
        let mut queue = vec![changed_project.clone()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            // Find all projects that depend on current
            for (from_idx, to_idx) in dependency_indices.iter() {
                if *from_idx < project_names.len() && *to_idx < project_names.len() {
                    let from = &project_names[*from_idx];
                    let to = &project_names[*to_idx];

                    if to == &current && !visited.contains(from) {
                        expected_affected.insert(from.clone());
                        queue.push(from.clone());
                    }
                }
            }
        }

        // Create a change
        let change = ProjectChange {
            change_id: "test-change".to_string(),
            project: changed_project.clone(),
            change_type: change_type.clone(),
            description: "Test change".to_string(),
            is_breaking,
        };

        // Analyze impact
        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // Verify all expected affected projects are in the report
        let affected_set: HashSet<String> = report.affected_projects.iter().cloned().collect();

        for expected in &expected_affected {
            prop_assert!(
                affected_set.contains(expected),
                "Expected project {} to be affected but it was not",
                expected
            );
        }

        // Verify no false positives (all reported affected projects should be expected)
        for affected in &affected_set {
            prop_assert!(
                expected_affected.contains(affected),
                "Project {} was reported as affected but should not be",
                affected
            );
        }

        // Verify the count matches
        prop_assert_eq!(
            report.affected_projects.len(),
            expected_affected.len(),
            "Affected project count mismatch"
        );
    }

    /// Property: No false negatives in impact analysis
    ///
    /// For any project that depends on a changed project, it should be identified
    /// as affected.
    #[test]
    fn prop_no_false_negatives_in_impact_analysis(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 2);

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Create a simple dependency: A depends on B
        let a = &project_names[0];
        let b = &project_names[1];

        analyzer.add_dependency(a.clone(), b.clone());

        // Create a change to B
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: b.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // A should be affected
        prop_assert!(
            report.affected_projects.contains(&a.clone()),
            "Project {} should be affected by change to {}",
            a,
            b
        );
    }

    /// Property: No false positives in impact analysis
    ///
    /// For any project that does not depend on a changed project, it should not
    /// be identified as affected.
    #[test]
    fn prop_no_false_positives_in_impact_analysis(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 3);

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Create a dependency: A depends on B
        let a = &project_names[0];
        let b = &project_names[1];
        let c = &project_names[2];

        analyzer.add_dependency(a.clone(), b.clone());

        // Create a change to B
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: b.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // C should not be affected (it doesn't depend on B)
        prop_assert!(
            !report.affected_projects.contains(&c.clone()),
            "Project {} should not be affected by change to {}",
            c,
            b
        );
    }

    /// Property: Transitive dependencies are identified
    ///
    /// For any transitive dependency chain A -> B -> C, a change to C should
    /// affect both A and B.
    #[test]
    fn prop_transitive_dependencies_identified(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 3);

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Create a chain: A -> B -> C
        let a = &project_names[0];
        let b = &project_names[1];
        let c = &project_names[2];

        analyzer.add_dependency(a.clone(), b.clone());
        analyzer.add_dependency(b.clone(), c.clone());

        // Create a change to C
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: c.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // Both A and B should be affected
        prop_assert!(
            report.affected_projects.contains(&a.clone()),
            "Project {} should be affected by transitive dependency",
            a
        );
        prop_assert!(
            report.affected_projects.contains(&b.clone()),
            "Project {} should be affected by direct dependency",
            b
        );
    }

    /// Property: Diamond dependency is handled correctly
    ///
    /// For a diamond dependency (A -> B -> D, A -> C -> D), a change to D
    /// should affect A, B, and C (but not duplicate D).
    #[test]
    fn prop_diamond_dependency_handled_correctly(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 4);

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Create diamond: A -> B -> D, A -> C -> D
        let a = &project_names[0];
        let b = &project_names[1];
        let c = &project_names[2];
        let d = &project_names[3];

        analyzer.add_dependency(a.clone(), b.clone());
        analyzer.add_dependency(a.clone(), c.clone());
        analyzer.add_dependency(b.clone(), d.clone());
        analyzer.add_dependency(c.clone(), d.clone());

        // Create a change to D
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: d.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // A, B, and C should be affected
        prop_assert!(
            report.affected_projects.contains(&a.clone()),
            "Project {} should be affected",
            a
        );
        prop_assert!(
            report.affected_projects.contains(&b.clone()),
            "Project {} should be affected",
            b
        );
        prop_assert!(
            report.affected_projects.contains(&c.clone()),
            "Project {} should be affected",
            c
        );

        // Should have exactly 3 affected projects (no duplicates)
        prop_assert_eq!(
            report.affected_projects.len(),
            3,
            "Should have exactly 3 affected projects"
        );
    }

    /// Property: Impact report contains all affected projects
    ///
    /// For any impact analysis, the report should contain all affected projects
    /// and no duplicates.
    #[test]
    fn prop_impact_report_contains_all_affected_no_duplicates(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
        changed_project_idx in 0usize..10,
    ) {
        prop_assume!(!project_names.is_empty());

        let mut analyzer = ImpactAnalyzer::new();

        // Add all projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Add dependencies
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                analyzer.add_dependency(from.clone(), to.clone());
            }
        }

        // Determine which project changed
        let changed_idx = changed_project_idx % project_names.len();
        let changed_project = &project_names[changed_idx];

        // Create a change
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: changed_project.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // Check for duplicates
        let mut seen = HashSet::new();
        for project in &report.affected_projects {
            prop_assert!(
                seen.insert(project.clone()),
                "Duplicate project {} in affected list",
                project
            );
        }

        // Verify all affected projects are valid
        for project in &report.affected_projects {
            prop_assert!(
                project_names.contains(project),
                "Invalid project {} in affected list",
                project
            );
        }
    }

    /// Property: Impact level is determined correctly
    ///
    /// For any change, the impact level should be appropriate based on the
    /// change type and breaking status.
    #[test]
    fn prop_impact_level_determined_correctly(
        project_names in unique_project_names_strategy(),
        change_type in change_type_strategy(),
        is_breaking in any::<bool>(),
    ) {
        prop_assume!(!project_names.is_empty());

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        let changed_project = &project_names[0];

        // Create a change
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: changed_project.clone(),
            change_type: change_type.clone(),
            description: "Test change".to_string(),
            is_breaking,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // Verify impact level is set
        match (change_type.as_str(), is_breaking) {
            ("api", true) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::Critical,
                    "Breaking API change should have Critical impact"
                );
            }
            ("api", false) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::Medium,
                    "Non-breaking API change should have Medium impact"
                );
            }
            ("dependency", true) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::High,
                    "Breaking dependency change should have High impact"
                );
            }
            ("dependency", false) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::Medium,
                    "Non-breaking dependency change should have Medium impact"
                );
            }
            ("config", true) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::High,
                    "Breaking config change should have High impact"
                );
            }
            ("config", false) => {
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::Low,
                    "Non-breaking config change should have Low impact"
                );
            }
            _ => {
                // Other types should have Low impact
                prop_assert_eq!(
                    report.impact_level,
                    ricecoder_orchestration::ImpactLevel::Low,
                    "Unknown change type should have Low impact"
                );
            }
        }
    }

    /// Property: Impact details are generated for all affected projects
    ///
    /// For any impact analysis, there should be a detail entry for each
    /// affected project.
    #[test]
    fn prop_impact_details_generated_for_all_affected(
        project_names in unique_project_names_strategy(),
        dependency_indices in dependency_config_strategy(),
        changed_project_idx in 0usize..10,
    ) {
        prop_assume!(!project_names.is_empty());

        let mut analyzer = ImpactAnalyzer::new();

        // Add all projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        // Add dependencies
        for (from_idx, to_idx) in dependency_indices {
            if from_idx < project_names.len() && to_idx < project_names.len() && from_idx != to_idx {
                let from = &project_names[from_idx];
                let to = &project_names[to_idx];

                analyzer.add_dependency(from.clone(), to.clone());
            }
        }

        // Determine which project changed
        let changed_idx = changed_project_idx % project_names.len();
        let changed_project = &project_names[changed_idx];

        // Create a change
        let change = ProjectChange {
            change_id: "change-1".to_string(),
            project: changed_project.clone(),
            change_type: "api".to_string(),
            description: "API change".to_string(),
            is_breaking: true,
        };

        let report = analyzer.analyze_impact(&change).expect("impact analysis failed");

        // Verify details count matches affected projects count
        prop_assert_eq!(
            report.details.len(),
            report.affected_projects.len(),
            "Details count should match affected projects count"
        );

        // Verify each detail corresponds to an affected project
        for detail in &report.details {
            prop_assert!(
                report.affected_projects.contains(&detail.project),
                "Detail project {} not in affected projects",
                detail.project
            );

            // Verify detail has required information
            prop_assert!(
                !detail.reason.is_empty(),
                "Detail reason should not be empty"
            );
            prop_assert!(
                !detail.required_actions.is_empty(),
                "Detail required actions should not be empty"
            );
        }
    }

    /// Property: Multiple changes are analyzed independently
    ///
    /// For any set of changes, each should be analyzed independently without
    /// affecting others.
    #[test]
    fn prop_multiple_changes_analyzed_independently(
        project_names in unique_project_names_strategy(),
    ) {
        prop_assume!(project_names.len() >= 2);

        let mut analyzer = ImpactAnalyzer::new();

        // Add projects
        for name in &project_names {
            analyzer.add_project(name.clone());
        }

        let a = &project_names[0];
        let b = &project_names[1];

        // A depends on B (so if B changes, A is affected)
        analyzer.add_dependency(a.clone(), b.clone());

        // Create two changes
        let change1 = ProjectChange {
            change_id: "change-1".to_string(),
            project: a.clone(),
            change_type: "api".to_string(),
            description: "Change to A".to_string(),
            is_breaking: true,
        };

        let change2 = ProjectChange {
            change_id: "change-2".to_string(),
            project: b.clone(),
            change_type: "api".to_string(),
            description: "Change to B".to_string(),
            is_breaking: true,
        };

        let report1 = analyzer.analyze_impact(&change1).expect("impact analysis failed");
        let report2 = analyzer.analyze_impact(&change2).expect("impact analysis failed");

        // Change to A should not affect B (A depends on B, not the other way around)
        prop_assert!(
            !report1.affected_projects.contains(&b.clone()),
            "Change to {} should not affect {}",
            a,
            b
        );

        // Change to B should affect A (A depends on B)
        prop_assert!(
            report2.affected_projects.contains(&a.clone()),
            "Change to {} should affect {}",
            b,
            a
        );
    }
}
