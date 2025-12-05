//! Property-based tests for state persistence
//! **Feature: ricecoder-workflows, Property 7: State Persistence**
//! **Validates: Requirements 2.1**

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::state::StateManager;
    use proptest::prelude::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Strategy for generating WorkflowState
    fn arb_workflow_state() -> impl Strategy<Value = WorkflowState> {
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
        /// Property 7: State Persistence
        /// For any workflow execution, the persisted state SHALL contain the current step,
        /// all completed steps, and results for all executed steps.
        #[test]
        fn prop_state_persistence_round_trip(state in arb_workflow_state()) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let state_path = temp_dir.path().join("workflow_state.yaml");

            // Persist the state
            StateManager::persist_state(&state, &state_path)
                .expect("Failed to persist state");

            // Load the state back
            let loaded_state = StateManager::load_state(&state_path)
                .expect("Failed to load state");

            // Verify all properties are preserved
            prop_assert_eq!(loaded_state.workflow_id, state.workflow_id);
            prop_assert_eq!(loaded_state.status, state.status);
            prop_assert_eq!(loaded_state.current_step, state.current_step);
            prop_assert_eq!(loaded_state.completed_steps, state.completed_steps);

            // Verify all step results are preserved
            prop_assert_eq!(loaded_state.step_results.len(), state.step_results.len());
            for (step_id, result) in &state.step_results {
                let loaded_result = loaded_state.step_results.get(step_id)
                    .expect(&format!("Missing result for step {}", step_id));
                prop_assert_eq!(loaded_result.status, result.status);
                prop_assert_eq!(&loaded_result.output, &result.output);
                prop_assert_eq!(&loaded_result.error, &result.error);
                prop_assert_eq!(loaded_result.duration_ms, result.duration_ms);
            }
        }

        /// Property 7: State Persistence (JSON format)
        /// For any workflow execution, the persisted state in JSON format SHALL contain
        /// the current step, all completed steps, and results for all executed steps.
        #[test]
        fn prop_state_persistence_json_round_trip(state in arb_workflow_state()) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let state_path = temp_dir.path().join("workflow_state.json");

            // Persist the state in JSON format
            StateManager::persist_state_json(&state, &state_path)
                .expect("Failed to persist state");

            // Load the state back
            let loaded_state = StateManager::load_state(&state_path)
                .expect("Failed to load state");

            // Verify all properties are preserved
            prop_assert_eq!(loaded_state.workflow_id, state.workflow_id);
            prop_assert_eq!(loaded_state.status, state.status);
            prop_assert_eq!(loaded_state.current_step, state.current_step);
            prop_assert_eq!(loaded_state.completed_steps, state.completed_steps);

            // Verify all step results are preserved
            prop_assert_eq!(loaded_state.step_results.len(), state.step_results.len());
            for (step_id, result) in &state.step_results {
                let loaded_result = loaded_state.step_results.get(step_id)
                    .expect(&format!("Missing result for step {}", step_id));
                prop_assert_eq!(loaded_result.status, result.status);
                prop_assert_eq!(&loaded_result.output, &result.output);
                prop_assert_eq!(&loaded_result.error, &result.error);
                prop_assert_eq!(loaded_result.duration_ms, result.duration_ms);
            }
        }

        /// Property 7: State Persistence - Validation
        /// For any persisted state, loading and validating it SHALL succeed
        /// if all completed steps have results.
        #[test]
        fn prop_state_persistence_validation(state in arb_workflow_state()) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let state_path = temp_dir.path().join("workflow_state.yaml");

            // Persist the state
            StateManager::persist_state(&state, &state_path)
                .expect("Failed to persist state");

            // Load and validate the state
            let loaded_state = StateManager::load_state_validated(&state_path)
                .expect("Failed to load and validate state");

            // Verify the state is valid
            prop_assert_eq!(loaded_state.workflow_id, state.workflow_id);
            prop_assert_eq!(loaded_state.completed_steps, state.completed_steps);
        }

        /// Property 7: State Persistence - Directory Creation
        /// For any state persisted to a non-existent directory,
        /// the directory SHALL be created automatically.
        #[test]
        fn prop_state_persistence_creates_directory(state in arb_workflow_state()) {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let state_path = temp_dir.path()
                .join("nested")
                .join("deep")
                .join("workflow_state.yaml");

            // Persist the state (directory doesn't exist yet)
            StateManager::persist_state(&state, &state_path)
                .expect("Failed to persist state");

            // Verify the file was created
            prop_assert!(state_path.exists());

            // Load the state back
            let loaded_state = StateManager::load_state(&state_path)
                .expect("Failed to load state");

            prop_assert_eq!(loaded_state.workflow_id, state.workflow_id);
        }
    }
}
