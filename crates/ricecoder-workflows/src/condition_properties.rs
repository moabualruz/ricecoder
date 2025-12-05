//! Property-based tests for conditional branching
//!
//! **Feature: ricecoder-workflows, Property 2: Conditional Branching Correctness**
//! **Validates: Requirements 1.2**

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use crate::models::*;
    use crate::condition::ConditionEvaluator;
    use crate::state::StateManager;

    /// Strategy for generating valid step IDs
    fn step_id_strategy() -> impl Strategy<Value = String> {
        "[a-z0-9_]{1,20}".prop_map(|s| format!("step_{}", s))
    }

    /// Strategy for generating JSON values
    fn json_value_strategy() -> impl Strategy<Value = serde_json::Value> {
        prop_oneof![
            Just(serde_json::json!({"status": "success", "count": 10})),
            Just(serde_json::json!({"status": "failed", "count": 3})),
            Just(serde_json::json!({"status": "pending", "count": 0})),
            Just(serde_json::json!({"status": "completed", "count": 100})),
            Just(serde_json::json!({"status": "success", "count": 5})),
        ]
    }

    proptest! {
        /// Property: For any workflow with conditional steps, when a condition evaluates to true,
        /// the then-branch steps SHALL execute; when false, the else-branch steps SHALL execute.
        #[test]
        fn prop_condition_branching_correctness(
            then_step_id in step_id_strategy(),
            else_step_id in step_id_strategy(),
            output_value in json_value_strategy(),
        ) {
            // Create a workflow with a condition step
            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],
                steps: vec![
                    WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: StepType::Agent(AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                    },
                    WorkflowStep {
                        id: "condition".to_string(),
                        name: "Condition".to_string(),
                        step_type: StepType::Condition(ConditionStep {
                            condition: "step1.output.count > 5".to_string(),
                            then_steps: vec![then_step_id.clone()],
                            else_steps: vec![else_step_id.clone()],
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec!["step1".to_string()],
                        approval_required: false,
                        on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                    },
                ],
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Create workflow state
            let mut state = StateManager::create_state(&workflow);

            // Add step1 result with the generated output
            state.step_results.insert(
                "step1".to_string(),
                StepResult {
                    status: StepStatus::Completed,
                    output: Some(output_value.clone()),
                    error: None,
                    duration_ms: 100,
                },
            );
            state.completed_steps.push("step1".to_string());

            // Get the condition step
            let condition_step = match &workflow.steps[1].step_type {
                StepType::Condition(cs) => cs,
                _ => panic!("Expected condition step"),
            };

            // Evaluate the condition
            let result = ConditionEvaluator::evaluate_condition(&workflow, &state, condition_step);
            prop_assert!(result.is_ok(), "Condition evaluation should succeed");

            let next_steps = result.unwrap();

            // Verify that the correct branch is selected
            let count = output_value.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
            if count > 5 {
                // Then branch should be selected
                prop_assert_eq!(next_steps, vec![then_step_id.clone()], 
                    "Then branch should be selected when condition is true");
            } else {
                // Else branch should be selected
                prop_assert_eq!(next_steps, vec![else_step_id.clone()], 
                    "Else branch should be selected when condition is false");
            }
        }

        /// Property: For any condition step, the evaluation result is deterministic.
        /// The same condition and state should always produce the same result.
        #[test]
        fn prop_condition_evaluation_is_deterministic(
            output_value in json_value_strategy(),
        ) {
            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],steps: vec![
                    WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: StepType::Agent(AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                    },
                    WorkflowStep {
                        id: "condition".to_string(),
                        name: "Condition".to_string(),
                        step_type: StepType::Condition(ConditionStep {
                            condition: "step1.output.count > 5".to_string(),
                            then_steps: vec!["step2".to_string()],
                            else_steps: vec!["step3".to_string()],
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec!["step1".to_string()],
                        approval_required: false,
                        on_error: ErrorAction::Fail, risk_score: None, risk_factors: RiskFactors::default(),
                    },
                ],
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Create workflow state
            let mut state = StateManager::create_state(&workflow);

            // Add step1 result
            state.step_results.insert(
                "step1".to_string(),
                StepResult {
                    status: StepStatus::Completed,
                    output: Some(output_value.clone()),
                    error: None,
                    duration_ms: 100,
                },
            );
            state.completed_steps.push("step1".to_string());

            // Get the condition step
            let condition_step = match &workflow.steps[1].step_type {
                StepType::Condition(cs) => cs,
                _ => panic!("Expected condition step"),
            };

            // Evaluate the condition multiple times
            let result1 = ConditionEvaluator::evaluate_condition(&workflow, &state, condition_step);
            let result2 = ConditionEvaluator::evaluate_condition(&workflow, &state, condition_step);
            let result3 = ConditionEvaluator::evaluate_condition(&workflow, &state, condition_step);

            // All results should be the same
            prop_assert!(result1.is_ok() && result2.is_ok() && result3.is_ok());
            let r1 = result1.as_ref().unwrap();
            let r2 = result2.as_ref().unwrap();
            let r3 = result3.as_ref().unwrap();
            prop_assert_eq!(r1, r2, 
                "Condition evaluation should be deterministic");
            prop_assert_eq!(r2, r3, 
                "Condition evaluation should be deterministic");
        }

        /// Property: For any condition step with multiple then-steps or else-steps,
        /// all steps in the selected branch should be returned.
        #[test]
        fn prop_condition_returns_all_branch_steps(
            num_then_steps in 1..5usize,
            num_else_steps in 1..5usize,
            output_value in json_value_strategy(),
        ) {
            // Generate step IDs for then and else branches
            let then_steps: Vec<String> = (0..num_then_steps)
                .map(|i| format!("then_step_{}", i))
                .collect();
            let else_steps: Vec<String> = (0..num_else_steps)
                .map(|i| format!("else_step_{}", i))
                .collect();

            let workflow = Workflow {
                id: "test-workflow".to_string(),
                name: "Test Workflow".to_string(),
                description: "A test workflow".to_string(),
                parameters: vec![],steps: vec![
                    WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: StepType::Agent(AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: RiskFactors::default(),
                    },
                    WorkflowStep {
                        id: "condition".to_string(),
                        name: "Condition".to_string(),
                        step_type: StepType::Condition(ConditionStep {
                            condition: "step1.output.count > 5".to_string(),
                            then_steps: then_steps.clone(),
                            else_steps: else_steps.clone(),
                        }),
                        config: StepConfig {
                            config: serde_json::json!({}),
                        },
                        dependencies: vec!["step1".to_string()],
                        approval_required: false,
                        on_error: ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: RiskFactors::default(),
                    },
                ],
                config: WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Create workflow state
            let mut state = StateManager::create_state(&workflow);

            // Add step1 result
            state.step_results.insert(
                "step1".to_string(),
                StepResult {
                    status: StepStatus::Completed,
                    output: Some(output_value.clone()),
                    error: None,
                    duration_ms: 100,
                },
            );
            state.completed_steps.push("step1".to_string());

            // Get the condition step
            let condition_step = match &workflow.steps[1].step_type {
                StepType::Condition(cs) => cs,
                _ => panic!("Expected condition step"),
            };

            // Evaluate the condition
            let result = ConditionEvaluator::evaluate_condition(&workflow, &state, condition_step);
            prop_assert!(result.is_ok());

            let next_steps = result.unwrap();

            // Verify that all steps in the selected branch are returned
            let count = output_value.get("count").and_then(|v| v.as_i64()).unwrap_or(0);
            if count > 5 {
                prop_assert_eq!(next_steps.len(), then_steps.len(), 
                    "All then-steps should be returned");
                prop_assert_eq!(next_steps, then_steps, 
                    "Returned steps should match then-steps");
            } else {
                prop_assert_eq!(next_steps.len(), else_steps.len(), 
                    "All else-steps should be returned");
                prop_assert_eq!(next_steps, else_steps, 
                    "Returned steps should match else-steps");
            }
        }
    }
}


