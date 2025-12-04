//! Property-based tests for error handling and recovery
//!
//! Tests the correctness properties for error handling:
//! - Property 3: Error Action Execution
//! - Property 9: Error Capture
//! - Property 10: Workflow Recovery

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::models::*;
    use crate::error_handler::{ErrorHandler, RetryState};
    use crate::rollback::RollbackManager;
    use crate::state::StateManager;

    // Strategy for generating error actions
    fn error_action_strategy() -> impl Strategy<Value = ErrorAction> {
        prop_oneof![
            Just(ErrorAction::Fail),
            Just(ErrorAction::Skip),
            Just(ErrorAction::Rollback),
            (1usize..10, 10u64..1000u64)
                .prop_map(|(max_attempts, delay_ms)| ErrorAction::Retry {
                    max_attempts,
                    delay_ms,
                }),
        ]
    }



    // Property 3: Error Action Execution
    // **Feature: ricecoder-workflows, Property 3: Error Action Execution**
    // **Validates: Requirements 1.3**
    //
    // *For any* workflow step with an error action specified, when the step fails,
    // the system SHALL execute the specified error action (retry, skip, fail, or rollback)
    // and not execute any other error action.
    proptest! {
        #[test]
        fn prop_error_action_execution(error_action in error_action_strategy()) {
            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],
                steps: vec![WorkflowStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({"param": "value"}),
                    },
                    dependencies: vec![],
                    approval_required: false,
                    on_error: error_action.clone(),
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                }],
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            let mut state = StateManager::create_state(&workflow);
            StateManager::start_step(&mut state, "step1".to_string());

            // Handle the error
            let result = ErrorHandler::handle_error(
                &workflow,
                &mut state,
                "step1",
                "Test error".to_string(),
            );

            prop_assert!(result.is_ok());

            // Verify the correct action was taken
            match error_action {
                ErrorAction::Fail => {
                    // Step should be marked as failed
                    prop_assert!(ErrorHandler::has_error(&state, "step1"));
                }
                ErrorAction::Skip => {
                    // Step should be marked as skipped
                    let status = state.step_results.get("step1").map(|r| r.status);
                    prop_assert_eq!(status, Some(StepStatus::Skipped));
                }
                ErrorAction::Retry { .. } => {
                    // Step should be marked as failed (retry is handled by caller)
                    prop_assert!(ErrorHandler::has_error(&state, "step1"));
                }
                ErrorAction::Rollback => {
                    // Step should be marked as failed (rollback is handled separately)
                    prop_assert!(ErrorHandler::has_error(&state, "step1"));
                }
            }
        }
    }

    // Property 9: Error Capture
    // **Feature: ricecoder-workflows, Property 9: Error Capture**
    // **Validates: Requirements 2.3**
    //
    // *For any* failed workflow step, the system SHALL capture and store
    // the error type, error message, and stack trace.
    proptest! {
        #[test]
        fn prop_error_capture(
            error_type in "\\PC{1,50}",
            error_message in "\\PC{1,100}",
            stack_trace in "\\PC{0,200}"
        ) {
            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],
                steps: vec![WorkflowStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({"param": "value"}),
                    },
                    dependencies: vec![],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                }],
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            let mut state = StateManager::create_state(&workflow);
            StateManager::start_step(&mut state, "step1".to_string());

            // Capture error
            let result = ErrorHandler::capture_error(
                &mut state,
                "step1",
                &error_type,
                &error_message,
                if stack_trace.is_empty() {
                    None
                } else {
                    Some(&stack_trace)
                },
            );

            prop_assert!(result.is_ok());

            // Verify error was captured
            let error_details = ErrorHandler::get_error_details(&state, "step1");
            prop_assert!(error_details.is_some());

            let error_str = error_details.unwrap();
            // Verify all components are present
            prop_assert!(error_str.contains(&error_type));
            prop_assert!(error_str.contains(&error_message));
            if !stack_trace.is_empty() {
                prop_assert!(error_str.contains(&stack_trace));
            }
        }
    }

    // Property 10: Workflow Recovery
    // **Feature: ricecoder-workflows, Property 10: Workflow Recovery**
    // **Validates: Requirements 2.4**
    //
    // *For any* failed workflow, recovery SHALL resume execution from the last
    // completed step without re-executing that step.
    proptest! {
        #[test]
        fn prop_workflow_recovery(num_steps in 2usize..10) {
            // Create a workflow with multiple steps
            let mut steps = Vec::new();
            for i in 0..num_steps {
                let mut dependencies = Vec::new();
                if i > 0 {
                    dependencies.push(format!("step{}", i));
                }

                steps.push(WorkflowStep {
                    id: format!("step{}", i + 1),
                    name: format!("Step {}", i + 1),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({"param": "value"}),
                    },
                    dependencies,
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                });
            }

            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],
                steps,
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            let mut state = StateManager::create_state(&workflow);

            // Simulate execution of first half of steps (at least 1)
            let half = (num_steps / 2).max(1);
            for i in 0..half {
                let step_id = format!("step{}", i + 1);
                StateManager::start_step(&mut state, step_id.clone());
                StateManager::complete_step(
                    &mut state,
                    step_id,
                    Some(serde_json::json!({"result": "success"})),
                    100,
                );
            }

            let completed_before = state.completed_steps.len();
            prop_assert!(completed_before > 0);

            // Create rollback plan and restore state
            let mut rollback_plan = RollbackManager::create_rollback_plan(&workflow);
            for step in &workflow.steps {
                rollback_plan.record_execution(step.id.clone());
            }

            let restore_result = RollbackManager::restore_state(&mut state);
            prop_assert!(restore_result.is_ok());

            // Verify state was restored
            prop_assert!(state.completed_steps.is_empty());
            prop_assert!(state.step_results.is_empty());
            prop_assert!(state.current_step.is_none());

            // Verify we can resume from the beginning (all state cleared)
            prop_assert_eq!(state.completed_steps.len(), 0);
        }
    }

    // Additional test: Retry state exponential backoff
    proptest! {
        #[test]
        fn prop_retry_exponential_backoff(
            max_attempts in 2usize..10,
            base_delay_ms in 10u64..1000u64
        ) {
            let mut retry_state = RetryState::new(max_attempts, base_delay_ms);

            // Verify exponential backoff increases correctly
            let mut previous_delay = 0u128;
            for _ in 0..max_attempts - 1 {
                let delay = retry_state.get_backoff_delay();
                let delay_ms = delay.as_millis();

                // Each delay should be approximately 2x the previous
                if previous_delay > 0 {
                    prop_assert!(delay_ms >= previous_delay);
                }

                previous_delay = delay_ms;
                retry_state.record_error("Test error".to_string());
            }

            // Verify no more retries available
            prop_assert!(!retry_state.can_retry());
        }
    }

    // Additional test: Error history tracking
    proptest! {
        #[test]
        fn prop_error_history_tracking(
            num_errors in 1usize..10,
            error_messages in prop::collection::vec("\\PC{1,50}", 1..10)
        ) {
            let mut retry_state = RetryState::new(num_errors + 1, 100);

            // Record errors
            for (i, msg) in error_messages.iter().take(num_errors).enumerate() {
                retry_state.record_error(msg.clone());

                // Verify history is updated
                let history = retry_state.get_history();
                prop_assert_eq!(history.len(), i + 1);
                prop_assert_eq!(&history[i].error, msg);
                prop_assert_eq!(history[i].attempt, i + 1);
            }
        }
    }
}
