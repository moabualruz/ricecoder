//! Property-based tests for spec-to-task traceability
//! **Feature: ricecoder-specs, Property 11: Spec-to-Task Traceability**
//! **Validates: Requirements 4.4, 4.5**

use proptest::prelude::*;
use ricecoder_specs::{models::*, workflow::WorkflowOrchestrator};

// ============================================================================
// Generators for property-based testing
// ============================================================================

fn arb_task_id() -> impl Strategy<Value = String> {
    "[0-9]{1,3}(\\.[0-9]{1,3})?".prop_map(|s| s)
}

fn arb_requirement_id() -> impl Strategy<Value = String> {
    "REQ-[0-9]{1,3}".prop_map(|s| s)
}

// ============================================================================
// Property 11: Spec-to-Task Traceability
// ============================================================================

proptest! {
    /// Property: For any task, there SHALL be explicit links to acceptance
    /// criteria from the requirements document.
    ///
    /// This property verifies that:
    /// 1. Each task has explicit requirement links
    /// 2. Requirement links are preserved correctly
    /// 3. Tasks can be traced back to requirements
    #[test]
    fn prop_task_has_explicit_requirement_links(
        task_id in arb_task_id(),
        req_id_1 in arb_requirement_id(),
        req_id_2 in arb_requirement_id(),
    ) {
        // Only test if requirement IDs are different
        if req_id_1 != req_id_2 {
            let mut orchestrator = WorkflowOrchestrator::new();
            let requirement_ids = vec![req_id_1.clone(), req_id_2.clone()];

            // Link task to requirements
            let result = orchestrator.link_task_to_requirements(
                task_id.clone(),
                requirement_ids.clone(),
            );

            // Property 11.1: Linking should succeed
            prop_assert!(result.is_ok(), "Task-to-requirement linking should succeed");

            // Property 11.2: Task should have explicit requirement links
            let linked_reqs = orchestrator.get_task_requirements(&task_id);
            prop_assert_eq!(
                linked_reqs.len(),
                requirement_ids.len(),
                "Task should have all requirement links"
            );

            // Property 11.3: All requirement IDs should be present
            for req_id in &requirement_ids {
                prop_assert!(
                    linked_reqs.contains(req_id),
                    "Task should have link to requirement: {}",
                    req_id
                );
            }
        }
    }

    /// Property: For any requirement, there SHALL be at least one task
    /// linked to it.
    ///
    /// This property verifies that requirements have task coverage.
    #[test]
    fn prop_requirement_has_task_links(
        req_id in arb_requirement_id(),
        task_id_1 in arb_task_id(),
        task_id_2 in arb_task_id(),
    ) {
        // Only test if task IDs are different
        if task_id_1 != task_id_2 {
            let mut orchestrator = WorkflowOrchestrator::new();

            // Link multiple tasks to same requirement
            orchestrator
                .link_task_to_requirements(
                    task_id_1.clone(),
                    vec![req_id.clone()],
                )
                .unwrap();

            orchestrator
                .link_task_to_requirements(
                    task_id_2.clone(),
                    vec![req_id.clone()],
                )
                .unwrap();

            // Property 11.4: Requirement should have task links
            let linked_tasks = orchestrator.get_requirement_tasks(&req_id);
            prop_assert_eq!(
                linked_tasks.len(),
                2,
                "Requirement should have all task links"
            );

            // Property 11.5: All task IDs should be present
            prop_assert!(
                linked_tasks.contains(&task_id_1),
                "Requirement should have link to task: {}",
                task_id_1
            );
            prop_assert!(
                linked_tasks.contains(&task_id_2),
                "Requirement should have link to task: {}",
                task_id_2
            );
        }
    }

    /// Property: For any task-to-requirement link, the reverse
    /// requirement-to-task link SHALL also exist.
    ///
    /// This property verifies bidirectional traceability.
    #[test]
    fn prop_bidirectional_traceability(
        task_id in arb_task_id(),
        req_id in arb_requirement_id(),
    ) {
        let mut orchestrator = WorkflowOrchestrator::new();

        // Link task to requirement
        orchestrator
            .link_task_to_requirements(
                task_id.clone(),
                vec![req_id.clone()],
            )
            .unwrap();

        // Property 11.6: Forward link should exist
        let task_reqs = orchestrator.get_task_requirements(&task_id);
        prop_assert!(
            task_reqs.contains(&req_id),
            "Forward link (task -> requirement) should exist"
        );

        // Property 11.7: Reverse link should exist
        let req_tasks = orchestrator.get_requirement_tasks(&req_id);
        prop_assert!(
            req_tasks.contains(&task_id),
            "Reverse link (requirement -> task) should exist"
        );
    }

    /// Property: For any task, updating its status SHALL not affect
    /// its requirement links.
    ///
    /// This property verifies that task status changes don't break traceability.
    #[test]
    fn prop_task_status_change_preserves_links(
        task_id in arb_task_id(),
        req_id in arb_requirement_id(),
    ) {
        let mut orchestrator = WorkflowOrchestrator::new();

        // Link task to requirement
        orchestrator
            .link_task_to_requirements(
                task_id.clone(),
                vec![req_id.clone()],
            )
            .unwrap();

        let original_links = orchestrator.get_task_requirements(&task_id);

        // Update task status
        orchestrator
            .update_task_status(task_id.clone(), TaskStatus::InProgress)
            .unwrap();

        // Property 11.8: Links should be preserved after status change
        let updated_links = orchestrator.get_task_requirements(&task_id);
        prop_assert_eq!(
            original_links, updated_links,
            "Task requirement links should be preserved after status change"
        );
    }

    /// Property: For any set of tasks, each task SHALL have at least
    /// one requirement link.
    ///
    /// This property verifies complete task coverage.
    #[test]
    fn prop_all_tasks_have_requirement_links(
        task_id_1 in arb_task_id(),
        task_id_2 in arb_task_id(),
        req_id_1 in arb_requirement_id(),
        req_id_2 in arb_requirement_id(),
    ) {
        // Only test if IDs are different
        if task_id_1 != task_id_2 && req_id_1 != req_id_2 {
            let mut orchestrator = WorkflowOrchestrator::new();

            // Link both tasks to requirements
            orchestrator
                .link_task_to_requirements(
                    task_id_1.clone(),
                    vec![req_id_1.clone()],
                )
                .unwrap();

            orchestrator
                .link_task_to_requirements(
                    task_id_2.clone(),
                    vec![req_id_2.clone()],
                )
                .unwrap();

            // Property 11.9: All tasks should have links
            let all_tasks = orchestrator.get_all_linked_tasks();
            prop_assert_eq!(
                all_tasks.len(),
                2,
                "All tasks should have requirement links"
            );

            prop_assert!(
                all_tasks.contains(&task_id_1),
                "Task 1 should be in linked tasks"
            );
            prop_assert!(
                all_tasks.contains(&task_id_2),
                "Task 2 should be in linked tasks"
            );
        }
    }

    /// Property: For any requirement, there SHALL be at least one task
    /// linked to it.
    ///
    /// This property verifies complete requirement coverage.
    #[test]
    fn prop_all_requirements_have_task_links(
        task_id in arb_task_id(),
        req_id_1 in arb_requirement_id(),
        req_id_2 in arb_requirement_id(),
    ) {
        // Only test if requirement IDs are different
        if req_id_1 != req_id_2 {
            let mut orchestrator = WorkflowOrchestrator::new();

            // Link task to multiple requirements
            orchestrator
                .link_task_to_requirements(
                    task_id.clone(),
                    vec![req_id_1.clone(), req_id_2.clone()],
                )
                .unwrap();

            // Property 11.10: All requirements should have links
            let all_reqs = orchestrator.get_all_linked_requirements();
            prop_assert_eq!(
                all_reqs.len(),
                2,
                "All requirements should have task links"
            );

            prop_assert!(
                all_reqs.contains(&req_id_1),
                "Requirement 1 should be in linked requirements"
            );
            prop_assert!(
                all_reqs.contains(&req_id_2),
                "Requirement 2 should be in linked requirements"
            );
        }
    }

    /// Property: For any task with multiple requirement links, all links
    /// SHALL be retrievable.
    ///
    /// This property verifies that multiple links are preserved correctly.
    #[test]
    fn prop_multiple_requirement_links_preserved(
        task_id in arb_task_id(),
        req_id_1 in arb_requirement_id(),
        req_id_2 in arb_requirement_id(),
        req_id_3 in arb_requirement_id(),
    ) {
        // Only test if requirement IDs are different
        if req_id_1 != req_id_2 && req_id_2 != req_id_3 && req_id_1 != req_id_3 {
            let mut orchestrator = WorkflowOrchestrator::new();

            let requirement_ids = vec![req_id_1.clone(), req_id_2.clone(), req_id_3.clone()];

            // Link task to multiple requirements
            orchestrator
                .link_task_to_requirements(
                    task_id.clone(),
                    requirement_ids.clone(),
                )
                .unwrap();

            // Property 11.11: All links should be retrievable
            let linked_reqs = orchestrator.get_task_requirements(&task_id);
            prop_assert_eq!(
                linked_reqs.len(),
                requirement_ids.len(),
                "All requirement links should be preserved"
            );

            for req_id in &requirement_ids {
                prop_assert!(
                    linked_reqs.contains(req_id),
                    "Requirement link should be preserved: {}",
                    req_id
                );
            }
        }
    }

    /// Property: For any requirement with multiple task links, all links
    /// SHALL be retrievable.
    ///
    /// This property verifies that multiple reverse links are preserved correctly.
    #[test]
    fn prop_multiple_task_links_preserved(
        task_id_1 in arb_task_id(),
        task_id_2 in arb_task_id(),
        task_id_3 in arb_task_id(),
        req_id in arb_requirement_id(),
    ) {
        // Only test if task IDs are different
        if task_id_1 != task_id_2 && task_id_2 != task_id_3 && task_id_1 != task_id_3 {
            let mut orchestrator = WorkflowOrchestrator::new();

            let task_ids = vec![task_id_1.clone(), task_id_2.clone(), task_id_3.clone()];

            // Link multiple tasks to same requirement
            for task_id in &task_ids {
                orchestrator
                    .link_task_to_requirements(
                        task_id.clone(),
                        vec![req_id.clone()],
                    )
                    .unwrap();
            }

            // Property 11.12: All reverse links should be retrievable
            let linked_tasks = orchestrator.get_requirement_tasks(&req_id);
            prop_assert_eq!(
                linked_tasks.len(),
                task_ids.len(),
                "All task links should be preserved"
            );

            for task_id in &task_ids {
                prop_assert!(
                    linked_tasks.contains(task_id),
                    "Task link should be preserved: {}",
                    task_id
                );
            }
        }
    }

    /// Property: For any task, resetting the orchestrator SHALL remove
    /// all its links.
    ///
    /// This property verifies that reset clears all traceability.
    #[test]
    fn prop_reset_clears_all_links(
        task_id in arb_task_id(),
        req_id in arb_requirement_id(),
    ) {
        let mut orchestrator = WorkflowOrchestrator::new();

        // Link task to requirement
        orchestrator
            .link_task_to_requirements(
                task_id.clone(),
                vec![req_id.clone()],
            )
            .unwrap();

        // Verify link exists
        prop_assert!(!orchestrator.get_task_requirements(&task_id).is_empty());

        // Reset orchestrator
        orchestrator.reset();

        // Property 11.13: All links should be cleared
        prop_assert!(
            orchestrator.get_task_requirements(&task_id).is_empty(),
            "Task links should be cleared after reset"
        );
        prop_assert!(
            orchestrator.get_requirement_tasks(&req_id).is_empty(),
            "Requirement links should be cleared after reset"
        );
        prop_assert_eq!(
            orchestrator.get_all_linked_tasks().len(),
            0,
            "All task links should be cleared after reset"
        );
        prop_assert_eq!(
            orchestrator.get_all_linked_requirements().len(),
            0,
            "All requirement links should be cleared after reset"
        );
    }
}
