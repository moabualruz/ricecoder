//! Property-based tests for operation sequencing in domain agent coordination
//!
//! **Feature: ricecoder-domain-agents, Property 7: Operation Sequencing**
//! **Validates: Requirements 4.4**

#[cfg(test)]
mod tests {
    use crate::domain::coordinator::{DomainCoordinator, Operation};

    fn create_operation(id: &str, name: &str, priority: u32, dependencies: Vec<&str>) -> Operation {
        Operation {
            id: id.to_string(),
            name: name.to_string(),
            priority,
            dependencies: dependencies.iter().map(|d| d.to_string()).collect(),
        }
    }

    /// Property 7: Operation Sequencing
    /// For any cross-domain task execution, the Domain Agent System SHALL sequence
    /// operations in the correct order (e.g., infrastructure setup before deployment,
    /// database schema before application deployment).
    ///
    /// This property tests that:
    /// 1. Operations are sorted by priority
    /// 2. Lower priority values execute first
    /// 3. Sequencing is deterministic
    /// 4. Dependencies are respected
    #[test]
    fn property_operation_sequencing_by_priority() {
        let coordinator = DomainCoordinator::new();

        // Create operations with different priorities
        let operations = vec![
            create_operation("deploy", "Deploy Application", 3, vec![]),
            create_operation("setup", "Setup Infrastructure", 1, vec![]),
            create_operation("migrate", "Database Migration", 2, vec!["setup"]),
        ];

        // Sequence multiple times to ensure determinism
        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: Operations should be sorted by priority
            assert_eq!(
                sequenced[0].priority, 1,
                "First operation should have priority 1"
            );
            assert_eq!(
                sequenced[1].priority, 2,
                "Second operation should have priority 2"
            );
            assert_eq!(
                sequenced[2].priority, 3,
                "Third operation should have priority 3"
            );

            // Property: Setup should come first
            assert_eq!(sequenced[0].id, "setup", "Setup should be first");

            // Property: Migration should come second
            assert_eq!(sequenced[1].id, "migrate", "Migration should be second");

            // Property: Deploy should come last
            assert_eq!(sequenced[2].id, "deploy", "Deploy should be last");
        }
    }

    /// Property 7: Operation Sequencing (Deterministic)
    /// For any set of operations, sequencing them multiple times SHALL produce
    /// identical results.
    #[test]
    fn property_operation_sequencing_deterministic() {
        let coordinator = DomainCoordinator::new();

        let operations = vec![
            create_operation("op1", "Operation 1", 5, vec![]),
            create_operation("op2", "Operation 2", 2, vec![]),
            create_operation("op3", "Operation 3", 8, vec![]),
            create_operation("op4", "Operation 4", 1, vec![]),
            create_operation("op5", "Operation 5", 3, vec![]),
        ];

        // Sequence multiple times
        let mut results = Vec::new();
        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();
            results.push(sequenced);
        }

        // Property: All sequences should be identical
        let first = &results[0];
        for result in &results[1..] {
            assert_eq!(
                result.len(),
                first.len(),
                "All sequences should have same length"
            );

            for (i, op) in result.iter().enumerate() {
                assert_eq!(
                    op.id, first[i].id,
                    "Operation {} ID should be consistent",
                    i
                );
                assert_eq!(
                    op.priority, first[i].priority,
                    "Operation {} priority should be consistent",
                    i
                );
            }
        }
    }

    /// Property 7: Operation Sequencing (Dependency Respect)
    /// For any operations with dependencies, dependencies SHALL be satisfied
    /// before dependent operations.
    #[test]
    fn property_operation_sequencing_respects_dependencies() {
        let coordinator = DomainCoordinator::new();

        // Create operations with explicit dependencies
        let operations = vec![
            create_operation("app-deploy", "Deploy Application", 3, vec!["db-migrate"]),
            create_operation("db-migrate", "Database Migration", 2, vec!["infra-setup"]),
            create_operation("infra-setup", "Setup Infrastructure", 1, vec![]),
        ];

        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: Infrastructure setup should come first
            assert_eq!(
                sequenced[0].id, "infra-setup",
                "Infrastructure setup should be first"
            );

            // Property: Database migration should come second
            assert_eq!(
                sequenced[1].id, "db-migrate",
                "Database migration should be second"
            );

            // Property: Application deployment should come last
            assert_eq!(
                sequenced[2].id, "app-deploy",
                "Application deployment should be last"
            );

            // Property: Each operation should come after its dependencies
            let mut seen_ids = std::collections::HashSet::new();
            for op in &sequenced {
                for dep in &op.dependencies {
                    assert!(
                        seen_ids.contains(dep),
                        "Dependency {} should be executed before {}",
                        dep,
                        op.id
                    );
                }
                seen_ids.insert(op.id.clone());
            }
        }
    }

    /// Property 7: Operation Sequencing (Empty Input)
    /// For any empty input, sequencing SHALL produce empty output.
    #[test]
    fn property_operation_sequencing_empty_input() {
        let coordinator = DomainCoordinator::new();

        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(vec![]).unwrap();

            // Property: Empty input should produce empty output
            assert!(
                sequenced.is_empty(),
                "Empty input should produce empty output"
            );
        }
    }

    /// Property 7: Operation Sequencing (Single Operation)
    /// For any single operation, sequencing SHALL return it unchanged.
    #[test]
    fn property_operation_sequencing_single_operation() {
        let coordinator = DomainCoordinator::new();

        let operation = create_operation("single", "Single Operation", 5, vec![]);

        for _ in 0..5 {
            let sequenced = coordinator
                .sequence_operations(vec![operation.clone()])
                .unwrap();

            // Property: Single operation should be returned unchanged
            assert_eq!(sequenced.len(), 1, "Should have one operation");
            assert_eq!(
                sequenced[0].id, "single",
                "Operation ID should be preserved"
            );
            assert_eq!(
                sequenced[0].priority, 5,
                "Operation priority should be preserved"
            );
        }
    }

    /// Property 7: Operation Sequencing (Stable Sort)
    /// For any operations with equal priority, their relative order SHALL be preserved.
    #[test]
    fn property_operation_sequencing_stable_sort() {
        let coordinator = DomainCoordinator::new();

        // Create operations with equal priorities
        let operations = vec![
            create_operation("op1", "Operation 1", 1, vec![]),
            create_operation("op2", "Operation 2", 1, vec![]),
            create_operation("op3", "Operation 3", 1, vec![]),
        ];

        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: All operations should have same priority
            for op in &sequenced {
                assert_eq!(op.priority, 1, "All operations should have priority 1");
            }

            // Property: Relative order should be preserved (stable sort)
            assert_eq!(sequenced[0].id, "op1", "First operation should be op1");
            assert_eq!(sequenced[1].id, "op2", "Second operation should be op2");
            assert_eq!(sequenced[2].id, "op3", "Third operation should be op3");
        }
    }

    /// Property 7: Operation Sequencing (Complex Dependencies)
    /// For any complex dependency graph, all dependencies SHALL be satisfied.
    #[test]
    fn property_operation_sequencing_complex_dependencies() {
        let coordinator = DomainCoordinator::new();

        // Create a complex dependency graph
        let operations = vec![
            create_operation("app", "Deploy App", 5, vec!["db", "cache"]),
            create_operation("db", "Setup Database", 2, vec!["infra"]),
            create_operation("cache", "Setup Cache", 3, vec!["infra"]),
            create_operation("infra", "Setup Infrastructure", 1, vec![]),
            create_operation("monitor", "Setup Monitoring", 4, vec!["infra"]),
        ];

        for _ in 0..3 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: Infrastructure should come first
            assert_eq!(sequenced[0].id, "infra", "Infrastructure should be first");

            // Property: All dependencies should be satisfied
            let mut seen_ids = std::collections::HashSet::new();
            for op in &sequenced {
                for dep in &op.dependencies {
                    assert!(
                        seen_ids.contains(dep),
                        "Dependency {} should be executed before {}",
                        dep,
                        op.id
                    );
                }
                seen_ids.insert(op.id.clone());
            }

            // Property: App deployment should come last (depends on db and cache)
            assert_eq!(
                sequenced[sequenced.len() - 1].id,
                "app",
                "App deployment should be last"
            );
        }
    }

    /// Property 7: Operation Sequencing (Priority Ordering)
    /// For any operations, lower priority values SHALL execute before higher values.
    #[test]
    fn property_operation_sequencing_priority_ordering() {
        let coordinator = DomainCoordinator::new();

        // Create operations with various priorities
        let operations = vec![
            create_operation("op10", "Operation 10", 10, vec![]),
            create_operation("op5", "Operation 5", 5, vec![]),
            create_operation("op1", "Operation 1", 1, vec![]),
            create_operation("op7", "Operation 7", 7, vec![]),
            create_operation("op3", "Operation 3", 3, vec![]),
        ];

        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: Operations should be in ascending priority order
            for i in 0..sequenced.len() - 1 {
                assert!(
                    sequenced[i].priority <= sequenced[i + 1].priority,
                    "Priority should be non-decreasing"
                );
            }

            // Property: Specific order should be maintained
            assert_eq!(sequenced[0].priority, 1);
            assert_eq!(sequenced[1].priority, 3);
            assert_eq!(sequenced[2].priority, 5);
            assert_eq!(sequenced[3].priority, 7);
            assert_eq!(sequenced[4].priority, 10);
        }
    }

    /// Property 7: Operation Sequencing (Preservation of Data)
    /// For any operations, all operation data SHALL be preserved during sequencing.
    #[test]
    fn property_operation_sequencing_data_preservation() {
        let coordinator = DomainCoordinator::new();

        let operations = vec![
            create_operation("op1", "Operation 1", 3, vec!["dep1", "dep2"]),
            create_operation("op2", "Operation 2", 1, vec![]),
            create_operation("op3", "Operation 3", 2, vec!["dep3"]),
        ];

        for _ in 0..5 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: All operations should be present
            assert_eq!(sequenced.len(), 3, "All operations should be present");

            // Property: Operation data should be preserved
            let op1 = sequenced.iter().find(|op| op.id == "op1").unwrap();
            assert_eq!(op1.name, "Operation 1");
            assert_eq!(op1.priority, 3);
            assert_eq!(op1.dependencies.len(), 2);
            assert!(op1.dependencies.contains(&"dep1".to_string()));
            assert!(op1.dependencies.contains(&"dep2".to_string()));

            let op2 = sequenced.iter().find(|op| op.id == "op2").unwrap();
            assert_eq!(op2.name, "Operation 2");
            assert_eq!(op2.priority, 1);
            assert!(op2.dependencies.is_empty());

            let op3 = sequenced.iter().find(|op| op.id == "op3").unwrap();
            assert_eq!(op3.name, "Operation 3");
            assert_eq!(op3.priority, 2);
            assert_eq!(op3.dependencies.len(), 1);
            assert!(op3.dependencies.contains(&"dep3".to_string()));
        }
    }

    /// Property 7: Operation Sequencing (Large Number of Operations)
    /// For any large number of operations, sequencing SHALL complete successfully
    /// and maintain correct ordering.
    #[test]
    fn property_operation_sequencing_large_number() {
        let coordinator = DomainCoordinator::new();

        // Create 100 operations with random priorities
        let mut operations = Vec::new();
        for i in 0..100 {
            let priority = (i * 7) % 100; // Pseudo-random priority
            operations.push(create_operation(
                &format!("op{}", i),
                &format!("Operation {}", i),
                priority as u32,
                vec![],
            ));
        }

        for _ in 0..3 {
            let sequenced = coordinator.sequence_operations(operations.clone()).unwrap();

            // Property: All operations should be present
            assert_eq!(sequenced.len(), 100, "All 100 operations should be present");

            // Property: Operations should be sorted by priority
            for i in 0..sequenced.len() - 1 {
                assert!(
                    sequenced[i].priority <= sequenced[i + 1].priority,
                    "Operations should be sorted by priority"
                );
            }
        }
    }
}
