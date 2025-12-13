use ricecoder_workflows::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AgentStep, ErrorAction, RiskFactors, StepConfig, StepStatus, StepType, WorkflowConfig,
        WorkflowStep,
    };

    fn create_test_workflow() -> Workflow {
        Workflow {
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
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
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
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["condition".to_string()],
                    approval_required: false,
                    on_error: ErrorAction::Fail,
                    risk_score: None,
                    risk_factors: RiskFactors::default(),
                },
                WorkflowStep {
                    id: "step3".to_string(),
                    name: "Step 3".to_string(),
                    step_type: StepType::Agent(AgentStep {
                        agent_id: "test-agent".to_string(),
                        task: "test-task".to_string(),
                    }),
                    config: StepConfig {
                        config: serde_json::json!({}),
                    },
                    dependencies: vec!["condition".to_string()],
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
        }
    }

    #[test]
    fn test_parse_value_string() {
        let value = ConditionEvaluator::parse_value("'hello'");
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_value_number() {
        let value = ConditionEvaluator::parse_value("42");
        assert_eq!(value.as_i64(), Some(42));
    }

    #[test]
    fn test_parse_value_boolean() {
        let value = ConditionEvaluator::parse_value("true");
        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn test_parse_value_null() {
        let value = ConditionEvaluator::parse_value("null");
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_evaluate_equality_true() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        // Add a step result with output
        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"status": "completed"})),
                error: None,
                duration_ms: 100,
            },
        );

        let result = ConditionEvaluator::evaluate_equality(
            "step1.output.status",
            "'completed'",
            &workflow,
            &state,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_equality_false() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"status": "failed"})),
                error: None,
                duration_ms: 100,
            },
        );

        let result = ConditionEvaluator::evaluate_equality(
            "step1.output.status",
            "'completed'",
            &workflow,
            &state,
        );
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_evaluate_greater_than_true() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 10})),
                error: None,
                duration_ms: 100,
            },
        );

        let result =
            ConditionEvaluator::evaluate_greater_than("step1.output.count", "5", &workflow, &state);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_greater_than_false() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 3})),
                error: None,
                duration_ms: 100,
            },
        );

        let result =
            ConditionEvaluator::evaluate_greater_than("step1.output.count", "5", &workflow, &state);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_evaluate_condition_then_branch() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 10})),
                error: None,
                duration_ms: 100,
            },
        );

        let condition_step = ConditionStep {
            condition: "step1.output.count > 5".to_string(),
            then_steps: vec!["step2".to_string()],
            else_steps: vec!["step3".to_string()],
        };

        let result = ConditionEvaluator::evaluate_condition(&workflow, &state, &condition_step);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["step2".to_string()]);
    }

    #[test]
    fn test_evaluate_condition_else_branch() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 3})),
                error: None,
                duration_ms: 100,
            },
        );

        let condition_step = ConditionStep {
            condition: "step1.output.count > 5".to_string(),
            then_steps: vec!["step2".to_string()],
            else_steps: vec!["step3".to_string()],
        };

        let result = ConditionEvaluator::evaluate_condition(&workflow, &state, &condition_step);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["step3".to_string()]);
    }

    #[test]
    fn test_evaluate_not_equal() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"status": "failed"})),
                error: None,
                duration_ms: 100,
            },
        );

        let result = ConditionEvaluator::evaluate_expression(
            "step1.output.status != 'completed'",
            &workflow,
            &state,
        );
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_less_than() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 3})),
                error: None,
                duration_ms: 100,
            },
        );

        let result =
            ConditionEvaluator::evaluate_expression("step1.output.count < 5", &workflow, &state);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_greater_equal() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 5})),
                error: None,
                duration_ms: 100,
            },
        );

        let result =
            ConditionEvaluator::evaluate_expression("step1.output.count >= 5", &workflow, &state);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_evaluate_less_equal() {
        let workflow = create_test_workflow();
        let mut state = crate::state::StateManager::create_state(&workflow);

        state.step_results.insert(
            "step1".to_string(),
            crate::models::StepResult {
                status: StepStatus::Completed,
                output: Some(serde_json::json!({"count": 5})),
                error: None,
                duration_ms: 100,
            },
        );

        let result =
            ConditionEvaluator::evaluate_expression("step1.output.count <= 5", &workflow, &state);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}