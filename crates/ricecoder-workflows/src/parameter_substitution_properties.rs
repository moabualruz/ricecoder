//! Property-based tests for parameter substitution

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// **Feature: ricecoder-workflows, Property 5: Parameter Substitution**
    /// **Validates: Requirements 1.5**
    ///
    /// For any workflow definition with parameters, when parameter values are provided
    /// at execution time, all parameter references in step configurations SHALL be
    /// replaced with the provided values.
    #[test]
    fn prop_parameter_substitution_replaces_all_references() {
        proptest!(|(
            param_name in "[a-z_][a-z0-9_]{0,19}",
            param_value in "[a-zA-Z0-9]{1,50}",
        )| {
            // Create a workflow with a parameter
            let mut workflow = crate::models::Workflow {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test workflow".to_string(),
                parameters: vec![
                    crate::models::WorkflowParameter {
                        name: param_name.clone(),
                        param_type: "string".to_string(),
                        default: None,
                        required: true,
                        description: "Test parameter".to_string(),
                    },
                ],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: json!({
                                "message": format!("Hello ${{{}}}", param_name),
                                "nested": {
                                    "value": format!("Value: ${{{}}}", param_name)
                                }
                            }),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Create parameter values
            let mut params = HashMap::new();
            params.insert(param_name.clone(), json!(param_value.clone()));

            // Perform substitution
            let result = crate::parameter_substitution::StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
            prop_assert!(result.is_ok(), "Substitution should succeed");

            // Verify all references were replaced
            let step = &workflow.steps[0];
            let message = step.config.config.get("message").and_then(|v| v.as_str());
            prop_assert!(message.is_some(), "Message should exist");
            prop_assert!(!message.unwrap().contains(&format!("${{{}}}", param_name)), "Parameter reference should be replaced");
            prop_assert!(message.unwrap().contains(&param_value), "Parameter value should be in message");

            let nested_value = step.config.config
                .get("nested")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_str());
            prop_assert!(nested_value.is_some(), "Nested value should exist");
            prop_assert!(!nested_value.unwrap().contains(&format!("${{{}}}", param_name)), "Nested parameter reference should be replaced");
            prop_assert!(nested_value.unwrap().contains(&param_value), "Parameter value should be in nested value");
        });
    }

    /// **Feature: ricecoder-workflows, Property 5: Parameter Substitution**
    /// **Validates: Requirements 1.5**
    ///
    /// For any workflow definition with parameters, when parameter values are provided
    /// at execution time, all parameter references in step configurations SHALL be
    /// replaced with the provided values. Multiple references to the same parameter
    /// should all be replaced.
    #[test]
    fn prop_parameter_substitution_handles_multiple_references() {
        proptest!(|(
            param_value in "[a-zA-Z0-9]{1,30}",
        )| {
            let mut workflow = crate::models::Workflow {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test workflow".to_string(),
                parameters: vec![
                    crate::models::WorkflowParameter {
                        name: "value".to_string(),
                        param_type: "string".to_string(),
                        default: None,
                        required: true,
                        description: "Test parameter".to_string(),
                    },
                ],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: json!({
                                "first": "${value}",
                                "second": "${value}",
                                "combined": "${value}-${value}"
                            }),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            let mut params = HashMap::new();
            params.insert("value".to_string(), json!(param_value.clone()));

            let result = crate::parameter_substitution::StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
            prop_assert!(result.is_ok(), "Substitution should succeed");

            let step = &workflow.steps[0];
            let first = step.config.config.get("first").and_then(|v| v.as_str());
            let second = step.config.config.get("second").and_then(|v| v.as_str());
            let combined = step.config.config.get("combined").and_then(|v| v.as_str());
            let expected_combined = format!("{}-{}", param_value, param_value);

            prop_assert_eq!(first, Some(param_value.as_str()), "First reference should be replaced");
            prop_assert_eq!(second, Some(param_value.as_str()), "Second reference should be replaced");
            prop_assert_eq!(combined, Some(expected_combined.as_str()), "Combined references should be replaced");
        });
    }

    /// **Feature: ricecoder-workflows, Property 5: Parameter Substitution**
    /// **Validates: Requirements 1.5**
    ///
    /// For any workflow definition with parameters, when parameter values are provided
    /// at execution time, missing required parameters should cause an error.
    #[test]
    fn prop_parameter_substitution_requires_all_parameters() {
        proptest!(|(
            param_name in "[a-z_][a-z0-9_]{0,19}",
        )| {
            let mut workflow = crate::models::Workflow {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test workflow".to_string(),
                parameters: vec![
                    crate::models::WorkflowParameter {
                        name: param_name.clone(),
                        param_type: "string".to_string(),
                        default: None,
                        required: true,
                        description: "Test parameter".to_string(),
                    },
                ],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Don't provide the required parameter
            let params = HashMap::new();

            let result = crate::parameter_substitution::StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
            prop_assert!(result.is_err(), "Substitution should fail when required parameter is missing");
        });
    }

    /// **Feature: ricecoder-workflows, Property 5: Parameter Substitution**
    /// **Validates: Requirements 1.5**
    ///
    /// For any workflow definition with parameters, when unknown parameters are provided,
    /// an error should be returned.
    #[test]
    fn prop_parameter_substitution_rejects_unknown_parameters() {
        proptest!(|(
            unknown_param in "[a-z_][a-z0-9_]{0,19}",
        )| {
            let mut workflow = crate::models::Workflow {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test workflow".to_string(),
                parameters: vec![
                    crate::models::WorkflowParameter {
                        name: "known".to_string(),
                        param_type: "string".to_string(),
                        default: None,
                        required: true,
                        description: "Test parameter".to_string(),
                    },
                ],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: json!({}),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Provide an unknown parameter
            let mut params = HashMap::new();
            params.insert("known".to_string(), json!("value"));
            if unknown_param != "known" {
                params.insert(unknown_param.clone(), json!("unknown_value"));

                let result = crate::parameter_substitution::StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
                prop_assert!(result.is_err(), "Substitution should fail when unknown parameter is provided");
            }
        });
    }

    /// **Feature: ricecoder-workflows, Property 5: Parameter Substitution**
    /// **Validates: Requirements 1.5**
    ///
    /// For any workflow definition with parameters, when default values are provided,
    /// they should be used when parameters are not explicitly provided.
    #[test]
    fn prop_parameter_substitution_uses_defaults() {
        proptest!(|(
            default_value in "[a-zA-Z0-9]{1,30}",
        )| {
            let mut workflow = crate::models::Workflow {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test workflow".to_string(),
                parameters: vec![
                    crate::models::WorkflowParameter {
                        name: "optional".to_string(),
                        param_type: "string".to_string(),
                        default: Some(json!(default_value.clone())),
                        required: false,
                        description: "Optional parameter".to_string(),
                    },
                ],
                steps: vec![
                    crate::models::WorkflowStep {
                        id: "step1".to_string(),
                        name: "Step 1".to_string(),
                        step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                            agent_id: "test-agent".to_string(),
                            task: "test-task".to_string(),
                        }),
                        config: crate::models::StepConfig {
                            config: json!({
                                "message": "${optional}"
                            }),
                        },
                        dependencies: vec![],
                        approval_required: false,
                        on_error: crate::models::ErrorAction::Fail,
                        risk_score: None,
                        risk_factors: crate::models::RiskFactors::default(),
                    },
                ],
                config: crate::models::WorkflowConfig {
                    timeout_ms: None,
                    max_parallel: None,
                },
            };

            // Don't provide the optional parameter
            let params = HashMap::new();

            let result = crate::parameter_substitution::StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
            prop_assert!(result.is_ok(), "Substitution should succeed with default value");

            let step = &workflow.steps[0];
            let message = step.config.config.get("message").and_then(|v| v.as_str());
            prop_assert_eq!(message, Some(default_value.as_str()), "Default value should be used");
        });
    }
}
