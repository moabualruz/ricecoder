//! Parameter substitution in workflow step configurations

use crate::error::{WorkflowError, WorkflowResult};
use crate::models::{Workflow, WorkflowStep};
use crate::parameters::ParameterSubstitutor;
use serde_json::Value;
use std::collections::HashMap;

/// Handles parameter substitution in workflow step configurations
pub struct StepConfigSubstitutor;

impl StepConfigSubstitutor {
    /// Substitute parameters in all step configurations
    ///
    /// Replaces all parameter references (${param_name}) in step configurations
    /// with their provided values. Handles nested parameter references and validates
    /// all parameters are provided at execution time.
    pub fn substitute_in_workflow(
        workflow: &mut Workflow,
        parameters: &HashMap<String, Value>,
    ) -> WorkflowResult<()> {
        // Build final parameters with defaults
        let final_params = Self::build_final_parameters(workflow, parameters)?;

        // Substitute in each step's configuration
        for step in &mut workflow.steps {
            Self::substitute_in_step(step, &final_params)?;
        }

        Ok(())
    }

    /// Substitute parameters in a single step's configuration
    pub fn substitute_in_step(
        step: &mut WorkflowStep,
        parameters: &HashMap<String, Value>,
    ) -> WorkflowResult<()> {
        // Substitute in the step config
        let substituted = ParameterSubstitutor::substitute(&step.config.config, parameters)?;
        step.config.config = substituted;

        Ok(())
    }

    /// Validate that all required parameters are provided
    fn validate_all_parameters_provided(
        workflow: &Workflow,
        parameters: &HashMap<String, Value>,
    ) -> WorkflowResult<()> {
        // Check that all required parameters are provided
        for param_def in &workflow.parameters {
            if param_def.required
                && !parameters.contains_key(&param_def.name)
                && param_def.default.is_none()
            {
                return Err(WorkflowError::Invalid(format!(
                    "Required parameter '{}' not provided",
                    param_def.name
                )));
            }
        }

        // Check that no unknown parameters are provided
        let known_names: std::collections::HashSet<_> =
            workflow.parameters.iter().map(|p| &p.name).collect();

        for provided_name in parameters.keys() {
            if !known_names.contains(provided_name) {
                return Err(WorkflowError::Invalid(format!(
                    "Unknown parameter: {}",
                    provided_name
                )));
            }
        }

        Ok(())
    }

    /// Build final parameter values with defaults
    ///
    /// Merges provided values with defaults from parameter definitions
    pub fn build_final_parameters(
        workflow: &Workflow,
        provided: &HashMap<String, Value>,
    ) -> WorkflowResult<HashMap<String, Value>> {
        Self::validate_all_parameters_provided(workflow, provided)?;

        let mut final_params = HashMap::new();

        for param_def in &workflow.parameters {
            if let Some(value) = provided.get(&param_def.name) {
                final_params.insert(param_def.name.clone(), value.clone());
            } else if let Some(default) = &param_def.default {
                final_params.insert(param_def.name.clone(), default.clone());
            }
        }

        Ok(final_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StepConfig;
    use serde_json::json;

    fn create_test_workflow() -> Workflow {
        Workflow {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test workflow".to_string(),
            parameters: vec![
                crate::models::WorkflowParameter {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    default: Some(json!("default-name")),
                    required: false,
                    description: "Name parameter".to_string(),
                },
                crate::models::WorkflowParameter {
                    name: "count".to_string(),
                    param_type: "number".to_string(),
                    default: None,
                    required: true,
                    description: "Count parameter".to_string(),
                },
            ],
            steps: vec![crate::models::WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: crate::models::StepType::Agent(crate::models::AgentStep {
                    agent_id: "test-agent".to_string(),
                    task: "test-task".to_string(),
                }),
                config: StepConfig {
                    config: json!({
                        "message": "Hello ${name}",
                        "count": "${count}"
                    }),
                },
                dependencies: vec![],
                approval_required: false,
                on_error: crate::models::ErrorAction::Fail,
                risk_score: None,
                risk_factors: crate::models::RiskFactors::default(),
            }],
            config: crate::models::WorkflowConfig {
                timeout_ms: None,
                max_parallel: None,
            },
        }
    }

    #[test]
    fn test_substitute_in_step_config() {
        let mut workflow = create_test_workflow();
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("count".to_string(), json!(42));

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_ok());

        let step = &workflow.steps[0];
        assert_eq!(
            step.config.config.get("message").and_then(|v| v.as_str()),
            Some("Hello Alice")
        );
    }

    #[test]
    fn test_substitute_missing_required_parameter() {
        let mut workflow = create_test_workflow();
        let params = HashMap::new();

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_substitute_unknown_parameter() {
        let mut workflow = create_test_workflow();
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("count".to_string(), json!(42));
        params.insert("unknown".to_string(), json!("value"));

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_final_parameters_with_defaults() {
        let workflow = create_test_workflow();
        let mut provided = HashMap::new();
        provided.insert("count".to_string(), json!(42));

        let result = StepConfigSubstitutor::build_final_parameters(&workflow, &provided);
        assert!(result.is_ok());

        let final_params = result.unwrap();
        assert_eq!(final_params.get("name"), Some(&json!("default-name")));
        assert_eq!(final_params.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_substitute_nested_parameters() {
        let mut workflow = create_test_workflow();
        workflow.steps[0].config.config = json!({
            "nested": {
                "message": "Hello ${name}",
                "items": ["${name}", "world"]
            }
        });

        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("count".to_string(), json!(42));

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_ok());

        let step = &workflow.steps[0];
        assert_eq!(
            step.config
                .config
                .get("nested")
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str()),
            Some("Hello Alice")
        );
    }

    #[test]
    fn test_substitute_multiple_references_in_string() {
        let mut workflow = create_test_workflow();
        workflow.steps[0].config.config = json!({
            "message": "${name} has ${count} items"
        });

        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("Alice"));
        params.insert("count".to_string(), json!(42));

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_ok());

        let step = &workflow.steps[0];
        assert_eq!(
            step.config.config.get("message").and_then(|v| v.as_str()),
            Some("Alice has 42 items")
        );
    }

    #[test]
    fn test_substitute_with_default_parameter() {
        let mut workflow = create_test_workflow();
        let mut params = HashMap::new();
        params.insert("count".to_string(), json!(42));
        // name is not provided, should use default

        let result = StepConfigSubstitutor::substitute_in_workflow(&mut workflow, &params);
        assert!(result.is_ok());

        let step = &workflow.steps[0];
        assert_eq!(
            step.config.config.get("message").and_then(|v| v.as_str()),
            Some("Hello default-name")
        );
    }
}
