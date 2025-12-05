//! Property-based tests for pause and resume functionality
//! **Feature: ricecoder-workflows, Property 8: Pause and Resume**
//! **Validates: Requirements 2.2**

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::state::StateManager;
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Strategy for generating WorkflowState in Running status
    fn arb_running_workflow_state() -> impl Strategy<Value = WorkflowState> {
        (
            "workflow-[a-z0-9]{1,10}",
            prop::option::of("[a-z0-9]{1,10}"),
            prop::collection::vec("[a-z0-9]{1,10}", 0..5),
        )
            .prop_map(|(workflow_id, current_step, completed_steps)| {
                let mut step_results = HashMap::new();

                // Add results for all completed steps
                for step_id in &completed_steps {
                    step_results.insert(
                        step_id.clone(),
                        StepResult {
                            status: StepStatus::Completed,
                            output: Some(serde_json::json!({"result": "success"})),
                            error: None,
                            duration_ms: 100,
                        },
                    );
                }

                // Add result for current step if present
                if let Some(ref current) = current_step {
                    if !step_results.contains_key(current) {
                        step_results.insert(
                            current.clone(),
                            StepResult {
                                status: StepStatus::Running,
                                output: None,
                                error: None,
                                duration_ms: 0,
                            },
                        );
                    }
                }

                WorkflowState {
                    workflow_id,
                    status: WorkflowStatus::Running,
                    current_step,
                    completed_steps,
                    step_results,
                    started_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            })
    }

    proptest! {
        /// Property 8: Pause and Resume
        /// For any paused workflow, resuming execution SHALL continue from the paused step
        /// without re-executing previously completed steps.
        #[test]
        fn prop_pause_resume_preserves_completed_steps(mut state in arb_running_workflow_state()) {
            let original_completed_steps = state.completed_steps.clone();
            let original_current_step = state.current_step.clone();

            // Pause the workflow
            StateManager::pause_workflow(&mut state)
                .expect("Failed to pause workflow");

            prop_assert_eq!(state.status, WorkflowStatus::Paused);
            prop_assert_eq!(&state.completed_steps, &original_completed_steps);
            prop_assert_eq!(&state.current_step, &original_current_step);

            // Resume the workflow
            StateManager::resume_workflow(&mut state)
                .expect("Failed to resume workflow");

            prop_assert_eq!(state.status, WorkflowStatus::Running);
            // Completed steps should not change
            prop_assert_eq!(&state.completed_steps, &original_completed_steps);
            // Current step should not change
            prop_assert_eq!(&state.current_step, &original_current_step);
        }

        /// Property 8: Pause and Resume - No Re-execution
        /// For any workflow with completed steps, pausing and resuming
        /// SHALL NOT re-execute any completed steps.
        #[test]
        fn prop_pause_resume_no_reexecution(mut state in arb_running_workflow_state()) {
            let original_completed_steps = state.completed_steps.clone();

            // Pause the workflow
            StateManager::pause_workflow(&mut state)
                .expect("Failed to pause workflow");

            // Resume the workflow
            StateManager::resume_workflow(&mut state)
                .expect("Failed to resume workflow");

            // Verify no steps were re-executed
            prop_assert_eq!(&state.completed_steps, &original_completed_steps);

            // Verify all completed steps are still marked as completed
            for step_id in &original_completed_steps {
                let is_completed = StateManager::is_step_completed(&state, step_id);
                prop_assert!(is_completed);
            }
        }

        /// Property 8: Pause and Resume - State Consistency
        /// For any workflow, pausing and resuming SHALL maintain state consistency.
        #[test]
        fn prop_pause_resume_maintains_consistency(mut state in arb_running_workflow_state()) {
            let original_step_results = state.step_results.clone();

            // Pause the workflow
            StateManager::pause_workflow(&mut state)
                .expect("Failed to pause workflow");

            // Resume the workflow
            StateManager::resume_workflow(&mut state)
                .expect("Failed to resume workflow");

            // Verify step results are unchanged
            prop_assert_eq!(&state.step_results, &original_step_results);
        }

        /// Property 8: Pause and Resume - Next Step Calculation
        /// For any paused and resumed workflow, the next step to execute
        /// SHALL be the first incomplete step.
        #[test]
        fn prop_pause_resume_next_step_correct(mut state in arb_running_workflow_state()) {
            let available_steps = vec![
                "step1".to_string(),
                "step2".to_string(),
                "step3".to_string(),
                "step4".to_string(),
            ];

            // Mark some steps as completed
            state.completed_steps = vec!["step1".to_string(), "step2".to_string()];
            state.step_results.insert(
                "step1".to_string(),
                StepResult {
                    status: StepStatus::Completed,
                    output: None,
                    error: None,
                    duration_ms: 100,
                },
            );
            state.step_results.insert(
                "step2".to_string(),
                StepResult {
                    status: StepStatus::Completed,
                    output: None,
                    error: None,
                    duration_ms: 100,
                },
            );

            // Get next step before pause
            let next_before = StateManager::get_next_step_to_execute(&state, &available_steps);
            prop_assert_eq!(&next_before, &Some("step3".to_string()));

            // Pause the workflow
            StateManager::pause_workflow(&mut state)
                .expect("Failed to pause workflow");

            // Resume the workflow
            StateManager::resume_workflow(&mut state)
                .expect("Failed to resume workflow");

            // Get next step after resume
            let next_after = StateManager::get_next_step_to_execute(&state, &available_steps);

            // Next step should be the same
            prop_assert_eq!(&next_after, &next_before);
        }

        /// Property 8: Pause and Resume - Idempotence
        /// For any workflow, pausing multiple times SHALL have the same effect as pausing once.
        #[test]
        fn prop_pause_is_idempotent(mut state in arb_running_workflow_state()) {
            let original_completed_steps = state.completed_steps.clone();

            // Pause the workflow
            StateManager::pause_workflow(&mut state)
                .expect("Failed to pause workflow");

            let state_after_first_pause = state.clone();

            // Try to pause again (should fail since not running)
            let result = StateManager::pause_workflow(&mut state);
            prop_assert!(result.is_err());

            // State should be unchanged
            prop_assert_eq!(state.status, state_after_first_pause.status);
            prop_assert_eq!(state.completed_steps, original_completed_steps);
        }

        /// Property 8: Pause and Resume - Cannot Resume Non-Paused
        /// For any workflow that is not paused, resuming SHALL fail.
        #[test]
        fn prop_cannot_resume_non_paused(mut state in arb_running_workflow_state()) {
            // Try to resume a running workflow (should fail)
            let result = StateManager::resume_workflow(&mut state);
            prop_assert!(result.is_err());

            // State should be unchanged
            prop_assert_eq!(state.status, WorkflowStatus::Running);
        }
    }
}
